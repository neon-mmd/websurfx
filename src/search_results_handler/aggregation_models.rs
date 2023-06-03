//! This module provides public models for handling, storing and serializing of search results
//! data scraped from the upstream search engines.

use serde::{Deserialize, Serialize};

use crate::config_parser::parser_models::Style;

/// A named struct to store, serialize and deserializes the individual search result from all the
/// scraped and aggregated search results from the upstream search engines.
///
/// # Fields
///
/// * `title` - The title of the search result.
/// * `visiting_url` - The url which is accessed when clicked on it (href url in html in simple
/// words).
/// * `url` - The url to be displayed below the search result title in html.
/// * `description` - The description of the search result.
/// * `engine` - The names of the upstream engines from which this results were provided.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub title: String,
    pub visiting_url: String,
    pub url: String,
    pub description: String,
    pub engine: Vec<String>,
}

impl SearchResult {
    /// Constructs a new `SearchResult` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the search result.
    /// * `visiting_url` - The url which is accessed when clicked on it
    /// (href url in html in simple words).
    /// * `url` - The url to be displayed below the search result title in html.
    /// * `description` - The description of the search result.
    /// * `engine` - The names of the upstream engines from which this results were provided.
    pub fn new(
        title: String,
        visiting_url: String,
        url: String,
        description: String,
        engine: Vec<String>,
    ) -> Self {
        SearchResult {
            title,
            visiting_url,
            url,
            description,
            engine,
        }
    }
}

/// A named struct to store the raw scraped search results scraped search results from the
/// upstream search engines before aggregating it.It derives the Clone trait which is needed
/// to write idiomatic rust using `Iterators`.
///
/// # Fields
///
/// * `title` - The title of the search result.
/// * `visiting_url` - The url which is accessed when clicked on it
/// (href url in html in simple words).
/// * `description` - The description of the search result.
/// * `engine` - The names of the upstream engines from which this results were provided.
#[derive(Clone)]
pub struct RawSearchResult {
    pub title: String,
    pub visiting_url: String,
    pub description: String,
    pub engine: Vec<String>,
}

impl RawSearchResult {
    /// Constructs a new `RawSearchResult` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the search result.
    /// * `visiting_url` - The url which is accessed when clicked on it
    /// (href url in html in simple words).
    /// * `description` - The description of the search result.
    /// * `engine` - The names of the upstream engines from which this results were provided.
    pub fn new(
        title: String,
        visiting_url: String,
        description: String,
        engine: Vec<String>,
    ) -> Self {
        RawSearchResult {
            title,
            visiting_url,
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

/// A named struct to store, serialize, deserialize the all the search results scraped and
/// aggregated from the upstream search engines.
///
/// # Fields
///
/// * `results` - Stores the individual serializable `SearchResult` struct into a vector of
/// `SearchResult` structs.
/// * `page_query` - Stores the current pages search query `q` provided in the search url.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub page_query: String,
    pub style: Style,
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
    pub fn new(results: Vec<SearchResult>, page_query: String) -> Self {
        SearchResults {
            results,
            page_query,
            style: Style::new("".to_string(), "".to_string()),
        }
    }

    pub fn add_style(&mut self, style: Style) {
        self.style = style;
    }
}
