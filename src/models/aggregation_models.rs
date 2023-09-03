//! This module provides public models for handling, storing and serializing of search results
//! data scraped from the upstream search engines.

use serde::{Deserialize, Serialize};

use super::{engine_models::EngineError, parser_models::Style};

/// A named struct to store the raw scraped search results scraped search results from the
/// upstream search engines before aggregating it.It derives the Clone trait which is needed
/// to write idiomatic rust using `Iterators`.
/// (href url in html in simple words).
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// The title of the search result.
    pub title: String,
    /// The url which is accessed when clicked on it
    pub url: String,
    /// The description of the search result.
    pub description: String,
    /// The names of the upstream engines from which this results were provided.
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

/// A named struct that stores the error info related to the upstream search engines.
#[derive(Serialize, Deserialize)]
pub struct EngineErrorInfo {
    /// It stores the error type which occured while fetching the result from a particular search
    /// engine.
    pub error: String,
    /// It stores the name of the engine that failed to provide the requested search results.
    pub engine: String,
    /// It stores the name of the color to indicate whether how severe the particular error is (In
    /// other words it indicates the severity of the error/issue).
    pub severity_color: String,
}

impl EngineErrorInfo {
    /// Constructs a new `SearchResult` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `error` - It takes the error type which occured while fetching the result from a particular
    /// search engine.
    /// * `engine` - It takes the name of the engine that failed to provide the requested search results.
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
/// `SearchResult` structs.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    /// Stores the individual serializable `SearchResult` struct into a vector of
    pub results: Vec<SearchResult>,
    /// Stores the current pages search query `q` provided in the search url.
    pub page_query: String,
    /// Stores the theming options for the website.
    pub style: Style,
    /// Stores the information on which engines failed with their engine name
    /// and the type of error that caused it.
    pub engine_errors_info: Vec<EngineErrorInfo>,
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
        page_query: String,
        engine_errors_info: Vec<EngineErrorInfo>,
    ) -> Self {
        SearchResults {
            results,
            page_query,
            style: Style::new("".to_string(), "".to_string()),
            engine_errors_info,
        }
    }

    /// A setter function to add website style to the return search results.
    pub fn add_style(&mut self, style: Style) {
        self.style = style;
    }
}
