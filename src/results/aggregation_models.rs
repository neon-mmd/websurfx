//! This module provides public models for handling, storing and serializing of search results
//! data scraped from the upstream search engines.

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{config::parser_models::Style, engines::engine_models::EngineError};

/// A named struct to store the raw scraped search results scraped search results from the
/// upstream search engines before aggregating it.It derives the Clone trait which is needed
/// to write idiomatic rust using `Iterators`.
///
/// # Fields
///
/// * `title` - The title of the search result.
/// * `url` - The url which is accessed when clicked on it
/// (href url in html in simple words).
/// * `description` - The description of the search result.
/// * `engine` - The names of the upstream engines from which this results were provided.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub description: String,
    pub engine: SmallVec<[String; 0]>,
}

impl SearchResult {
    /// Constructs a new `RawSearchResult` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the search result.
    /// * `url` - The url which is accessed when clicked on it
    /// (href url in html in simple words).
    /// * `description` - The description of the search result.
    /// * `engine` - The names of the upstream engines from which this results were provided.
    pub fn new(title: &str, url: &str, description: &str, engine: &[&str]) -> Self {
        SearchResult {
            title: title.to_owned(),
            url: url.to_owned(),
            description: description.to_owned(),
            engine: engine.iter().map(|name| name.to_string()).collect(),
        }
    }

    /// A function which adds the engine name provided as a string into a vector of strings.
    ///
    /// # Arguments
    ///
    /// * `engine` - Takes an engine name provided as a String.
    pub fn add_engines(&mut self, engine: &str) {
        self.engine.push(engine.to_owned())
    }

    /// A function which returns the engine name stored from the struct as a string.
    ///
    /// # Returns
    ///
    /// An engine name stored as a string from the struct.
    pub fn engine(&mut self) -> String {
        std::mem::take(&mut self.engine[0])
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EngineErrorInfo {
    pub error: String,
    pub engine: String,
    pub severity_color: String,
}

impl EngineErrorInfo {
    pub fn new(error: &EngineError, engine: &str) -> Self {
        Self {
            error: match error {
                EngineError::RequestError => "RequestError".to_owned(),
                EngineError::EmptyResultSet => "EmptyResultSet".to_owned(),
                EngineError::UnexpectedError => "UnexpectedError".to_owned(),
            },
            engine: engine.to_owned(),
            severity_color: match error {
                EngineError::RequestError => "green".to_owned(),
                EngineError::EmptyResultSet => "blue".to_owned(),
                EngineError::UnexpectedError => "red".to_owned(),
            },
        }
    }
}

/// A named struct to store, serialize, deserialize the all the search results scraped and
/// aggregated from the upstream search engines.
///
/// # Fields
///
/// * `results` - Stores the individual serializable `SearchResult` struct into a vector of
/// `SearchResult` structs.
/// * `page_query` - Stores the current pages search query `q` provided in the search url.
/// * `style` - Stores the theming options for the website.
/// * `engine_errors_info` - Stores the information on which engines failed with their engine name
/// and the type of error that caused it.
/// * `empty_result_set` - Stores a boolean which indicates that no engines gave a result for the
/// given search query.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub page_query: String,
    pub style: Style,
    pub engine_errors_info: SmallVec<[EngineErrorInfo; 0]>,
}

impl SearchResults {
    /// Constructs a new `SearchResult` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `results` - Takes an argument of individual serializable `SearchResult` struct
    /// and stores it into a vector of `SearchResult` structs.
    /// * `page_query` - Takes an argument of current page`s search query `q` provided in
    /// the search url.
    /// * `empty_result_set` - Takes a boolean which indicates that no engines gave a result for the
    /// given search query.
    pub fn new(
        results: Vec<SearchResult>,
        page_query: &str,
        engine_errors_info: &[EngineErrorInfo],
    ) -> Self {
        Self {
            results,
            page_query: page_query.to_owned(),
            style: Style::default(),
            engine_errors_info: SmallVec::from(engine_errors_info),
        }
    }

    /// A setter function to add website style to the return search results.
    pub fn add_style(&mut self, style: &Style) {
        self.style = style.to_owned();
    }
}
