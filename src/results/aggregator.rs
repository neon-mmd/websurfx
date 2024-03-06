//! This module provides the functionality to scrape and gathers all the results from the upstream
//! search engines and then removes duplicate results.

use super::user_agent::random_user_agent;
use crate::config::parser::Config;
use crate::handler::{file_path, FileType};
use crate::models::{
    aggregation_models::{EngineErrorInfo, SearchResult, SearchResults},
    engine_models::{EngineError, EngineHandler},
};
use error_stack::Report;
use futures::stream::FuturesUnordered;
use regex::Regex;
use reqwest::{Client, ClientBuilder};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs::File, io::BufRead};
use std::{
    io::{BufReader, Read},
    time::Duration,
};
use tokio::task::JoinHandle;

/// A constant for holding the prebuilt Client globally in the app.
static CLIENT: std::sync::OnceLock<Client> = std::sync::OnceLock::new();

/// Aliases for long type annotations

type FutureVec =
    FuturesUnordered<JoinHandle<Result<Vec<(String, SearchResult)>, Report<EngineError>>>>;

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
/// engines. After this, all the data in the `Vec` is removed and placed into a struct that contains all
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
    config: &Config,
    upstream_search_engines: &[EngineHandler],
    safe_search: u8,
) -> Result<SearchResults, Box<dyn std::error::Error>> {
    let client = CLIENT.get_or_init(|| {
        ClientBuilder::new()
            .timeout(Duration::from_secs(config.request_timeout as u64)) // Add timeout to request to avoid DDOSing the server
            .https_only(true)
            .gzip(true)
            .brotli(true)
            .http2_adaptive_window(config.adaptive_window)
            .build()
            .unwrap()
    });

    let user_agent: &str = random_user_agent();

    // Add a random delay before making the request.
    if config.aggregator.random_delay || !config.debug {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos() as f32;
        let delay = ((nanos / 1_0000_0000 as f32).floor() as u64) + 1;
        tokio::time::sleep(Duration::from_secs(delay)).await;
    }

    let mut names: Vec<&str> = Vec::with_capacity(0);

    // create tasks for upstream result fetching
    let tasks: FutureVec = FutureVec::new();

    for engine_handler in upstream_search_engines {
        let (name, search_engine) = engine_handler.to_owned().into_name_engine();
        names.push(name);
        let query: String = query.to_owned();
        tasks.push(tokio::spawn(async move {
            search_engine
                .results(&query, page, user_agent, client, safe_search)
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
    let mut result_map: Vec<(String, SearchResult)> = Vec::new();
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
                Ok(results) => result_map = results,
                Err(error) => handle_error(&error, engine),
            };
            continue;
        }

        match response {
            Ok(result) => {
                result.into_iter().for_each(|(key, value)| {
                    match result_map.iter().find(|(key_s, _)| key_s == &key) {
                        Some(value) => value.1.to_owned().add_engines(engine),
                        None => result_map.push((key, value)),
                    };
                });
            }
            Err(error) => handle_error(&error, engine),
        };
    }

    if safe_search >= 3 {
        let mut blacklist_map: Vec<(String, SearchResult)> = Vec::new();
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

    let results: Vec<SearchResult> = result_map.iter().map(|(_, value)| value.clone()).collect();

    Ok(SearchResults::new(results, &engine_errors_info))
}

/// Filters a map of search results using a list of regex patterns.
///
/// # Arguments
///
/// * `map_to_be_filtered` - A mutable reference to a `Vec` of search results to filter, where the filtered results will be removed from.
/// * `resultant_map` - A mutable reference to a `Vec` to hold the filtered results.
/// * `file_path` - A `&str` representing the path to a file containing regex patterns to use for filtering.
///
/// # Errors
///
/// Returns an error if the file at `file_path` cannot be opened or read, or if a regex pattern is invalid.
pub fn filter_with_lists(
    map_to_be_filtered: &mut Vec<(String, SearchResult)>,
    resultant_map: &mut Vec<(String, SearchResult)>,
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(File::open(file_path)?);

    for line in reader.by_ref().lines() {
        let re = Regex::new(line?.trim())?;

        let mut length = map_to_be_filtered.len();
        let mut idx: usize = Default::default();
        // Iterate over each search result in the map and check if it matches the regex pattern
        while idx < length {
            let ele = &map_to_be_filtered[idx];
            let ele_inner = &ele.1;
            match re.is_match(&ele.0.to_lowercase())
                || re.is_match(&ele_inner.title.to_lowercase())
                || re.is_match(&ele_inner.description.to_lowercase())
            {
                true => {
                    // If the search result matches the regex pattern, move it from the original map to the resultant map
                    resultant_map.push(map_to_be_filtered.swap_remove(idx));
                    length -= 1;
                }
                false => idx += 1,
            };
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_with_lists() -> Result<(), Box<dyn std::error::Error>> {
        // Create a map of search results to filter
        let mut map_to_be_filtered = Vec::new();
        map_to_be_filtered.push((
            "https://www.example.com".to_owned(),
            SearchResult {
                title: "Example Domain".to_owned(),
                url: "https://www.example.com".to_owned(),
                description: "This domain is for use in illustrative examples in documents."
                    .to_owned(),
                engine: smallvec!["Google".to_owned(), "Bing".to_owned()],
            },
        ));
        map_to_be_filtered.push((
            "https://www.rust-lang.org/".to_owned(),
            SearchResult {
                title: "Rust Programming Language".to_owned(),
                url: "https://www.rust-lang.org/".to_owned(),
                description: "A systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.".to_owned(),
                engine: smallvec!["Google".to_owned(), "DuckDuckGo".to_owned()],
            },)
        );

        // Create a temporary file with regex patterns
        let mut file = NamedTempFile::new()?;
        writeln!(file, "example")?;
        writeln!(file, "rust")?;
        file.flush()?;

        let mut resultant_map = Vec::new();
        filter_with_lists(
            &mut map_to_be_filtered,
            &mut resultant_map,
            file.path().to_str().unwrap(),
        )?;

        assert_eq!(resultant_map.len(), 2);
        assert!(resultant_map
            .iter()
            .any(|(key, _)| key == "https://www.example.com"));
        assert!(resultant_map
            .iter()
            .any(|(key, _)| key == "https://www.rust-lang.org/"));
        assert_eq!(map_to_be_filtered.len(), 0);

        Ok(())
    }

    #[test]
    fn test_filter_with_lists_wildcard() -> Result<(), Box<dyn std::error::Error>> {
        let mut map_to_be_filtered = Vec::new();
        map_to_be_filtered.push((
            "https://www.example.com".to_owned(),
            SearchResult {
                title: "Example Domain".to_owned(),
                url: "https://www.example.com".to_owned(),
                description: "This domain is for use in illustrative examples in documents."
                    .to_owned(),
                engine: smallvec!["Google".to_owned(), "Bing".to_owned()],
            },
        ));
        map_to_be_filtered.push((
            "https://www.rust-lang.org/".to_owned(),
            SearchResult {
                title: "Rust Programming Language".to_owned(),
                url: "https://www.rust-lang.org/".to_owned(),
                description: "A systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.".to_owned(),
                engine: smallvec!["Google".to_owned(), "DuckDuckGo".to_owned()],
            },
        ));

        // Create a temporary file with a regex pattern containing a wildcard
        let mut file = NamedTempFile::new()?;
        writeln!(file, "ex.*le")?;
        file.flush()?;

        let mut resultant_map = Vec::new();

        filter_with_lists(
            &mut map_to_be_filtered,
            &mut resultant_map,
            file.path().to_str().unwrap(),
        )?;

        assert_eq!(resultant_map.len(), 1);
        assert!(resultant_map
            .iter()
            .any(|(key, _)| key == "https://www.example.com"));
        assert_eq!(map_to_be_filtered.len(), 1);
        assert!(map_to_be_filtered
            .iter()
            .any(|(key, _)| key == "https://www.rust-lang.org/"));

        Ok(())
    }

    #[test]
    fn test_filter_with_lists_file_not_found() {
        let mut map_to_be_filtered = Vec::new();

        let mut resultant_map = Vec::new();

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
        let mut map_to_be_filtered = Vec::new();
        map_to_be_filtered.push((
            "https://www.example.com".to_owned(),
            SearchResult {
                title: "Example Domain".to_owned(),
                url: "https://www.example.com".to_owned(),
                description: "This domain is for use in illustrative examples in documents."
                    .to_owned(),
                engine: smallvec!["Google".to_owned(), "Bing".to_owned()],
            },
        ));

        let mut resultant_map = Vec::new();

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
