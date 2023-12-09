/// # LibreX Search Engine
///
/// The `LibreX` module contains the implementation of a search engine for LibreX using the reqwest and scraper libraries.
/// It includes a `SearchEngine` trait implementation for interacting with the search engine and retrieving search results.

use std::collections::HashMap;

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::Html;

use crate::models::aggregation_models::SearchResult;
use crate::models::engine_models::{EngineError, SearchEngine};

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
                ".text-result-container",
                ".text-result-container>a>h2",
                ".text-result-container>a",
                ".text-result-container>span",
            )?,
        })
    }
}

#[async_trait::async_trait]
impl SearchEngine for LibreX {
    /// Retrieves search results from LibreX based on the provided query, page, user agent, and client.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query.
    /// * `page` - The page number for pagination.
    /// * `user_agent` - The user agent string.
    /// * `client` - The reqwest client for making HTTP requests.
    /// * `_safe_search` - A parameter for safe search (not currently used).
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `HashMap` of search results if successful, otherwise an `EngineError`.
    #[allow(clippy::unnecessary_wraps)] // The `Err` variant is explicit for better documentation.
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        _safe_search: u8,
    ) -> Result<HashMap<String, SearchResult>, EngineError> {
        let url: String = match page {
            0 => format!("https://search.ahwx.org/search.php?q={query}&p=0&t=10"),
            _ => format!("https://search.ahwx.org/search.php?q={query}&p={}&t=10", page * 10),
        };
        
        let header_map = HeaderMap::try_from(&HashMap::from([
            ("USER_AGENT".to_string(), user_agent.to_string()),
            ("REFERER".to_string(), "https://google.com/".to_string()),
            ("CONTENT_TYPE".to_string(), "application/x-www-form-urlencoded".to_string()),
            (
                "COOKIE".to_string(),
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

        self.parser
            .parse_for_results(&document, |title, url, desc| {
                Some(SearchResult::new(
                    title.inner_html().trim(),
                    url.inner_html().trim(),
                    desc.inner_html().trim(),
                    &["librex"],
                ))
            })
    }
}
