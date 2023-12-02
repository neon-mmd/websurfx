//! This module provides the functionality to scrape and gathers all the results from the upstream
//! search engines and then removes duplicate results.

use crate::engine::RawResults;
use crate::handler::{file_path, FileType};
use crate::models::{
    aggregation_models::{EngineErrorInfo, SearchResult, SearchResults},
    engine_models::EngineError,
};
use error_stack::Report;
use regex::Regex;
use reqwest::Client;

use std::{
    collections::HashMap,
    io::{BufReader, Read},
};
use std::{fs::File, io::BufRead};
use tokio::task::JoinHandle;

pub struct Ranker;

impl Ranker {
    // The function preprocesses the scraped results from the user-selected upstream search engines.
    ///
    /// Additionally, the function eliminates duplicate results. If two results are identified as coming from
    /// multiple engines, their names are combined to indicate that the results were fetched from these upstream
    /// engines. After this, all the data in the `HashMap` is removed and placed into a struct that contains all
    /// the aggregated results in a vector. Furthermore, the query used is also added to the struct. This step is
    /// necessary to ensure that the search bar in the search remains populated even when searched from the query URL.
    /// # Error
    ///
    /// Returns an error a reqwest and scraping selector errors if any error occurs in the results
    /// function in either `searx` or `duckduckgo` or both otherwise returns a `SearchResults struct`
    /// containing appropriate values.
    fn preprocess(
        &self,
        responses: RawResults,
        safe_search: u8,
    ) -> Result<(Vec<SearchResult>, Vec<EngineErrorInfo>), Box<dyn std::error::Error>> {
        let mut result_map: HashMap<String, SearchResult> = HashMap::new();
        let mut engine_errors_info: Vec<EngineErrorInfo> = Vec::new();

        let mut handle_error = |error: Report<EngineError>| {
            log::error!("Engine Error: {:?}", error);
            let error = error.downcast_ref::<EngineError>().unwrap();
            engine_errors_info.push(EngineErrorInfo::new(&error.error_type, &error.engine))
        };

        for response in responses {
            if result_map.is_empty() {
                match response {
                    Ok(results) => {
                        result_map = results.clone();
                    }
                    Err(error) => {
                        handle_error(error);
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
                                result.add_engines(&value.engine[0]);
                            })
                            .or_insert_with(|| -> SearchResult { value });
                    });
                }
                Err(error) => {
                    handle_error(error);
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

        Ok((result_map.into_values().collect(), engine_errors_info))
    }

    pub fn process(
        &self,
        responses: RawResults,
        safe_search: u8,
    ) -> Result<SearchResults, Box<dyn std::error::Error>> {
        let (results, engine_errors_info) = self.preprocess(responses, safe_search)?;

        Ok(SearchResults::new(results, &engine_errors_info))
    }
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
    use crate::models::aggregation_models::ResultType;
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
                page_url: "https://www.example.com".to_owned(),
                description: "This domain is for use in illustrative examples in documents."
                    .to_owned(),
                engine: smallvec!["Google".to_owned(), "Bing".to_owned()],
            },
        );
        map_to_be_filtered.insert(
            "https://www.rust-lang.org/".to_owned(),
            SearchResult {
                title: "Rust Programming Language".to_owned(),
                page_url: "https://www.rust-lang.org/".to_owned(),
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
                page_url: "https://www.example.com".to_owned(),
                description: "This domain is for use in illustrative examples in documents."
                    .to_owned(),
                engine: smallvec!["Google".to_owned(), "Bing".to_owned()],
            },
        );
        map_to_be_filtered.insert(
            "https://www.rust-lang.org/".to_owned(),
            SearchResult {
                title: "Rust Programming Language".to_owned(),
                page_url: "https://www.rust-lang.org/".to_owned(),
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
                page_url: "https://www.example.com".to_owned(),
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
