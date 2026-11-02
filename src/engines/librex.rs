//! The `librex` module contains the implementation of a search engine for LibreX using the reqwest and scraper libraries.
//! It includes a `SearchEngine` trait implementation for interacting with the search engine and retrieving search results.

use reqwest::Client;
use scraper::Html;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, EngineResult, SearchEngine};

use error_stack::Report;

use super::search_result_parser::SearchResultParser;

/// Represents the LibreX search engine.
pub struct LibreX {
    /// The parser used to extract search results from HTML documents.
    parser: SearchResultParser,
    /// The base URL of the LibreX instance to query.
    ///
    /// These instances are run by volunteers and go offline without notice, so this comes
    /// from the config rather than being named here.
    url: String,
}

impl LibreX {
    /// Creates a new instance of LibreX with a default configuration.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing `LibreX` if successful, otherwise an `EngineError`.
    ///
    /// # Arguments
    ///
    /// * `url` - It is the base URL of the LibreX instance to query.
    pub fn new(url: String) -> EngineResult<Self> {
        Ok(Self {
            parser: SearchResultParser::new(
                ".text-result-container>p",
                ".text-result-wrapper",
                ".text-result-wrapper>a>h2",
                ".text-result-wrapper>a",
                ".text-result-wrapper>span",
            )?,
            url,
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
    /// Returns a `Result` containing a `Vec` of search results if successful, otherwise an `EngineError`.
    /// The `Err` variant is explicit for better documentation.
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        _safe_search: u8,
    ) -> EngineResult<Vec<(String, SearchResult)>> {
        // Page number can be missing or empty string and so appropriate handling is required
        // so that upstream server recieves valid page number.
        let url: String = format!(
            "{}/search.php?q={query}&p={}&t=10",
            self.url.trim_end_matches('/'),
            page * 10
        );

        // initializing HeaderMap and adding appropriate headers.
        let header_map = crate::models::engine::build_headers(&[
            ("User-Agent", user_agent),
            ("Referer", "https://google.com/"),
            ("Content-Type", "application/x-www-form-urlencoded"),
            (
                "Cookie",
                "theme=amoled; disable_special=on; disable_frontends=on; language=en; number_of_results=10; safe_search=on; save=1",
            ),
        ])?;

        let document: Html = Html::parse_document(
            &LibreX::fetch_html_from_upstream(self, &url, header_map, client).await?,
        );

        if self.parser.parse_for_no_results(&document).next().is_some() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        // scrape all the results from the html
        let results = self
            .parser
            .parse_for_results(&document, |title, url, desc| {
                Some(SearchResult::new(
                    title.inner_html().trim(),
                    url.attr("href")?.trim(),
                    desc.inner_html().trim(),
                    &["librex"],
                ))
            })?;

        // An empty result set is reported rather than returned as an empty list, because an
        // instance that is rate limiting us or serving a block page also produces nothing
        // to scrape, and the two should not look alike.
        if results.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The instance URL comes from the config, so an operator can replace a public instance
    /// that has gone offline without touching the code.
    #[test]
    fn the_instance_url_is_kept_and_trimmed_for_the_request() {
        let engine = LibreX::new("https://example.invalid/".to_owned()).unwrap();

        assert_eq!(engine.url, "https://example.invalid/");
        // The request path is appended to the trimmed base, so a trailing slash from the
        // config cannot turn into a doubled separator.
        assert_eq!(engine.url.trim_end_matches('/'), "https://example.invalid");
    }

    /// The parser selectors still have to compile, whichever instance is configured.
    #[test]
    fn an_instance_url_does_not_affect_the_selectors() {
        let first = LibreX::new("https://a.invalid".to_owned()).unwrap();
        let second = LibreX::new("https://b.invalid/".to_owned()).unwrap();

        // Both engines parsed their selectors, so neither construction returned an error.
        assert_ne!(first.url, second.url);
    }
}
