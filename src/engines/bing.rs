//! The `bing` module handles scraping results from the Bing search engine.

use std::collections::HashMap;

use reqwest::Client;
use reqwest::header::HeaderMap;
use scraper::Html;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, EngineResult, SearchEngine};

use error_stack::{Report, ResultExt};

use super::search_result_parser::SearchResultParser;
use super::{build_cookie, url_encode};

/// A Bing engine handler implementing the [`SearchEngine`] trait.
pub struct Bing {
    /// The parser used to interpret search results.
    parser: SearchResultParser,
}

impl Bing {
    /// Creates a new Bing parser.
    ///
    /// # Errors
    ///
    /// Returns an [`EngineError`] if CSS selector compilation fails.
    pub fn new() -> EngineResult<Self> {
        Ok(Self {
            parser: SearchResultParser::new(
                ".b_results",
                ".b_algo",
                "h2 a",
                ".tpcn a.tilk",
                ".b_caption p",
            )?,
        })
    }
}

#[async_trait::async_trait]
impl SearchEngine for Bing {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        safe_search: u8,
    ) -> EngineResult<Vec<(String, SearchResult)>> {
        // Bing uses 1-based indexing for search result pagination.
        let results_per_page: u32 = 10;
        let start_result = results_per_page.saturating_mul(page).saturating_add(1);

        let adlt = match safe_search {
            0 => "off",
            1 => "demote",
            _ => "strict",
        };

        let encoded_query = url_encode(query);
        let url: String = if page == 0 {
            format!("https://www.bing.com/search?q={encoded_query}&adlt={adlt}")
        } else {
            format!(
                "https://www.bing.com/search?q={encoded_query}&first={start_result}&adlt={adlt}"
            )
        };

        let query_params: Vec<(&str, &str)> = vec![
            ("_EDGE_V", "1"),
            ("SRCHD=AF", "NOFORM"),
            ("_Rwho=u", "d"),
            ("bngps=s", "0"),
            ("_UR=QS=0&TQS", "0"),
            ("adlt", adlt),
        ];

        let cookie_string = build_cookie(&query_params);

        let header_map = HeaderMap::try_from(&HashMap::from([
            ("User-Agent".to_string(), user_agent.to_string()),
            ("Referer".to_string(), "https://google.com/".to_string()),
            (
                "Content-Type".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ),
            ("Cookie".to_string(), cookie_string),
        ]))
        .change_context(EngineError::UnexpectedError)?;

        let document: Html = Html::parse_document(
            &Bing::fetch_html_from_upstream(self, &url, header_map, client).await?,
        );

        if let Some(no_result_msg) = self.parser.parse_for_no_results(&document).nth(0)
            && no_result_msg
                .value()
                .attr("class")
                .map(|classes| classes.contains("b_algo"))
                .unwrap_or(false)
        {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        self.parser
            .parse_for_results(&document, |title, url, desc| {
                let href = url.value().attr("href")?;
                let clean_title = title.text().collect::<String>();
                let clean_desc = desc.text().collect::<String>();
                Some(SearchResult::new(
                    clean_title.trim(),
                    href.trim(),
                    clean_desc.trim(),
                    &["bing"],
                ))
            })
    }
}
