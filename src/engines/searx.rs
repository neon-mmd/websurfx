//! The `searx` module handles the scraping of results from the searx search engine instance
//! by querying the upstream searx search engine instance with user provided query and with a page
//! number if provided.

use reqwest::header::HeaderMap;
use scraper::Html;
use std::collections::HashMap;

use super::search_result_parser::SearchResultParser;
use crate::models::aggregation_models::SearchResult;
use crate::models::engine_models::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// A new Searx engine type defined in-order to implement the `SearchEngine` trait which allows to
/// reduce code duplication as well as allows to create vector of different search engines easily.
pub struct Searx {
    /// The parser, used to interpret the search result.
    parser: SearchResultParser,
}

impl Searx {
    /// creates a Searx parser
    pub fn new() -> Result<Searx, EngineError> {
        Ok(Self {
            parser: SearchResultParser::new(
                "#urls>.dialog-error>p",
                ".result",
                "h3>a",
                "h3>a",
                ".content",
            )?,
        })
    }
}

#[async_trait::async_trait]
impl SearchEngine for Searx {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        request_timeout: u8,
        mut safe_search: u8,
    ) -> Result<HashMap<String, SearchResult>, EngineError> {
        // Page number can be missing or empty string and so appropriate handling is required
        // so that upstream server recieves valid page number.
        if safe_search == 3 {
            safe_search = 2;
        };

        let url: String = match page {
            0 | 1 => {
                format!("https://searx.work/search?q={query}&pageno=1&safesearch={safe_search}")
            }
            _ => format!(
                "https://searx.work/search?q={query}&pageno={page}&safesearch={safe_search}"
            ),
        };

        // initializing headers and adding appropriate headers.
        let header_map = HeaderMap::try_from(&HashMap::from([
            ("USER_AGENT".to_string(), user_agent.to_string()),
            ("REFERER".to_string(), "https://google.com/".to_string()),
            ("CONTENT_TYPE".to_string(), "application/x-www-form-urlencoded".to_string()),
            ("COOKIE".to_string(), "categories=general; language=auto; locale=en; autocomplete=duckduckgo; image_proxy=1; method=POST; safesearch=2; theme=simple; results_on_new_tab=1; doi_resolver=oadoi.org; simple_style=auto; center_alignment=1; query_in_title=1; infinite_scroll=0; disabled_engines=; enabled_engines=\"archive is__general\\054yep__general\\054curlie__general\\054currency__general\\054ddg definitions__general\\054wikidata__general\\054duckduckgo__general\\054tineye__general\\054lingva__general\\054startpage__general\\054yahoo__general\\054wiby__general\\054marginalia__general\\054alexandria__general\\054wikibooks__general\\054wikiquote__general\\054wikisource__general\\054wikiversity__general\\054wikivoyage__general\\054dictzone__general\\054seznam__general\\054mojeek__general\\054naver__general\\054wikimini__general\\054brave__general\\054petalsearch__general\\054goo__general\"; disabled_plugins=; enabled_plugins=\"searx.plugins.hostname_replace\\054searx.plugins.oa_doi_rewrite\\054searx.plugins.vim_hotkeys\"; tokens=; maintab=on; enginetab=on".to_string())
        ]))
        .change_context(EngineError::UnexpectedError)?;

        let document: Html = Html::parse_document(
            &Searx::fetch_html_from_upstream(self, &url, header_map, request_timeout).await?,
        );

        if let Some(no_result_msg) = self.parser.parse_for_no_results(&document).nth(1) {
            if no_result_msg.inner_html()
            == "we didn't find any results. Please use another query or search in more categories"
        {
            return Err(Report::new(EngineError::EmptyResultSet));
        }
        }

        // scrape all the results from the html
        self.parser
            .parse_for_results(&document, |title, url, desc| {
                url.value().attr("href").map(|url| {
                    SearchResult::new(
                        title.inner_html().trim(),
                        url,
                        desc.inner_html().trim(),
                        &["searx"],
                    )
                })
            })
    }
}
