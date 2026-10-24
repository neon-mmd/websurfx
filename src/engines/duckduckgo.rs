// Rust guideline compliant 2026-10-09

//! The `duckduckgo` module handles scraping results from the DuckDuckGo search engine.

use std::collections::HashMap;

use reqwest::Client;
use reqwest::header::HeaderMap;
use scraper::Html;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, EngineResult, SearchEngine};

use error_stack::{Report, ResultExt};

use super::search_result_parser::SearchResultParser;
use super::url_encode;

/// A DuckDuckGo engine handler implementing the [`SearchEngine`] trait.
pub struct DuckDuckGo {
    /// The parser used to interpret search results.
    parser: SearchResultParser,
}

impl DuckDuckGo {
    /// Creates a new DuckDuckGo parser.
    ///
    /// # Errors
    ///
    /// Returns an [`EngineError`] if CSS selector compilation fails.
    pub fn new() -> EngineResult<Self> {
        Ok(Self {
            parser: SearchResultParser::new(
                ".no-results",
                ".results>.result",
                ".result__title>.result__a",
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
        client: &Client,
        safe_search: u8,
    ) -> EngineResult<Vec<(String, SearchResult)>> {
        let kp = match safe_search {
            0 => "-2",
            1 => "-1",
            _ => "1",
        };

        let encoded_query = url_encode(query);
        let url: String = if page == 0 {
            format!(
                "https://html.duckduckgo.com/html/?q={encoded_query}&s=&dc=&v=1&o=json&api=/d.js&kp={kp}"
            )
        } else {
            let s = page.saturating_mul(30);
            let dc = s.saturating_add(1);
            format!(
                "https://duckduckgo.com/html/?q={encoded_query}&s={s}&dc={dc}&v=1&o=json&api=/d.js&kp={kp}"
            )
        };

        let header_map = HeaderMap::try_from(&HashMap::from([
            ("User-Agent".to_string(), user_agent.to_string()),
            ("Referer".to_string(), "https://google.com/".to_string()),
            (
                "Content-Type".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ),
            ("Cookie".to_string(), format!("kl=wt-wt; kp={kp}")),
        ]))
        .change_context(EngineError::UnexpectedError)?;

        let document: Html = Html::parse_document(
            &DuckDuckGo::fetch_html_from_upstream(self, &url, header_map, client).await?,
        );

        if self.parser.parse_for_no_results(&document).next().is_some() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        self.parser
            .parse_for_results(&document, |title, url, desc| {
                let raw_url = url.text().collect::<String>();
                let trimmed_url = raw_url.trim();
                let full_url =
                    if trimmed_url.starts_with("http://") || trimmed_url.starts_with("https://") {
                        trimmed_url.to_string()
                    } else {
                        format!("https://{trimmed_url}")
                    };
                let clean_title = title.text().collect::<String>();
                let clean_desc = desc.text().collect::<String>();
                Some(SearchResult::new(
                    clean_title.trim(),
                    &full_url,
                    clean_desc.trim(),
                    &["duckduckgo"],
                ))
            })
    }
}
