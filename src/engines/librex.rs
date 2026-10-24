// Rust guideline compliant 2026-10-09

//! The `librex` module contains the implementation of a search engine for LibreX.

use std::collections::HashMap;

use reqwest::Client;
use reqwest::header::HeaderMap;
use scraper::Html;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, EngineResult, SearchEngine};

use error_stack::{Report, ResultExt};

use super::search_result_parser::SearchResultParser;
use super::url_encode;

/// Represents the LibreX search engine.
pub struct LibreX {
    /// The parser used to extract search results from HTML documents.
    parser: SearchResultParser,
}

impl LibreX {
    /// Creates a new instance of LibreX with a default configuration.
    ///
    /// # Errors
    ///
    /// Returns an [`EngineError`] if CSS selector compilation fails.
    pub fn new() -> EngineResult<Self> {
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
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        safe_search: u8,
    ) -> EngineResult<Vec<(String, SearchResult)>> {
        let encoded_query = url_encode(query);
        let p = page.saturating_mul(10);
        let url: String =
            format!("https://search.davidovski.xyz/search.php?q={encoded_query}&p={p}&t=10");

        let safe_search_val = if safe_search == 0 { "off" } else { "on" };

        let header_map = HeaderMap::try_from(&HashMap::from([
            ("User-Agent".to_string(), user_agent.to_string()),
            ("Referer".to_string(), "https://google.com/".to_string()),
            ("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string()),
            (
                "Cookie".to_string(),
                format!("theme=amoled; disable_special=on; disable_frontends=on; language=en; number_of_results=10; safe_search={safe_search_val}; save=1"),
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
                let href = url.attr("href")?;
                let clean_title = title.text().collect::<String>();
                let clean_desc = desc.text().collect::<String>();
                Some(SearchResult::new(
                    clean_title.trim(),
                    href.trim(),
                    clean_desc.trim(),
                    &["librex"],
                ))
            })
    }
}
