//! This module provides the error enum to handle different errors associated while requesting data from
//! the upstream search engines with the search query provided by the user.

use error_stack::Context;
use std::fmt;

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

impl Context for EngineError {}
