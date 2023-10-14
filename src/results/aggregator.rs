//! This module provides the functionality to scrape and gathers all the results from the upstream
//! search engines and then removes duplicate results.

use super::user_agent::random_user_agent;
use crate::handler::paths::{file_path, FileType};
use crate::models::{
    aggregation_models::{EngineErrorInfo, SearchResult, SearchResults},
    engine_models::{EngineError, EngineHandler},
};
use error_stack::Report;
use rand::Rng;
use regex::Regex;
use std::{
    collections::HashMap,
    io::{BufReader, Read},
    time::Duration,
};
use std::{fs::File, io::BufRead};
use tokio::task::JoinHandle;

/// Aliases for long type annotations
type FutureVec = Vec<JoinHandle<Result<HashMap<String, SearchResult>, Report<EngineError>>>>;

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
    query: &str,
    page: u32,
    random_delay: bool,
    debug: bool,
    upstream_search_engines: &[EngineHandler],
    request_timeout: u8,
    safe_search: u8,
) -> Result<SearchResults, Box<dyn std::error::Error>> {
    let user_agent: &str = random_user_agent();

    // Add a random delay before making the request.
    if random_delay || !debug {
        let mut rng = rand::thread_rng();
        let delay_secs = rng.gen_range(1..10);
        tokio::time::sleep(Duration::from_secs(delay_secs)).await;
    }

    let mut names: Vec<&str> = Vec::with_capacity(0);

    // create tasks for upstream result fetching
    let mut tasks: FutureVec = FutureVec::new();

    for engine_handler in upstream_search_engines {
        let (name, search_engine) = engine_handler.to_owned().into_name_engine();
        names.push(name);
        let query: String = query.to_owned();
        tasks.push(tokio::spawn(async move {
            search_engine
                .results(&query, page, user_agent, request_timeout, safe_search)
                .await
        }));
    }

    // get upstream responses
    let mut responses = Vec::with_capacity(tasks.len());

    for task in tasks {
        if let Ok(result) = task.await {
            responses.push(result)
        }
    }

    // aggregate search results, removing duplicates and handling errors the upstream engines returned
    let mut result_map: HashMap<String, SearchResult> = HashMap::new();
    let mut engine_errors_info: Vec<EngineErrorInfo> = Vec::new();

    let mut handle_error = |error: &Report<EngineError>, engine_name: &'static str| {
        log::error!("Engine Error: {:?}", error);
        engine_errors_info.push(EngineErrorInfo::new(
            error.downcast_ref::<EngineError>().unwrap(),
            engine_name,
        ));
    };

    for _ in 0..responses.len() {
        let response = responses.pop().unwrap();
        let engine = names.pop().unwrap();

        if result_map.is_empty() {
            match response {
                Ok(results) => {
                    result_map = results.clone();
                }
                Err(error) => {
                    handle_error(&error, engine);
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
                            result.add_engines(engine);
                        })
                        .or_insert_with(|| -> SearchResult { value });
                });
            }
            Err(error) => {
                handle_error(&error, engine);
            }
        }
    }

    if safe_search >= 3 {
        let mut blacklist_map: HashMap<String, SearchResult> = HashMap::new();
        filter_with_lists(
            &mut result_map,
            &mut blacklist_map,
            file_path(FileType::BlockList)?,
        )?;

        filter_with_lists(
            &mut blacklist_map,
            &mut result_map,
            file_path(FileType::AllowList)?,
        )?;

        drop(blacklist_map);
    }

    let results: Vec<SearchResult> = result_map.into_values().collect();

    Ok(SearchResults::new(results, query, &engine_errors_info))
}

