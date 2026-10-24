// Rust guideline compliant 2026-10-09

//! The `wikipedia` module handles scraping results from Wikipedia.

use std::collections::HashMap;

use reqwest::Client;
use reqwest::header::HeaderMap;
use scraper::Html;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, EngineResult, SearchEngine};

use error_stack::{Report, ResultExt};

use super::build_query;
use super::search_result_parser::SearchResultParser;

/// A Wikipedia engine handler implementing the [`SearchEngine`] trait.
pub struct Wikipedia {
    /// The parser used to interpret search results.
    parser: SearchResultParser,
    /// The id of the engine, equals to 'wikipedia-' + language.
    id: String,
    /// The host where wikipedia can be accessed.
    host: String,
}

impl Wikipedia {
    /// Creates a new Wikipedia parser for a given language code.
    ///
    /// # Errors
    ///
    /// Returns an [`EngineError`] if CSS selector compilation fails.
    pub fn new(language: &str) -> EngineResult<Self> {
        let host = format!("https://{}.wikipedia.org", language);
        let id = format!("wikipedia-{}", language);
        Ok(Self {
            parser: SearchResultParser::new(
                "p.mw-search-nonefound",
                ".mw-search-results li.mw-search-result",
                ".mw-search-result-heading a",
                ".mw-search-result-heading a",
                ".searchresult",
            )?,
            id,
            host,
        })
    }
}

#[async_trait::async_trait]
impl SearchEngine for Wikipedia {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        _safe_search: u8,
    ) -> EngineResult<Vec<(String, SearchResult)>> {
        let header_map = HeaderMap::try_from(&HashMap::from([
            ("User-Agent".to_string(), user_agent.to_string()),
            ("Referer".to_string(), self.host.to_string()),
        ]))
        .change_context(EngineError::UnexpectedError)?;

        let offset = page.saturating_mul(20).to_string();
        let query_params: Vec<(&str, &str)> = vec![
            ("limit", "20"),
            ("offset", &offset),
            ("profile", "default"),
            ("search", query),
            ("title", "Special:Search"),
            ("ns0", "1"),
        ];

        let query_params_string = build_query(&query_params);

        let url: String = format!("{}/w/index.php?{}", self.host, query_params_string);

        let document: Html = Html::parse_document(
            &Wikipedia::fetch_html_from_upstream(self, &url, header_map, client).await?,
        );

        if self.parser.parse_for_no_results(&document).next().is_some() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        // scrape all the results from the html
        self.parser
            .parse_for_results(&document, |title, url, desc| {
                let found_url = url.attr("href");
                found_url.map(|relative_url| {
                    let clean_title = title.text().collect::<String>();
                    let clean_desc = desc.text().collect::<String>();
                    SearchResult::new(
                        clean_title.trim(),
                        &format!("{}{relative_url}", self.host),
                        clean_desc.trim(),
                        &[&self.id],
                    )
                })
            })
    }
}
