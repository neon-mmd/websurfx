//! This module provides the functionality to scrape and gathers all the results from the upstream
//! search engines and then removes duplicate results.

use crate::handler::{FileType, file_path};
use crate::models::{
    aggregation::{EngineErrorInfo, SearchResult, SearchResults},
    engine::{EngineError, EngineHandler},
};
use crate::parser::Config;

use error_stack::Report;
use rayon::prelude::*;
use regex::Regex;
use reqwest::{Client, ClientBuilder};
use std::sync::{Arc, OnceLock};
use tokio::task::JoinSet;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    time::Duration,
};

/// A constant for holding the prebuilt Client globally in the app.
static CLIENT: std::sync::OnceLock<Client> = std::sync::OnceLock::new();

/// The language an upstream is told the browser prefers.
///
/// Sent on every request so that a request never goes out with a missing `Accept-Language`,
/// which on its own is a strong signal that the caller is not a browser.
const ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9";

/// The `Accept` sent to the engines that answer with a page of HTML.
const ACCEPT_HTML: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";

/// Cached compiled regex patterns for blocklist/allowlist filtering.
/// Uses ahash for faster hashing.
pub static FILTER_PATTERNS_CACHE: OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, Vec<Regex>, ahash::RandomState>>,
> = OnceLock::new();

