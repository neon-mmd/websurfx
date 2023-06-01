//! This module provides the error enum to handle different errors associated while requesting data from
//! the upstream search engines with the search query provided by the user.

use reqwest::header::InvalidHeaderValue;
use scraper::error::SelectorErrorKind;

/// A custom error type used for handle engine associated errors.
///
/// This enum provides variants three different categories of errors:
/// * `RequestError` - This variant handles all request related errors like forbidden, not found,
/// etc.
/// * `EmptyResultSet` - This variant handles the not results found error provide by the upstream
/// search engines.
/// * `UnexpectedError` - This variant handles all the errors which are unexpected or occur rarely
/// and are errors mostly related to failure in initialization of HeaderMap, Selector errors and
/// all other errors.
#[derive(Debug)]
pub enum EngineErrorKind {
    RequestError(reqwest::Error),
    EmptyResultSet,
    UnexpectedError(String),
}

/// Implementing `Display` trait to make errors writable on the stdout and also providing/passing the
/// appropriate errors that should be written to the stdout when this error is raised/encountered.
impl std::fmt::Display for EngineErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineErrorKind::RequestError(request_error) => write!(f, "{}", request_error),
            EngineErrorKind::EmptyResultSet => {
                write!(f, "The upstream search engine returned an empty result set")
            }
            EngineErrorKind::UnexpectedError(unexpected_error) => write!(f, "{}", unexpected_error),
        }
    }
}

/// Implementing `Error` trait to make the the `EngineErrorKind` enum an error type.
impl std::error::Error for EngineErrorKind {}

/// Implementing `From` trait to map the `SelectorErrorKind` to `UnexpectedError` variant.
impl<'a> From<SelectorErrorKind<'a>> for EngineErrorKind {
    fn from(err: SelectorErrorKind<'a>) -> Self {
        Self::UnexpectedError(err.to_string())
    }
}

/// Implementing `From` trait to map the `InvalidHeaderValue` to `UnexpectedError` variant.
impl From<InvalidHeaderValue> for EngineErrorKind {
    fn from(err: InvalidHeaderValue) -> Self {
        Self::UnexpectedError(err.to_string())
    }
}

/// Implementing `From` trait to map all `reqwest::Error` to `UnexpectedError` variant.
impl From<reqwest::Error> for EngineErrorKind {
    fn from(err: reqwest::Error) -> Self {
        Self::RequestError(err)
    }
}
