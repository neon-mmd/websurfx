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
use std::sync::Arc;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    task::JoinHandle,
    time::Duration,
};

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
///   user through the UI or the config file.
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
        let mut cb = ClientBuilder::new()
            .timeout(Duration::from_secs(config.request_timeout as u64)) // Add timeout to request to avoid DDOSing the server
            .pool_idle_timeout(Duration::from_secs(
                config.pool_idle_connection_timeout as u64,
            ))
            .tcp_keepalive(Duration::from_secs(config.tcp_connection_keep_alive as u64))
            .pool_max_idle_per_host(config.number_of_https_connections as usize)
            .connect_timeout(Duration::from_secs(config.request_timeout as u64)) // Add timeout to request to avoid DDOSing the server
            .use_rustls_tls()
            .tls_built_in_root_certs(config.operating_system_tls_certificates)
            .https_only(true)
            .gzip(true)
            .brotli(true)
            .http2_adaptive_window(config.adaptive_window);

        if config.proxy.is_some() {
            cb = cb.proxy(config.proxy.clone().unwrap());
        }

        cb.build().unwrap()
    });

    let user_agent: &str = random_user_agent();

    let mut names: Vec<&str> = Vec::with_capacity(0);

    // create tasks for upstream result fetching
    let tasks: FutureVec = FutureVec::new();

    let query: Arc<String> = Arc::new(query.to_string());
    for engine_handler in upstream_search_engines {
        let (name, search_engine) = engine_handler.clone().into_name_engine();
        names.push(name);
        let query_partially_cloned = query.clone();
        tasks.push(tokio::spawn(async move {
            search_engine
                .results(
                    &query_partially_cloned,
                    page,
                    user_agent,
                    client,
                    safe_search,
                )
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
        )
        .await?;

        filter_with_lists(
            &mut blacklist_map,
            &mut result_map,
            file_path(FileType::AllowList)?,
        )
        .await?;

        drop(blacklist_map);
    }

    let mut results: Box<[SearchResult]> = result_map
        .into_iter()
        .map(|(_, mut value)| {
            if !value.url.contains("temu.com") {
                value.calculate_relevance(query.as_str())
            }
            value
        })
        .collect();
    sort_search_results(&mut results);

    Ok(SearchResults::new(
        results,
        engine_errors_info.into_boxed_slice(),
    ))
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
pub async fn filter_with_lists(
    map_to_be_filtered: &mut Vec<(String, SearchResult)>,
    resultant_map: &mut Vec<(String, SearchResult)>,
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let reader = BufReader::new(File::open(file_path).await?);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        let re = Regex::new(line.trim())?;

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

/// Sorts  SearchResults by relevance score.
/// <br> sort_unstable is used as its faster,stability is not an issue on our side.
/// For reasons why, check out [`this`](https://rust-lang.github.io/rfcs/1884-unstable-sort.html)
///  # Arguments
///  * `results` - A mutable slice or Vec of SearchResults
///  
fn sort_search_results(results: &mut [SearchResult]) {
    results.sort_unstable_by(|a, b| {
        use std::cmp::Ordering;

        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(Ordering::Less)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_filter_with_lists() -> Result<(), Box<dyn std::error::Error>> {
        // Create a map of search results to filter
        let mut map_to_be_filtered = Vec::new();
        map_to_be_filtered.push((
            "https://www.example.com".to_owned(),
            SearchResult {
                title: "Example Domain".to_owned(),
                url: "https://www.example.com".to_owned(),
                description: "This domain is for use in illustrative examples in documents."
                    .to_owned(),
                relevance_score: 0.0,
                engine: vec!["Google".to_owned(), "Bing".to_owned()],
            },
        ));
        map_to_be_filtered.push((
            "https://www.rust-lang.org/".to_owned(),
            SearchResult {
                title: "Rust Programming Language".to_owned(),
                url: "https://www.rust-lang.org/".to_owned(),
                description: "A systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.".to_owned(),
                engine: vec!["Google".to_owned(), "DuckDuckGo".to_owned()],
                relevance_score:0.0
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
        )
        .await?;

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

    #[tokio::test]
    async fn test_filter_with_lists_wildcard() -> Result<(), Box<dyn std::error::Error>> {
        let mut map_to_be_filtered = Vec::new();
        map_to_be_filtered.push((
            "https://www.example.com".to_owned(),
            SearchResult {
                title: "Example Domain".to_owned(),
                url: "https://www.example.com".to_owned(),
                description: "This domain is for use in illustrative examples in documents."
                    .to_owned(),
                engine: vec!["Google".to_owned(), "Bing".to_owned()],
                relevance_score: 0.0,
            },
        ));
        map_to_be_filtered.push((
            "https://www.rust-lang.org/".to_owned(),
            SearchResult {
                title: "Rust Programming Language".to_owned(),
                url: "https://www.rust-lang.org/".to_owned(),
                description: "A systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.".to_owned(),
                engine: vec!["Google".to_owned(), "DuckDuckGo".to_owned()],
                relevance_score:0.0
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
        )
        .await?;

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

    #[tokio::test]
    async fn test_filter_with_lists_file_not_found() {
        let mut map_to_be_filtered = Vec::new();

        let mut resultant_map = Vec::new();

        // Call the `filter_with_lists` function with a non-existent file path
        let result = filter_with_lists(
            &mut map_to_be_filtered,
            &mut resultant_map,
            "non-existent-file.txt",
        );

        assert!(result.await.is_err());
    }

    #[tokio::test]
    async fn test_filter_with_lists_invalid_regex() {
        let mut map_to_be_filtered = Vec::new();
        map_to_be_filtered.push((
            "https://www.example.com".to_owned(),
            SearchResult {
                title: "Example Domain".to_owned(),
                url: "https://www.example.com".to_owned(),
                description: "This domain is for use in illustrative examples in documents."
                    .to_owned(),
                engine: vec!["Google".to_owned(), "Bing".to_owned()],
                relevance_score: 0.0,
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

        assert!(result.await.is_err());
    }
}
