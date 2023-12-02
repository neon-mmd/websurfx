//! The `duckduckgo` module handles the scraping of results from the duckduckgo search engine
//! by querying the upstream duckduckgo search engine with user provided query and with a page
//! number if provided.

use std::collections::HashMap;
use std::sync::Arc;

use reqwest::header::HeaderMap;

use scraper::Html;

use crate::models::aggregation_models::SearchResult;

use crate::models::client_models::HttpClient;
use crate::models::engine_models::{EngineError, EngineErrorType, QueryType, SearchEngine};

use error_stack::{Result, ResultExt};

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
            )
            .change_context(EngineError {
                error_type: EngineErrorType::UnexpectedError,
                engine: "duckduckgo".to_string(),
            })?,
        })
    }
}

#[async_trait::async_trait]
impl SearchEngine for DuckDuckGo {
    fn get_name(&self) -> &'static str {
        "duckduckgo"
    }

    fn get_query_types(&self) -> QueryType {
        QueryType::Text
    }

    async fn fetch_results(
        &self,
        query: &str,
        // category: QueryType,
        // query_relevance: Option<QueryRelavancy>,
        page: u32,
        client: Arc<HttpClient>,
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
            ("REFERER".to_string(), "https://google.com/".to_string()),
            (
                "CONTENT_TYPE".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ),
            ("COOKIE".to_string(), "kl=wt-wt".to_string()),
        ]))
        .change_context(EngineError {
            error_type: EngineErrorType::UnexpectedError,
            engine: self.get_name().to_string(),
        })?;

        let document: Html = Html::parse_document(
            &client
                .fetch_html(&url, header_map, None)
                .await
                .change_context(EngineError {
                    error_type: EngineErrorType::RequestError,
                    engine: self.get_name().to_string(),
                })?,
        );

        if self.parser.parse_for_no_results(&document).next().is_some() {
            return Err(EngineError {
                error_type: EngineErrorType::EmptyResultSet,
                engine: self.get_name().to_string(),
            }
            .into());
        }

        // scrape all the results from the html
        Ok(self
            .parser
            .parse_for_results(&document, |title, url, desc| {
                Some(SearchResult::new(
                    title.inner_html().trim(),
                    &format!("https://{}", url.inner_html().trim()),
                    desc.inner_html().trim(),
                    &["duckduckgo"],
                ))
            }))
    }
}
