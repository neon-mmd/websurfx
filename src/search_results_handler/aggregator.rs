use std::collections::HashMap;

use fake_useragent::{Browsers, UserAgentsBuilder};

use super::aggregation_models::{RawSearchResult, SearchResult, SearchResults};
use crate::engines::{duckduckgo, searx};

// A function that aggregates all the scraped results from the above upstream engines and
// then removes duplicate results and if two results are found to be from two or more engines
// then puts their names together to show the results are fetched from these upstream engines
// and then removes all data from the HashMap and puts into a struct of all results aggregated
// into a vector and also adds the query used into the struct this is neccessory because otherwise
// the search bar in search remains empty if searched from the query url
//
// For Example:
//
// If you search from the url like *https://127.0.0.1/search?q=huston* then the search bar should
// contain the word huston and not remain empty.
pub async fn aggregate(
    query: &str,
    page: Option<u32>,
) -> Result<SearchResults, Box<dyn std::error::Error>> {
    // Generate random user agent to improve privacy of the user.
    let user_agent: String = UserAgentsBuilder::new()
        .cache(false)
        .dir("/tmp")
        .thread(1)
        .set_browsers(
            Browsers::new()
                .set_chrome()
                .set_safari()
                .set_edge()
                .set_firefox()
                .set_mozilla(),
        )
        .build()
        .random()
        .to_string();

    let mut result_map: HashMap<String, RawSearchResult> = HashMap::new();

    let ddg_map_results: HashMap<String, RawSearchResult> =
        duckduckgo::results(query, page, &user_agent).await?;
    let searx_map_results: HashMap<String, RawSearchResult> =
        searx::results(query, page, &user_agent).await?;

    result_map.extend(ddg_map_results);

    for (key, value) in searx_map_results.into_iter() {
        if result_map.contains_key(&key) {
            result_map
                .get_mut(&key)
                .unwrap()
                .engine
                .push(value.engine.get(0).unwrap().to_string())
        } else {
            result_map.insert(key, value);
        }
    }

    let mut search_results: Vec<SearchResult> = Vec::new();

    for (key, value) in result_map.into_iter() {
        search_results.push(SearchResult {
            title: value.title,
            visiting_url: value.visiting_url,
            url: key,
            description: value.description,
            engine: value.engine,
        })
    }

    Ok(SearchResults {
        results: search_results,
        page_query: query.to_string(),
    })
}
