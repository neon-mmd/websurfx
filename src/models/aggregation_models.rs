//! This module provides public models for handling, storing and serializing of search results
//! data scraped from the upstream search engines.

use super::engine_models::EngineError;
use serde::{Deserialize, Serialize};
#[cfg(any(
    feature = "use-synonyms-search",
    feature = "use-non-static-synonyms-search"
))]
use thesaurus::synonyms;
/// A named struct to store the raw scraped search results scraped search results from the
/// upstream search engines before aggregating it.It derives the Clone trait which is needed
/// to write idiomatic rust using `Iterators`.
///
///   (href url in html in simple words).
///
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
    /// The td-tdf score of the result in regards to the title, url and description and the user's query
    pub relevance_score: f32,
}

impl SearchResult {
    /// Constructs a new `RawSearchResult` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the search result.
    /// * `url` - The url which is accessed when clicked on it
    ///   (href url in html in simple words).
    /// * `description` - The description of the search result.
    /// * `engine` - The names of the upstream engines from which this results were provided.
    pub fn new(title: &str, url: &str, description: &str, engine: &[&str]) -> Self {
        SearchResult {
            title: title.to_owned(),
            url: url.to_owned(),
            description: description.to_owned(),
            relevance_score: 0.0,
            engine: engine.iter().map(|name| name.to_string()).collect(),
        }
    }
    /// calculates and update the relevance score of the current search.

    /// # Arguments
    ///
    /// * query -  the query string  used to obtain the results
    ///
    ///

    pub fn calculate_relevance(&mut self, query: &str) {
        use stop_words::{get, LANGUAGE};
        // when language settings can change to any of the ones supported on this crate: https://docs.rs/crate/stop-words/0.8.0
        let documents = [
            self.title.clone(),
            self.url.clone(),
            self.description.clone(),
        ];

        let stop_words = get(LANGUAGE::English);
        let punctuation = [
            ".".to_owned(),
            ",".to_owned(),
            ":".to_owned(),
            ";".to_owned(),
            "!".to_owned(),
            "?".to_owned(),
            "(".to_owned(),
            ")".to_owned(),
            "[".to_owned(),
            "]".to_owned(),
            "{".to_owned(),
            "}".to_owned(),
            "\"".to_owned(),
            "'".to_owned(),
            "<".to_owned(),
            ">".to_owned(),
        ];

        self.relevance_score = calculate_tf_idf(query, &documents, &stop_words, &punctuation);
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
    ///   search engine.
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
    pub results: Box<[SearchResult]>,
    /// Stores the information on which engines failed with their engine name
    /// and the type of error that caused it.
    pub engine_errors_info: Box<[EngineErrorInfo]>,
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
    ///   and stores it into a vector of `SearchResult` structs.
    /// * `page_query` - Takes an argument of current page`s search query `q` provided in
    ///   the search url.
    /// * `engine_errors_info` - Takes an array of structs which contains information regarding
    ///   which engines failed with their names, reason and their severity color name.
    pub fn new(results: Box<[SearchResult]>, engine_errors_info: Box<[EngineErrorInfo]>) -> Self {
        Self {
            results,
            engine_errors_info,
            disallowed: Default::default(),
            filtered: Default::default(),
            safe_search_level: Default::default(),
            no_engines_selected: Default::default(),
        }
    }

    /// A setter function that sets disallowed to true.
    pub fn set_disallowed(&mut self) {
        self.disallowed = true;
    }

    /// A setter function that sets the filtered to true.
    pub fn set_filtered(&mut self, filtered: bool) {
        self.filtered = filtered;
    }

    /// A getter function that gets the value of `engine_errors_info`.
    pub fn engine_errors_info(&mut self) -> Box<[EngineErrorInfo]> {
        std::mem::take(&mut self.engine_errors_info)
    }
    /// A getter function that gets the value of `results`.
    pub fn results(&mut self) -> Box<[SearchResult]> {
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

/// Helper function to calculate the tf-idf for the search query.
/// <br> The approach is  as [`as`](https://en.wikipedia.org/wiki/Tf%E2%80%93idf).
///  <br> Find a sample article about TF-IDF [`here`](https://medium.com/analytics-vidhya/tf-idf-term-frequency-technique-easiest-explanation-for-text-classification-in-nlp-with-code-8ca3912e58c3)
/// ### Arguments
/// * `query` -  a user's search query
/// * `documents` -  a list of text used for comparision (url, title, description)
/// * `stop_words` - A list of language specific stop words.
/// * `punctuation` - list of punctuation symbols.
/// ### Returns
/// * `score` - The average tf-idf score of the word tokens (and synonyms) in the query
fn calculate_tf_idf(
    query: &str,
    documents: &[String],
    stop_words: &[String],
    punctuation: &[String],
) -> f32 {
    use keyword_extraction::{
        tf_idf::{TfIdf, TfIdfParams},
        tokenizer::Tokenizer,
    };

    let params = TfIdfParams::UnprocessedDocuments(documents, stop_words, Some(punctuation));
    let tf_idf = TfIdf::new(params);
    let tokener = Tokenizer::new(query, stop_words, Some(punctuation));
    let query_tokens = tokener.split_into_words();

    #[cfg(any(
        feature = "use-synonyms-search",
        feature = "use-non-static-synonyms-search"
    ))]
    let mut extra_tokens = vec![];

    let total_score: f32 = query_tokens
        .iter()
        .map(|token| {
            #[cfg(any(
                feature = "use-synonyms-search",
                feature = "use-non-static-synonyms-search"
            ))]
            {
                // find some synonyms and add them to the search  (from wordnet or moby if feature is enabled)
                extra_tokens.extend(synonyms(token))
            }

            tf_idf.get_score(token)
        })
        .sum();

    #[cfg(not(any(
        feature = "use-synonyms-search",
        feature = "use-non-static-synonyms-search"
    )))]
    let result = total_score / (query_tokens.len() as f32);

    #[cfg(any(
        feature = "use-synonyms-search",
        feature = "use-non-static-synonyms-search"
    ))]
    let extra_total_score: f32 = extra_tokens
        .iter()
        .map(|token| tf_idf.get_score(token))
        .sum();

    #[cfg(any(
        feature = "use-synonyms-search",
        feature = "use-non-static-synonyms-search"
    ))]
    let result =
        (extra_total_score + total_score) / ((query_tokens.len() + extra_tokens.len()) as f32);

    f32::from(!result.is_nan()) * result
}
