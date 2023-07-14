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

type FutureVec = Vec<JoinHandle<Result<HashMap<String, RawSearchResult>, Report<EngineError>>>>;

/// A function that aggregates all the scraped results from the above upstream engines and
/// then removes duplicate results and if two results are found to be from two or more engines
/// then puts their names together to show the results are fetched from these upstream engines
/// and then removes all data from the HashMap and puts into a struct of all results aggregated
/// into a vector and also adds the query used into the struct this is neccessory because
/// otherwise the search bar in search remains empty if searched from the query url
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
