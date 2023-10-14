//! This module provides public models for handling, storing and serializing of search results
//! data scraped from the upstream search engines.

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

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

/// A named struct that stores the error info related to the upstream search engines.
#[derive(Serialize, Deserialize, Clone)]
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
    pub fn new(error: &EngineError, engine: &str) -> Self {
        Self {
            error: match error {
                EngineError::NoSuchEngineFound(_) => "EngineNotFound".to_owned(),
                EngineError::RequestError => "RequestError".to_owned(),
                EngineError::EmptyResultSet => "EmptyResultSet".to_owned(),
                EngineError::UnexpectedError => "UnexpectedError".to_owned(),
            },
            engine: engine.to_owned(),
            severity_color: match error {
                EngineError::NoSuchEngineFound(_) => "red".to_owned(),
                EngineError::RequestError => "green".to_owned(),
                EngineError::EmptyResultSet => "blue".to_owned(),
                EngineError::UnexpectedError => "red".to_owned(),
            },
        }
    }
}

/// A named struct to store, serialize, deserialize the all the search results scraped and
/// aggregated from the upstream search engines.
/// `SearchResult` structs.
#[derive(Serialize, Deserialize, Default, Clone)]
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
    /// Stores the flag option which holds the check value that the following
    /// search query was disallowed when the safe search level set to 4 and it
    /// was present in the `Blocklist` file.
    pub disallowed: bool,
    /// Stores the flag option which holds the check value that the following
    /// search query was filtered when the safe search level set to 3 and it
    /// was present in the `Blocklist` file.
    pub filtered: bool,
    /// Stores the safe search level `safesearch` provided in the search url.
    pub safe_search_level: u8,
    /// Stores the flag option which holds the check value that whether any search engines were
    /// selected or not.
    pub no_engines_selected: bool,
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
    /// * `engine_errors_info` - Takes an array of structs which contains information regarding
    /// which engines failed with their names, reason and their severity color name.
    pub fn new(
        results: Vec<SearchResult>,
        page_query: &str,
        engine_errors_info: &[EngineErrorInfo],
    ) -> Self {
        Self {
            results,
            page_query: page_query.to_owned(),
            style: Style::default(),
            engine_errors_info: engine_errors_info.to_owned(),
            disallowed: Default::default(),
            filtered: Default::default(),
            safe_search_level: Default::default(),
            no_engines_selected: Default::default(),
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

    /// A setter function to set the current page safe search level.
    pub fn set_safe_search_level(&mut self, safe_search_level: u8) {
        self.safe_search_level = safe_search_level;
    }

    /// A getter function that gets the value of `no_engines_selected`.
    pub fn no_engines_selected(&self) -> bool {
        self.no_engines_selected
    }

    /// A setter function to set the `no_engines_selected` to true.
    pub fn set_no_engines_selected(&mut self) {
        self.no_engines_selected = true;
    }
}
