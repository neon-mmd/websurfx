//! The `duckduckgo` module handles the scraping of results from the duckduckgo search engine
//! by querying the upstream duckduckgo search engine with user provided query and with a page
//! number if provided.

use std::collections::HashMap;

use reqwest::header::HeaderMap;
use scraper::Html;

use crate::models::aggregation_models::SearchResult;

use crate::models::engine_models::{EngineError, SearchEngine};

use error_stack::{Report, Result, ResultExt};

use super::search_result_parser::SearchResultParser;

/// A new DuckDuckGo engine type defined in-order to implement the `SearchEngine` trait which allows to
/// reduce code duplication as well as allows to create vector of different search engines easily.
pub struct DuckDuckGo {
    /// The parser, used to interpret the search result.
    parser: SearchResultParser,
}

impl DuckDuckGo {
    /// Creates the DuckDuckGo parser.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            parser: SearchResultParser::new(
                ".no-results",
                ".result",
                ".result__a",
                ".result__url",
                ".result__snippet",
            )?,
        })
    }
}

#[async_trait::async_trait]
impl SearchEngine for DuckDuckGo {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        request_timeout: u8,
        _safe_search: u8,
    ) -> Result<HashMap<String, SearchResult>, EngineError> {
        // Page number can be missing or empty string and so appropriate handling is required
        // so that upstream server recieves valid page number.
        let url: String = match page {
            1 | 0 => {
                format!("https://html.duckduckgo.com/html/?q={query}&s=&dc=&v=1&o=json&api=/d.js")
            }
            _ => {
                format!(
                    "https://duckduckgo.com/html/?q={}&s={}&dc={}&v=1&o=json&api=/d.js",
                    query,
                    (page / 2 + (page % 2)) * 30,
                    (page / 2 + (page % 2)) * 30 + 1
                )
            }
        };

        // initializing HeaderMap and adding appropriate headers.
        let header_map = HeaderMap::try_from(&HashMap::from([
            ("USER_AGENT".to_string(), user_agent.to_string()),
            ("REFERER".to_string(), "https://google.com/".to_string()),
            (
                "CONTENT_TYPE".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ),
            ("COOKIE".to_string(), "kl=wt-wt".to_string()),
        ]))
        .change_context(EngineError::UnexpectedError)?;

        let document: Html = Html::parse_document(
            &DuckDuckGo::fetch_html_from_upstream(self, &url, header_map, request_timeout).await?,
        );

        if self.parser.parse_for_no_results(&document).next().is_some() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        // scrape all the results from the html
        self.parser
            .parse_for_results(&document, |title, url, desc| {
                Some(SearchResult::new(
                    title.inner_html().trim(),
                    &format!("https://{}", url.inner_html().trim()),
                    desc.inner_html().trim(),
                    &["duckduckgo"],
                ))
            })
    }
}