/// Aliases for long type annotations
type EngineTaskOutput = (
    &'static str,
    Result<Vec<(String, SearchResult)>, Report<EngineError>>,
);
/// A JoinSet of engine fetch tasks, each tagged with its engine name so results
/// and errors can be attributed correctly.
type FutureVec = JoinSet<EngineTaskOutput>;

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
    upstream_search_engines: &[&'static EngineHandler],
    safe_search: u8,
    user_agent: &'static str,
) -> Result<SearchResults, Box<dyn std::error::Error>> {
    let client = CLIENT.get_or_init(|| {
        // The headers every upstream expects to see from a browser, so that no engine can
        // forget one and no request goes out bare. They live on the client rather than on
        // each request, which is what makes them apply to the first attempt and to every
        // retry alike. An engine still overrides any of them per request, and adds whatever
        // else that particular site needs.
        let mut spoofed = reqwest::header::HeaderMap::with_capacity(7);
        spoofed.insert(
            reqwest::header::ACCEPT_LANGUAGE,
            reqwest::header::HeaderValue::from_static(ACCEPT_LANGUAGE),
        );
        spoofed.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static(ACCEPT_HTML),
        );
        for (name, value) in [
            ("sec-fetch-dest", "document"),
            ("sec-fetch-mode", "navigate"),
            ("sec-fetch-site", "same-origin"),
            ("sec-fetch-user", "?1"),
            ("upgrade-insecure-requests", "1"),
        ] {
            let name = reqwest::header::HeaderName::from_static(name);
            let value = reqwest::header::HeaderValue::from_static(value);

            spoofed.insert(name, value);
        }

        let mut cb = ClientBuilder::new()
            .timeout(Duration::from_secs(config.request_timeout as u64))
            .pool_idle_timeout(Duration::from_secs(
                config.pool_idle_connection_timeout as u64,
            ))
            .tcp_keepalive(Duration::from_secs(config.tcp_connection_keep_alive as u64))
            .pool_max_idle_per_host(config.number_of_https_connections as usize)
            .connect_timeout(Duration::from_secs(config.request_timeout as u64))
            .use_rustls_tls()
            // `reqwest` 0.13 removed `ClientBuilder::tls_built_in_root_certs` in favour of
            // `tls_certs_only`/`tls_certs_merge`. The rustls backend that `use_rustls_tls`
            // selects now resolves the operating system certificate store through its platform
            // verifier by default, which is exactly what the `operating_system_tls_certificates`
            // configuration asks for, so the dropped builder knob no longer needs to be set here.
            .https_only(true)
            .gzip(true)
            .brotli(true)
            .http2_adaptive_window(config.adaptive_window)
            // HTTP/2 is negotiated per host through ALPN, which is why neither
            // `http1_only` nor `http2_prior_knowledge` is set. Prior knowledge offers `h2`
            // and nothing else, so any upstream that answers ALPN with `http/1.1` rejects
            // the connection outright with a `no_application_protocol` alert. That is
            // exactly what mojeek does, and it also left yahoo dropping connections
            // mid-session. Leaving the fallback in place lets each engine use the version
            // it actually speaks.
            // Enable connection reuse across requests
            .pool_max_idle_per_host(config.number_of_https_connections as usize)
            // Set a reasonable user agent for connection identification
            .user_agent(user_agent)
            .default_headers(spoofed);

        if config.proxy.is_some() {
            cb = cb.proxy(config.proxy.clone().unwrap());
        }

        cb.build().unwrap()
    });

    // create tasks for upstream result fetching
    let mut tasks: FutureVec = JoinSet::new();

    let query: Arc<String> = Arc::new(query.to_string());
    for engine_handler in upstream_search_engines {
        // The engine is shared, so handing it to the task is a refcount bump rather than a
        // rebuild of its compiled selectors.
        let (name, search_engine) = engine_handler.name_engine();
        let query_partially_cloned = query.clone();
        tasks.spawn(async move {
            let result = search_engine
                .results(
                    &query_partially_cloned,
                    page,
                    user_agent,
                    client,
                    safe_search,
                )
                .await;
            (name, result)
        });
    }

    // aggregate search results, removing duplicates and handling errors the upstream engines returned
    // Use a vector for memory efficiency; on cache miss below we do one linear
    // scan per result which is O(n²) worst-case but n is small (dozens of results).
    let mut result_map: Vec<(String, SearchResult)> = Vec::new();
    let mut engine_errors_info: Vec<EngineErrorInfo> = Vec::new();

    while let Some(Ok((engine, response))) = tasks.join_next().await {
        match response {
            Ok(result) => {
                for (key, value) in result {
                    if let Some(existing) = result_map
                        .iter_mut()
                        .find(|(key_s, _)| key_s == &key)
                        .map(|(_, v)| v)
                    {
                        // The same URL came from multiple upstream engines; merge the
                        // engine names so attribution is preserved. (Previously the
                        // merged value was dropped on the floor.)
                        if !existing.engine.iter().any(|e| e == engine) {
                            existing.add_engines(engine);
                        }
                    } else {
                        result_map.push((key, value))
                    }
                }
            }
            Err(error) => {
                log::error!("Engine Error: {:?}", error);
                engine_errors_info.push(EngineErrorInfo::new(
                    error.downcast_ref::<EngineError>().unwrap(),
                    engine,
                ));
            }
        }
    }

    if safe_search >= 3 {
        let mut blacklist_map: Vec<(String, SearchResult)> = Vec::new();
        filter_with_lists(
            &mut result_map,
            &mut blacklist_map,
            &file_path(FileType::BlockList).await?,
        )
        .await?;

        filter_with_lists(
            &mut blacklist_map,
            &mut result_map,
            &file_path(FileType::AllowList).await?,
        )
        .await?;

        drop(blacklist_map);
    }

    let results: Box<[SearchResult]> = tokio::task::spawn_blocking(move || {
        let mut values: Vec<SearchResult> =
            result_map.into_iter().map(|(_, value)| value).collect();

        // Batch compute relevance scores using optimized TF-IDF calculator
        let calculator = crate::models::aggregation::TfIdfCalculator::new();
        calculator.compute_scores(query.as_str(), &mut values);

        let mut unsorted_results: Box<[SearchResult]> = values.into_boxed_slice();

        sort_search_results(&mut unsorted_results);

        unsorted_results
    })
    .await?;

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
    // Get or compile patterns from cache (with mutex for thread-safe initialization)
    let cache = FILTER_PATTERNS_CACHE.get_or_init(|| {
        std::sync::Mutex::new(std::collections::HashMap::with_hasher(
            ahash::RandomState::new(),
        ))
    });

    // Try to get cached patterns first (minimal lock time)
    let patterns = {
        let cache_guard = cache.lock().unwrap();
        if let Some(cached) = cache_guard.get(file_path) {
            cached.clone()
        } else {
            Vec::new() // Will compile below
        }
    };

    // If not in cache, compile patterns (without holding the lock during I/O)
    let patterns = if patterns.is_empty() {
        let reader = BufReader::new(File::open(file_path).await?);
        let mut lines = reader.lines();
        let mut compiled: Vec<Regex> = Vec::new();
        while let Some(line) = lines.next_line().await? {
            compiled.push(Regex::new(line.trim())?);
        }
        // Store in cache (minimal lock time)
        {
            let mut cache_guard = cache.lock().unwrap();
            cache_guard.insert(file_path.to_string(), compiled.clone());
        }
        compiled
    } else {
        patterns
    };

    let mut idx: usize = 0;
    while idx < map_to_be_filtered.len() {
        let (key, value) = &map_to_be_filtered[idx];
        let key_lower = key.to_lowercase();
        let title_lower = value.title.to_lowercase();
        let description_lower = value.description.to_lowercase();

        let matches = patterns.iter().any(|re| {
            re.is_match(&key_lower) || re.is_match(&title_lower) || re.is_match(&description_lower)
        });

        if matches {
            resultant_map.push(map_to_be_filtered.swap_remove(idx));
        } else {
            idx += 1;
        }
    }

    Ok(())
}

/// Sorts  SearchResults by relevance score.
/// <br> sort_unstable is used as its faster,stability is not an issue on our side.
/// For reasons why, check out [`this`](https://rust-lang.github.io/rfcs/1884-unstable-sort.html)
///
///  # Arguments
///
///  * `results` - A mutable slice or Vec of SearchResults
fn sort_search_results(results: &mut [SearchResult]) {
    results.par_sort_unstable_by(|a, b| {
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
        assert!(
            resultant_map
                .iter()
                .any(|(key, _)| key == "https://www.example.com")
        );
        assert!(
            resultant_map
                .iter()
                .any(|(key, _)| key == "https://www.rust-lang.org/")
        );
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
        assert!(
            resultant_map
                .iter()
                .any(|(key, _)| key == "https://www.example.com")
        );
        assert_eq!(map_to_be_filtered.len(), 1);
        assert!(
            map_to_be_filtered
                .iter()
                .any(|(key, _)| key == "https://www.rust-lang.org/")
        );

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
