//! The `librex` module contains the implementation of a search engine for LibreX using the reqwest and scraper libraries.
//! It includes a `SearchEngine` trait implementation for interacting with the search engine and retrieving search results.

use std::collections::HashMap;

use reqwest::Client;
use reqwest::header::HeaderMap;
use scraper::Html;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

use error_stack::{Report, Result, ResultExt};

use super::search_result_parser::SearchResultParser;

/// Represents the LibreX search engine.
pub struct LibreX {
    /// The parser used to extract search results from HTML documents.
    parser: SearchResultParser,
}

impl LibreX {
    /// Creates a new instance of LibreX with a default configuration.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing `LibreX` if successful, otherwise an `EngineError`.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            parser: SearchResultParser::new(
                ".text-result-container>p",
                ".text-result-wrapper",
                ".text-result-wrapper>a>h2",
                ".text-result-wrapper>a",
                ".text-result-wrapper>span",
            )?,
        })
    }
}

#[async_trait::async_trait]
impl SearchEngine for LibreX {
    /// Retrieve LibreX search results for a query and page.
    ///
    /// Builds and fetches the LibreX search page for `query` and `page`, parses the HTML,
    /// and returns the extracted results.
    ///
    /// # Returns
    ///
    /// `Ok` with a vector of `(title, SearchResult)` tuples on success; `Err(EngineError::EmptyResultSet)` when no results are found, or another `EngineError` for unexpected failures.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use reqwest::Client;
    /// # use crate::librex::LibreX;
    /// # tokio_test::block_on(async {
    /// let engine = LibreX::new().unwrap();
    /// let client = Client::new();
    /// let res = engine.results("example query", 0, "my-agent", &client, 0).await;
    /// assert!(res.is_ok() || matches!(res.unwrap_err(), crate::models::EngineError::EmptyResultSet));
    /// # });
    /// ```
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        // Page number can be missing or empty string and so appropriate handling is required
        // so that upstream server recieves valid page number.
        let url: String = format!(
            "https://search.davidovski.xyz/search.php?q={query}&p={}&t=10",
            page * 10
        );

        // initializing HeaderMap and adding appropriate headers.
        let header_map = HeaderMap::try_from(&HashMap::from([
            ("User-Agent".to_string(), user_agent.to_string()),
            ("Referer".to_string(), "https://google.com/".to_string()),
            ("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string()),
            (
                "Cookie".to_string(),
                "theme=amoled; disable_special=on; disable_frontends=on; language=en; number_of_results=10; safe_search=on; save=1".to_string(),
            ),
        ]))
        .change_context(EngineError::UnexpectedError)?;

        let document: Html = Html::parse_document(
            &LibreX::fetch_html_from_upstream(self, &url, header_map, client).await?,
        );

        if self.parser.parse_for_no_results(&document).next().is_some() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        // scrape all the results from the html
        self.parser
            .parse_for_results(&document, |title, url, desc| {
                Some(SearchResult::new(
                    title.inner_html().trim(),
                    url.attr("href")?.trim(),
                    desc.inner_html().trim(),
                    &["librex"],
                ))
            })
    }
}