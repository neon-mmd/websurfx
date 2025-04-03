//! The `wikipedia` module handles the scraping of results from wikipedia
//! with user provided query and with a page number if provided.

use std::collections::HashMap;

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::Html;

use crate::models::aggregation_models::SearchResult;

use crate::models::engine_models::{EngineError, SearchEngine};

use error_stack::{Report, Result, ResultExt};

use super::common::build_query;
use super::search_result_parser::SearchResultParser;

/// A new Wikipedia engine type defined in-order to implement the `SearchEngine` trait which allows to
/// reduce code duplication as well as allows to create vector of different search engines easily.
pub struct Wikipedia {
    /// The parser, used to interpret the search result.
    parser: SearchResultParser,
    /// The id of the engine, equals to 'wikipedia-' + language
    id: String,
    /// The host where wikipedia can be accessed.
    host: String,
}

impl Wikipedia {
    /// Creates the Wikipedia parser.
    pub fn new(language: &str) -> Result<Self, EngineError> {
        let host = format!("https://{}.wikipedia.org", &language);
        let id = format!("wikipedia-{}", &language);
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
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let header_map = HeaderMap::try_from(&HashMap::from([
            ("User-Agent".to_string(), user_agent.to_string()),
            ("Referer".to_string(), self.host.to_string()),
        ]))
        .change_context(EngineError::UnexpectedError)?;

        let offset = (page * 20).to_string();
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
                    SearchResult::new(
                        title.inner_html().trim(),
                        &format!("{}{relative_url}", self.host),
                        desc.inner_html().trim(),
                        &[&self.id],
                    )
                })
            })
    }
}
