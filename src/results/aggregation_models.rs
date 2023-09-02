//! This module provides public models for handling, storing and serializing of search results
//! data scraped from the upstream search engines.

use serde::{Deserialize, Serialize};

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
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub description: String,
    pub engine: Vec<String>,
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
    pub fn new(title: String, url: String, description: String, engine: Vec<String>) -> Self {
        SearchResult {
            title,
            url,
            description,
            engine,
        }
    }

    /// A function which adds the engine name provided as a string into a vector of strings.
    ///
    /// # Arguments
    ///
    /// * `engine` - Takes an engine name provided as a String.
    pub fn add_engines(&mut self, engine: String) {
        self.engine.push(engine)
    }

    /// A function which returns the engine name stored from the struct as a string.
    ///
    /// # Returns
    ///
    /// An engine name stored as a string from the struct.
    pub fn engine(self) -> String {
        self.engine.get(0).unwrap().to_string()
    }
}

///
#[derive(Serialize, Deserialize)]
pub struct EngineErrorInfo {
    pub error: String,
    pub engine: String,
    pub severity_color: String,
}

impl EngineErrorInfo {
    pub fn new(error: &EngineError, engine: String) -> Self {
        Self {
            error: match error {
                EngineError::RequestError => String::from("RequestError"),
                EngineError::EmptyResultSet => String::from("EmptyResultSet"),
                EngineError::UnexpectedError => String::from("UnexpectedError"),
            },
            engine,
            severity_color: match error {
                EngineError::RequestError => String::from("green"),
                EngineError::EmptyResultSet => String::from("blue"),
                EngineError::UnexpectedError => String::from("red"),
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
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub page_query: String,
    pub style: Style,
    pub engine_errors_info: Vec<EngineErrorInfo>,
    pub disallowed: bool,
    pub filtered: bool,
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
    /// * ``
    pub fn new(
        results: Vec<SearchResult>,
        page_query: String,
        engine_errors_info: Vec<EngineErrorInfo>,
    ) -> Self {
        SearchResults {
            results,
            page_query,
            style: Style::default(),
            engine_errors_info,
            disallowed: Default::default(),
            filtered: Default::default(),
        }
    }

    /// A setter function to add website style to the return search results.
    pub fn add_style(&mut self, style: &Style) {
        self.style = style.clone();
    }

    /// A setter function that sets disallowed to true.
    pub fn set_disallowed(&mut self) {
        self.disallowed = true;
    }

    /// A setter function to set the current page search query.
    pub fn set_page_query(&mut self, page: &str) {
        self.page_query = page.to_owned();
    }

    /// A setter function that sets the filtered to true.
    pub fn set_filtered(&mut self) {
        self.filtered = true;
    }

    /// A getter function that gets the value of `engine_errors_info`.
    pub fn engine_errors_info(&mut self) -> Vec<EngineErrorInfo> {
        std::mem::take(&mut self.engine_errors_info)
    }
    /// A getter function that gets the value of `results`.
    pub fn results(&mut self) -> Vec<SearchResult> {
        self.results.clone()
    }
}
