//! This module provides the functionality to scrape and gathers all the results from the upstream
//! search engines and then removes duplicate results.

use std::{collections::HashMap, time::Duration};

use rand::Rng;
use tokio::join;

use super::{
    aggregation_models::{RawSearchResult, SearchResult, SearchResults},
    user_agent::random_user_agent,
};

use crate::engines::{duckduckgo, searx};

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
    query: &str,
    page: u32,
    random_delay: bool,
    debug: bool,
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
    let (ddg_map_results, searx_map_results) = join!(
        duckduckgo::results(query, page, &user_agent),
        searx::results(query, page, &user_agent)
    );

    let ddg_map_results: HashMap<String, RawSearchResult> = ddg_map_results?;
    let searx_map_results: HashMap<String, RawSearchResult> = searx_map_results?;

    result_map.extend(ddg_map_results);

    searx_map_results.into_iter().for_each(|(key, value)| {
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
    ))
}
