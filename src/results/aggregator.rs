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
/// * `request_timeout` - Accepts a time (secs) as a value which controls the server request timeout.
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
    mut upstream_search_engines: Vec<String>,
    request_timeout: u8,
) -> Result<SearchResults, Box<dyn std::error::Error>> {
    let user_agent: String = random_user_agent();

    // Add a random delay before making the request.
    if random_delay || !debug {
        let mut rng = rand::thread_rng();
        let delay_secs = rng.gen_range(1..10);
        tokio::time::sleep(Duration::from_secs(delay_secs)).await;
    }

    // create tasks for upstream result fetching
    let tasks: FutureVec = upstream_search_engines
        .iter()
        .map(|engine| match engine.to_lowercase().as_str() {
            "duckduckgo" => Box::new(duckduckgo::DuckDuckGo) as Box<dyn SearchEngine + Send + Sync>,
            "searx" => Box::new(searx::Searx) as Box<dyn SearchEngine + Send + Sync>,
            &_ => panic!("Config Error: Incorrect config file option provided"),
        })
        .map(|search_engine| {
            let query: String = query.clone();
            let user_agent: String = user_agent.clone();
            tokio::spawn(async move {
                search_engine
                    .results(query, page, user_agent.clone(), request_timeout)
                    .await
            })
        })
        .collect();

    // get upstream responses
    let mut responses = Vec::with_capacity(tasks.len());

    for task in tasks {
        if let Ok(result) = task.await {
            responses.push(result)
        }
    }

    // aggregate search results, removing duplicates and handling errors the upstream engines returned
    let mut result_map: HashMap<String, RawSearchResult> = HashMap::new();
    let mut engine_errors_info: Vec<EngineErrorInfo> = Vec::new();

    let mut handle_error = |error: Report<EngineError>, engine_name: String| {
        log::error!("Engine Error: {:?}", error);
        engine_errors_info.push(EngineErrorInfo::new(
            error.downcast_ref::<EngineError>().unwrap(),
            engine_name,
        ));
    };

    for _ in 0..responses.len() {
        let response = responses.pop().unwrap();
        let engine_name = upstream_search_engines.pop().unwrap();

        if result_map.is_empty() {
            match response {
                Ok(results) => {
                    result_map = results.clone();
                }
                Err(error) => {
                    handle_error(error, engine_name.clone());
                }
            }
            continue;
        }

        match response {
            Ok(result) => {
                result.into_iter().for_each(|(key, value)| {
                    result_map
                        .entry(key)
                        .and_modify(|result| {
                            result.add_engines(engine_name.clone());
                        })
                        .or_insert_with(|| -> RawSearchResult { value });
                });
            }
            Err(error) => {
                handle_error(error, engine_name.clone());
            }
        }
    }

    let mut results = Vec::with_capacity(result_map.len());
    for (_, result) in result_map {
        results.push(SearchResult::from_raw(result))
    }

    Ok(SearchResults::new(
        results,
        query.to_string(),
        engine_errors_info,
    ))
}
