//! The `bing` module handles the scraping of results from the bing search engine
//! by querying the upstream bing search engine with user provided query and with a page
//! number if provided.

use regex::Regex;
use reqwest::Client;
use scraper::Html;

/// Pre-compiled regexes for Bing result cleaning
static BING_SPAN_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
/// Matches the `<strong>` runs Bing wraps highlighted query terms in.
static BING_STRONG_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();

use crate::models::aggregation::SearchResult;

use crate::models::engine::{EngineError, EngineResult, SearchEngine};

use error_stack::Report;

use super::build_cookie;
use super::search_result_parser::SearchResultParser;

/// A new Bing engine type defined in-order to implement the `SearchEngine` trait which allows to
/// reduce code duplication as well as allows to create vector of different search engines easily.
pub struct Bing {
    /// The parser, used to interpret the search result.
    parser: SearchResultParser,
}

impl Bing {
    /// Creates the Bing parser.
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
        _safe_search: u8,
    ) -> EngineResult<Vec<(String, SearchResult)>> {
        // Bing uses `start results from this number` convention
        // So, for 10 results per page, page 0 starts at 1, page 1
        // starts at 11, and so on.
        let results_per_page = 10;
        let start_result = results_per_page * page + 1;

        let url: String = if page == 0 {
            format!("https://www.bing.com/search?q={query}")
        } else {
            format!("https://www.bing.com/search?q={query}&first={start_result}")
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

        let header_map = crate::models::engine::build_headers(&[
            ("User-Agent", user_agent),
            ("Referer", "https://google.com/"),
            ("Content-Type", "application/x-www-form-urlencoded"),
            ("Cookie", &cookie_string),
        ])?;

        let document: Html = Html::parse_document(
            &Bing::fetch_html_from_upstream(self, &url, header_map, client).await?,
        );

        // Bing is very aggressive in finding matches
        // even with the most absurd of queries. ".b_algo" is the
        // class for the list item of results
        if let Some(no_result_msg) = self.parser.parse_for_no_results(&document).nth(0)
            && no_result_msg
                .value()
                .attr("class")
                .map(|classes| classes.contains("b_algo"))
                .unwrap_or(false)
        {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        let re_span = BING_SPAN_RE
            .get_or_init(|| Regex::new(r#"<span.*?>.*?(?:</span>&nbsp;·|</span>)"#).unwrap());
        let re_strong =
            BING_STRONG_RE.get_or_init(|| Regex::new(r#"(<strong>|</strong>)"#).unwrap());

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
