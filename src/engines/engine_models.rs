//! This module provides the error enum to handle different errors associated while requesting data from
//! the upstream search engines with the search query provided by the user.

use crate::results::aggregation_models::SearchResult;
use error_stack::{Result, ResultExt};
use std::{collections::HashMap, fmt, time::Duration};

/// A custom error type used for handle engine associated errors.
///
/// This enum provides variants three different categories of errors:
/// * `RequestError` - This variant handles all request related errors like forbidden, not found,
/// etc.
/// * `EmptyResultSet` - This variant handles the not results found error provide by the upstream
/// search engines.
/// * `UnexpectedError` - This variant handles all the errors which are unexpected or occur rarely
/// and are errors mostly related to failure in initialization of HeaderMap, Selector errors and
/// all other errors occurring within the code handling the `upstream search engines`.
#[derive(Debug)]
pub enum EngineError {
    EmptyResultSet,
    RequestError,
    UnexpectedError,
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::EmptyResultSet => {
                write!(f, "The upstream search engine returned an empty result set")
            }
            EngineError::RequestError => {
                write!(
                    f,
                    "Error occurred while requesting data from upstream search engine"
                )
            }
            EngineError::UnexpectedError => {
                write!(f, "An unexpected error occurred while processing the data")
            }
        }
    }
}

impl error_stack::Context for EngineError {}

/// A trait to define common behavior for all search engines.
#[async_trait::async_trait]
pub trait SearchEngine: Sync + Send {
    async fn fetch_html_from_upstream(
        &self,
        url: &str,
        header_map: reqwest::header::HeaderMap,
        request_timeout: u8,
    ) -> Result<String, EngineError> {
        // fetch the html from upstream search engine
        Ok(reqwest::Client::new()
            .get(url)
            .timeout(Duration::from_secs(request_timeout as u64)) // Add timeout to request to avoid DDOSing the server
            .headers(header_map) // add spoofed headers to emulate human behavior
            .send()
            .await
            .change_context(EngineError::RequestError)?
            .text()
            .await
            .change_context(EngineError::RequestError)?)
    }

    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        request_timeout: u8,
        safe_search: u8,
    ) -> Result<HashMap<String, SearchResult>, EngineError>;
}

pub struct EngineHandler {
    engine: Box<dyn SearchEngine>,
    name: &'static str,
}

impl Clone for EngineHandler {
    fn clone(&self) -> Self {
        Self::new(self.name).unwrap()
    }
}

impl EngineHandler {
    /// parses an engine name into an engine handler, returns none if the engine is unknown
    pub fn new(engine_name: &str) -> Option<Self> {
        let engine: (&'static str, Box<dyn SearchEngine>) =
            match engine_name.to_lowercase().as_str() {
                "duckduckgo" => ("duckduckgo", Box::new(super::duckduckgo::DuckDuckGo)),
                "searx" => ("searx", Box::new(super::searx::Searx)),
                _ => return None,
            };

        Some(Self {
            engine: engine.1,
            name: engine.0,
        })
    }

    pub fn into_name_engine(self) -> (&'static str, Box<dyn SearchEngine>) {
        (self.name, self.engine)
    }
}
