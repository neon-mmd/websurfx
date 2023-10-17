//! This module provides the error enum to handle different errors associated while requesting data from
//! the upstream search engines with the search query provided by the user.

use super::aggregation_models::SearchResult;
use error_stack::{Report, Result, ResultExt};
use std::{collections::HashMap, fmt, time::Duration};

/// A custom error type used for handle engine associated errors.
#[derive(Debug)]
pub enum EngineError {
    /// No matching engine found
    NoSuchEngineFound(String),
    /// This variant handles all request related errors like forbidden, not found,
    /// etc.
    EmptyResultSet,
    /// This variant handles the not results found error provide by the upstream
    /// search engines.
    RequestError,
    ///  This variant handles all the errors which are unexpected or occur rarely
    /// and are errors mostly related to failure in initialization of HeaderMap,
    /// Selector errors and all other errors occurring within the code handling
    /// the `upstream search engines`.
    UnexpectedError,
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::NoSuchEngineFound(engine) => {
                write!(f, "No such engine with the name '{engine}' found")
            }
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
    /// This helper function fetches/requests the search results from the upstream search engine in
    /// an html form.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes the url of the upstream search engine with the user requested search
    /// query appended in the search parameters.
    /// * `header_map` - It takes the http request headers to be sent to the upstream engine in
    /// order to prevent being detected as a bot. It takes the header as a HeaderMap type.
    /// * `request_timeout` - It takes the request timeout value as seconds which is used to limit
    /// the amount of time for each request to remain connected when until the results can be provided
    /// by the upstream engine.
    ///
    /// # Error
    ///
    /// It returns the html data as a string if the upstream engine provides the data as expected
    /// otherwise it returns a custom `EngineError`.
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

    /// This function scrapes results from the upstream engine and puts all the scraped results like
    /// title, visiting_url (href in html),engine (from which engine it was fetched from) and description
    /// in a RawSearchResult and then adds that to HashMap whose keys are url and values are RawSearchResult
    /// struct and then returns it within a Result enum.
    ///
    /// # Arguments
    ///
    /// * `query` - Takes the user provided query to query to the upstream search engine with.
    /// * `page` - Takes an u32 as an argument.
    /// * `user_agent` - Takes a random user agent string as an argument.
    /// * `request_timeout` - Takes a time (secs) as a value which controls the server request timeout.
    ///
    /// # Errors
    ///
    /// Returns an `EngineErrorKind` if the user is not connected to the internet or if their is failure to
    /// reach the above `upstream search engine` page or if the `upstream search engine` is unable to
    /// provide results for the requested search query and also returns error if the scraping selector
    /// or HeaderMap fails to initialize.
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        request_timeout: u8,
        safe_search: u8,
    ) -> Result<HashMap<String, SearchResult>, EngineError>;
}

/// A named struct which stores the engine struct with the name of the associated engine.
pub struct EngineHandler {
    /// It stores the engine struct wrapped in a box smart pointer as the engine struct implements
    /// the `SearchEngine` trait.
    engine: Box<dyn SearchEngine>,
    /// It stores the name of the engine to which the struct is associated to.
    name: &'static str,
}

impl Clone for EngineHandler {
    fn clone(&self) -> Self {
        Self::new(self.name).unwrap()
    }
}

impl EngineHandler {
    /// Parses an engine name into an engine handler.
    ///
    /// # Arguments
    ///
    /// * `engine_name` - It takes the name of the engine to which the struct was associated to.
    ///
    /// # Returns
    ///
    /// It returns an option either containing the value or a none if the engine is unknown
    pub fn new(engine_name: &str) -> Result<Self, EngineError> {
        let engine: (&'static str, Box<dyn SearchEngine>) =
            match engine_name.to_lowercase().as_str() {
                "duckduckgo" => {
                    let engine = crate::engines::duckduckgo::DuckDuckGo::new()?;
                    ("duckduckgo", Box::new(engine))
                }
                "searx" => {
                    let engine = crate::engines::searx::Searx::new()?;
                    ("searx", Box::new(engine))
                }
                "brave" => {
                    let engine = crate::engines::brave::Brave::new()?;
                    ("brave", Box::new(engine))
                }
                _ => {
                    return Err(Report::from(EngineError::NoSuchEngineFound(
                        engine_name.to_string(),
                    )))
                }
            };

        Ok(Self {
            engine: engine.1,
            name: engine.0,
        })
    }

    /// This function converts the EngineHandler type into a tuple containing the engine name and
    /// the associated engine struct.
    pub fn into_name_engine(self) -> (&'static str, Box<dyn SearchEngine>) {
        (self.name, self.engine)
    }
}
