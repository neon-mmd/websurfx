//! This module provides the error enum to handle different errors associated while requesting data from
//! the upstream search engines with the search query provided by the user.

use crate::search_results_handler::aggregation_models::RawSearchResult;
use error_stack::{IntoReport, Result, ResultExt};
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
/// all other errors occuring within the code handling the `upstream search engines`.
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

#[async_trait::async_trait]
pub trait SearchEngine {
    async fn fetch_html_from_upstream(
        &self,
        url: String,
        header_map: reqwest::header::HeaderMap,
    ) -> Result<String, EngineError> {
        // fetch the html from upstream search engine
        Ok(reqwest::Client::new()
            .get(url)
            .timeout(Duration::from_secs(30))
            .headers(header_map) // add spoofed headers to emulate human behaviour
            .send()
            .await
            .into_report()
            .change_context(EngineError::RequestError)?
            .text()
            .await
            .into_report()
            .change_context(EngineError::RequestError)?)
    }

    async fn results(
        &self,
        query: String,
        page: u32,
        user_agent: String,
    ) -> Result<HashMap<String, RawSearchResult>, EngineError>;
}
