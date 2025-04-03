//! The `bing` module handles the scraping of results from the bing search engine
//! by querying the upstream bing search engine with user provided query and with a page
//! number if provided.

use std::collections::HashMap;

use regex::Regex;
use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::Html;

use crate::models::aggregation_models::SearchResult;

use crate::models::engine_models::{EngineError, SearchEngine};

use error_stack::{Report, Result, ResultExt};

use super::common::build_cookie;
use super::search_result_parser::SearchResultParser;

/// A new Bing engine type defined in-order to implement the `SearchEngine` trait which allows to
/// reduce code duplication as well as allows to create vector of different search engines easily.
pub struct Bing {
    /// The parser, used to interpret the search result.
    parser: SearchResultParser,
}

impl Bing {
    /// Creates the Bing parser.
    pub fn new() -> Result<Self, EngineError> {
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
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        // Bing uses `start results from this number` convention
        // So, for 10 results per page, page 0 starts at 1, page 1
        // starts at 11, and so on.
        let results_per_page = 10;
        let start_result = results_per_page * page + 1;

        let url: String = match page {
            0 => {
                format!("https://www.bing.com/search?q={query}")
            }
            _ => {
                format!("https://www.bing.com/search?q={query}&first={start_result}")
            }
        };

        let query_params: Vec<(&str, &str)> = vec![
            ("_EDGE_V", "1"),
            ("SRCHD=AF", "NOFORM"),
            ("_Rwho=u", "d"),
            ("bngps=s", "0"),
            ("_UR=QS=0&TQS", "0"),
            ("_UR=QS=0&TQS", "0"),
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

        // Bing is very aggressive in finding matches
        // even with the most absurd of queries. ".b_algo" is the
        // class for the list item of results
        if let Some(no_result_msg) = self.parser.parse_for_no_results(&document).nth(0) {
            if no_result_msg
                .value()
                .attr("class")
                .map(|classes| classes.contains("b_algo"))
                .unwrap_or(false)
            {
                return Err(Report::new(EngineError::EmptyResultSet));
            }
        }

        let re_span = Regex::new(r#"<span.*?>.*?(?:</span>&nbsp;·|</span>)"#).unwrap();
        let re_strong = Regex::new(r#"(<strong>|</strong>)"#).unwrap();

        // scrape all the results from the html
        self.parser
            .parse_for_results(&document, |title, url, desc| {
                Some(SearchResult::new(
                    &re_strong.replace_all(title.inner_html().trim(), ""),
                    url.value().attr("href").unwrap(),
                    &re_span.replace_all(desc.inner_html().trim(), ""),
                    &["bing"],
                ))
            })
    }
}
