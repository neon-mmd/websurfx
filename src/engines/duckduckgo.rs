//! The `duckduckgo` module handles the scraping of results from the duckduckgo search engine
//! by querying the upstream duckduckgo search engine with user provided query and with a page
//! number if provided.

use reqwest::Client;
use scraper::Html;

use crate::models::aggregation::SearchResult;

use crate::models::engine::{EngineError, EngineResult, SearchEngine};

use error_stack::Report;

use super::search_result_parser::SearchResultParser;

/// Pre-compiled regex for cleaning DuckDuckGo result URLs
static DDG_URL_CLEAN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

/// A new DuckDuckGo engine type defined in-order to implement the `SearchEngine` trait which allows to
/// reduce code duplication as well as allows to create vector of different search engines easily.
pub struct DuckDuckGo {
    /// The parser, used to interpret the search result.
    parser: SearchResultParser,
}

impl DuckDuckGo {
    /// Creates the DuckDuckGo parser.
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
        _safe_search: u8,
    ) -> EngineResult<Vec<(String, SearchResult)>> {
        // Page number can be missing or empty string and so appropriate handling is required
        // so that upstream server recieves valid page number.
        let url: String = if page == 0 {
            format!("https://html.duckduckgo.com/html/?q={query}&s=&dc=&v=1&o=json&api=/d.js")
        } else {
            format!(
                "https://duckduckgo.com/html/?q={query}&s={}&dc={}&v=1&o=json&api=/d.js",
                page * 30,
                page * 30 + 1
            )
        };

        // initializing HeaderMap and adding appropriate headers.
        let header_map = crate::models::engine::build_headers(&[
            ("User-Agent", user_agent),
            ("Referer", "https://google.com/"),
            ("Content-Type", "application/x-www-form-urlencoded"),
            ("Cookie", "kl=wt-wt"),
        ])?;

        let html = DuckDuckGo::fetch_html_from_upstream(self, &url, header_map, client).await?;
        let document: Html = Html::parse_document(&html);

        if self.parser.parse_for_no_results(&document).next().is_some() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        let url_clean_re = DDG_URL_CLEAN.get_or_init(|| regex::Regex::new(r"^//").unwrap());

        // scrape all the results from the html
        self.parser
            .parse_for_results(&document, |title, url_elem, desc| {
                let url_html = url_elem.inner_html();
                let url_text = url_html.trim();
                let clean_url = url_clean_re.replace(url_text, "https://");
                Some(SearchResult::new(
                    title.inner_html().trim(),
                    clean_url.as_ref(),
                    desc.inner_html().trim(),
                    &["duckduckgo"],
                ))
            })
    }
}
