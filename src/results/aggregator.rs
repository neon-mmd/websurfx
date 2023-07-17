//! This module provides the functionality to scrape and gathers all the results from the upstream
//! search engines and then removes duplicate results.

use std::{collections::HashMap, time::Duration};

use error_stack::Report;
use rand::Rng;
use tokio::task::JoinHandle;

use super::{
    aggregation_models::{EngineErrorInfo, RawSearchResult, SearchResult, SearchResults},
    user_agent::random_user_agent,
};

use crate::engines::{
    duckduckgo,
    engine_models::{EngineError, SearchEngine},
    searx,
};

/// Aliases for long type annotations
type FutureVec = Vec<JoinHandle<Result<HashMap<String, RawSearchResult>, Report<EngineError>>>>;

/// The function aggregates the scraped results from the user-selected upstream search engines.
/// These engines can be chosen either from the user interface (UI) or from the configuration file.
/// The code handles this process by matching the selected search engines and adding them to a vector.
/// This vector is then used to create an asynchronous task vector using `tokio::spawn`, which returns
/// a future. This future is awaited in another loop. Once the results are collected, they are filtered
/// to remove any errors and ensure only proper results are included. If an error is encountered, it is
/// sent to the UI along with the name of the engine and the type of error. This information is finally
/// placed in the returned `SearchResults` struct.
///
/// Additionally, the function eliminates duplicate results. If two results are identified as coming from
/// multiple engines, their names are combined to indicate that the results were fetched from these upstream
/// engines. After this, all the data in the `HashMap` is removed and placed into a struct that contains all
/// the aggregated results in a vector. Furthermore, the query used is also added to the struct. This step is
/// necessary to ensure that the search bar in the search remains populated even when searched from the query URL.
///
/// Overall, this function serves to aggregate scraped results from user-selected search engines, handling errors,
/// removing duplicates, and organizing the data for display in the UI.
///
/// # Example:
///
/// If you search from the url like `https://127.0.0.1/search?q=huston` then the search bar should
/// contain the word huston and not remain empty.
///
/// # Arguments
///
/// * `query` - Accepts a string to query with the above upstream search engines.
/// * `page` - Accepts an u32 page number.
/// * `random_delay` - Accepts a boolean value to add a random delay before making the request.
/// * `debug` - Accepts a boolean value to enable or disable debug mode option.
/// * `upstream_search_engines` - Accepts a vector of search engine names which was selected by the
/// user through the UI or the config file.
///
/// # Error
///
/// Returns an error a reqwest and scraping selector errors if any error occurs in the results
/// function in either `searx` or `duckduckgo` or both otherwise returns a `SearchResults struct`
/// containing appropriate values.
pub async fn aggregate(
    query: String,
    page: u32,
    random_delay: bool,
    debug: bool,
    upstream_search_engines: Vec<String>,
) -> Result<SearchResults, Box<dyn std::error::Error>> {
    let user_agent: String = random_user_agent();
    let mut result_map: HashMap<String, RawSearchResult> = HashMap::new();

    // Add a random delay before making the request.
    if random_delay || !debug {
        let mut rng = rand::thread_rng();
        let delay_secs = rng.gen_range(1..10);
        std::thread::sleep(Duration::from_secs(delay_secs));
    }

    // fetch results from upstream search engines simultaneously/concurrently.
    let search_engines: Vec<Box<dyn SearchEngine + Send + Sync>> = upstream_search_engines
        .iter()
        .map(|engine| match engine.to_lowercase().as_str() {
            "duckduckgo" => Box::new(duckduckgo::DuckDuckGo) as Box<dyn SearchEngine + Send + Sync>,
            "searx" => Box::new(searx::Searx) as Box<dyn SearchEngine + Send + Sync>,
            &_ => panic!("Config Error: Incorrect config file option provided"),
        })
        .collect();

    let task_capacity: usize = search_engines.len();

    let tasks: FutureVec = search_engines
        .into_iter()
        .map(|search_engine| {
            let query: String = query.clone();
            let user_agent: String = user_agent.clone();
            tokio::spawn(
                async move { search_engine.results(query, page, user_agent.clone()).await },
            )
        })
        .collect();

    let mut outputs = Vec::with_capacity(task_capacity);

    for task in tasks {
        if let Ok(result) = task.await {
            outputs.push(result)
        }
    }

    let mut engine_errors_info: Vec<EngineErrorInfo> = Vec::new();

    // The code block `outputs.iter()` determines whether it is the first time the code is being run.
    // It does this by checking the initial flag. If it is the first time, the code selects the first
    // engine from which results are fetched and adds or extends them into the `result_map`. If the
    // initially selected engine fails, the code automatically selects another engine to map or extend
    // into the `result_map`. On the other hand, if an engine selected for the first time successfully
    // fetches results and maps them into the `result_map`, the initial flag is set to false. Subsequently,
    // the code iterates through the remaining engines one by one. It compares the fetched results from each
    // engine with the results already present in the `result_map` to identify any duplicates. If duplicate
    // results are found, the code groups them together with the name of the engine from which they were
    // fetched, and automatically removes the duplicate results from the newly fetched data.
    //
    // Additionally, the code handles errors returned by the engines. It keeps track of which engines
    // encountered errors and stores this information in a vector of structures called `EngineErrorInfo`.
    // Each structure in this vector contains the name of the engine and the type of error it returned.
    // These structures will later be added to the final `SearchResults` structure. The `SearchResults`
    // structure is used to display an error box in the UI containing the relevant information from
    // the `EngineErrorInfo` structure.
    //
    // In summary, this code block manages the selection of engines, handling of duplicate results, and tracking
    // of errors in order to populate the `result_map` and provide informative feedback to the user through the
    // `SearchResults` structure.
    let mut initial: bool = true;
    let mut counter: usize = 0;
    outputs.iter().for_each(|results| {
        if initial {
            match results {
                Ok(result) => {
                    result_map.extend(result.clone());
                    counter += 1;
                    initial = false
                }
                Err(error_type) => {
                    engine_errors_info.push(EngineErrorInfo::new(
                        error_type.downcast_ref::<EngineError>().unwrap(),
                        upstream_search_engines[counter].clone(),
                    ));
                    counter += 1
                }
            }
        } else {
            match results {
                Ok(result) => {
                    result.clone().into_iter().for_each(|(key, value)| {
                        result_map
                            .entry(key)
                            .and_modify(|result| {
                                result.add_engines(value.clone().engine());
                            })
                            .or_insert_with(|| -> RawSearchResult {
                                RawSearchResult::new(
                                    value.title.clone(),
                                    value.visiting_url.clone(),
                                    value.description.clone(),
                                    value.engine.clone(),
                                )
                            });
                    });
                    counter += 1
                }
                Err(error_type) => {
                    engine_errors_info.push(EngineErrorInfo::new(
                        error_type.downcast_ref::<EngineError>().unwrap(),
                        upstream_search_engines[counter].clone(),
                    ));
                    counter += 1
                }
            }
        }
    });

    Ok(SearchResults::new(
        result_map
            .into_iter()
            .map(|(key, value)| {
                SearchResult::new(
                    value.title,
                    value.visiting_url,
                    key,
                    value.description,
                    value.engine,
                )
            })
            .collect(),
        query.to_string(),
        engine_errors_info,
    ))
}