/// Filters a map of search results using a list of regex patterns.
///
/// # Arguments
///
/// * `map_to_be_filtered` - A mutable reference to a `HashMap` of search results to filter, where the filtered results will be removed from.
/// * `resultant_map` - A mutable reference to a `HashMap` to hold the filtered results.
/// * `file_path` - A `&str` representing the path to a file containing regex patterns to use for filtering.
///
/// # Errors
///
/// Returns an error if the file at `file_path` cannot be opened or read, or if a regex pattern is invalid.
pub fn filter_with_lists(
    map_to_be_filtered: &mut HashMap<String, SearchResult>,
    resultant_map: &mut HashMap<String, SearchResult>,
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(File::open(file_path)?);

    for line in reader.by_ref().lines() {
        let re = Regex::new(line?.trim())?;

        // Iterate over each search result in the map and check if it matches the regex pattern
        for (url, search_result) in map_to_be_filtered.clone().into_iter() {
            if re.is_match(&url.to_lowercase())
                || re.is_match(&search_result.title.to_lowercase())
                || re.is_match(&search_result.description.to_lowercase())
            {
                // If the search result matches the regex pattern, move it from the original map to the resultant map
                resultant_map.insert(
                    url.to_owned(),
                    map_to_be_filtered.remove(&url.to_owned()).unwrap(),
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_with_lists() -> Result<(), Box<dyn std::error::Error>> {
        // Create a map of search results to filter
        let mut map_to_be_filtered = HashMap::new();
        map_to_be_filtered.insert(
            "https://www.example.com".to_owned(),
            SearchResult {
                title: "Example Domain".to_owned(),
                url: "https://www.example.com".to_owned(),
                description: "This domain is for use in illustrative examples in documents."
                    .to_owned(),
                engine: smallvec!["Google".to_owned(), "Bing".to_owned()],
            },
        );
        map_to_be_filtered.insert(
            "https://www.rust-lang.org/".to_owned(),
            SearchResult {
                title: "Rust Programming Language".to_owned(),
                url: "https://www.rust-lang.org/".to_owned(),
                description: "A systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.".to_owned(),
                engine: smallvec!["Google".to_owned(), "DuckDuckGo".to_owned()],
            },
        );

        // Create a temporary file with regex patterns
        let mut file = NamedTempFile::new()?;
        writeln!(file, "example")?;
        writeln!(file, "rust")?;
        file.flush()?;

        let mut resultant_map = HashMap::new();
        filter_with_lists(
            &mut map_to_be_filtered,
            &mut resultant_map,
            file.path().to_str().unwrap(),
        )?;

        assert_eq!(resultant_map.len(), 2);
        assert!(resultant_map.contains_key("https://www.example.com"));
        assert!(resultant_map.contains_key("https://www.rust-lang.org/"));
        assert_eq!(map_to_be_filtered.len(), 0);

        Ok(())
    }

    #[test]
    fn test_filter_with_lists_wildcard() -> Result<(), Box<dyn std::error::Error>> {
        let mut map_to_be_filtered = HashMap::new();
        map_to_be_filtered.insert(
            "https://www.example.com".to_owned(),
            SearchResult {
                title: "Example Domain".to_owned(),
                url: "https://www.example.com".to_owned(),
                description: "This domain is for use in illustrative examples in documents."
                    .to_owned(),
                engine: smallvec!["Google".to_owned(), "Bing".to_owned()],
            },
        );
        map_to_be_filtered.insert(
            "https://www.rust-lang.org/".to_owned(),
            SearchResult {
                title: "Rust Programming Language".to_owned(),
                url: "https://www.rust-lang.org/".to_owned(),
                description: "A systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.".to_owned(),
                engine: smallvec!["Google".to_owned(), "DuckDuckGo".to_owned()],
            },
        );

        // Create a temporary file with a regex pattern containing a wildcard
        let mut file = NamedTempFile::new()?;
        writeln!(file, "ex.*le")?;
        file.flush()?;

        let mut resultant_map = HashMap::new();

        filter_with_lists(
            &mut map_to_be_filtered,
            &mut resultant_map,
            file.path().to_str().unwrap(),
        )?;

        assert_eq!(resultant_map.len(), 1);
        assert!(resultant_map.contains_key("https://www.example.com"));
        assert_eq!(map_to_be_filtered.len(), 1);
        assert!(map_to_be_filtered.contains_key("https://www.rust-lang.org/"));

        Ok(())
    }

    #[test]
    fn test_filter_with_lists_file_not_found() {
        let mut map_to_be_filtered = HashMap::new();

        let mut resultant_map = HashMap::new();

        // Call the `filter_with_lists` function with a non-existent file path
        let result = filter_with_lists(
            &mut map_to_be_filtered,
            &mut resultant_map,
            "non-existent-file.txt",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_filter_with_lists_invalid_regex() {
        let mut map_to_be_filtered = HashMap::new();
        map_to_be_filtered.insert(
            "https://www.example.com".to_owned(),
            SearchResult {
                title: "Example Domain".to_owned(),
                url: "https://www.example.com".to_owned(),
                description: "This domain is for use in illustrative examples in documents."
                    .to_owned(),
                engine: smallvec!["Google".to_owned(), "Bing".to_owned()],
            },
        );

        let mut resultant_map = HashMap::new();

        // Create a temporary file with an invalid regex pattern
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "example(").unwrap();
        file.flush().unwrap();

        let result = filter_with_lists(
            &mut map_to_be_filtered,
            &mut resultant_map,
            file.path().to_str().unwrap(),
        );

        assert!(result.is_err());
    }
}
