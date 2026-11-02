//! The `searx` module handles scraping results from a Searx search engine instance.

use reqwest::Client;
use reqwest::header::HeaderMap;
use scraper::Html;
use std::collections::HashMap;

use super::search_result_parser::SearchResultParser;
use super::url_encode;
use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, EngineResult, SearchEngine};
use error_stack::{Report, ResultExt};

/// A Searx engine handler implementing the [`SearchEngine`] trait.
pub struct Searx {
    /// The parser used to interpret search results.
    parser: SearchResultParser,
}

impl Searx {
    /// Creates a new Searx parser.
    ///
    /// # Errors
    ///
    /// Returns an [`EngineError`] if CSS selector compilation fails.
    pub fn new() -> EngineResult<Searx> {
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
        client: &Client,
        safe_search: u8,
    ) -> EngineResult<Vec<(String, SearchResult)>> {
        // Map safe search levels: 0 => none (0), 1 => moderate (1), 2..=4 => strict (2)
        let upstream_safesearch = match safe_search {
            0 => 0,
            1 => 1,
            _ => 2,
        };

        let encoded_query = url_encode(query);
        let pageno = page.saturating_add(1);
        let url: String = format!(
            "https://searx.be/search?q={encoded_query}&pageno={pageno}&safesearch={upstream_safesearch}"
        );

        let header_map = HeaderMap::try_from(&HashMap::from([
            ("User-Agent".to_string(), user_agent.to_string()),
            ("Referer".to_string(), "https://google.com/".to_string()),
            ("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string()),
            ("Cookie".to_string(), "categories=general; language=auto; locale=en; autocomplete=duckduckgo; image_proxy=1; method=POST; safesearch=2; theme=simple; results_on_new_tab=1; doi_resolver=oadoi.org; simple_style=auto; center_alignment=1; query_in_title=1; infinite_scroll=0; disabled_engines=; enabled_engines=\"archive is__general\\054yep__general\\054curlie__general\\054currency__general\\054ddg definitions__general\\054wikidata__general\\054duckduckgo__general\\054tineye__general\\054lingva__general\\054startpage__general\\054yahoo__general\\054wiby__general\\054marginalia__general\\054alexandria__general\\054wikibooks__general\\054wikiquote__general\\054wikisource__general\\054wikiversity__general\\054wikivoyage__general\\054dictzone__general\\054seznam__general\\054mojeek__general\\054naver__general\\054wikimini__general\\054brave__general\\054petalsearch__general\\054goo__general\"; disabled_plugins=; enabled_plugins=\"searx.plugins.hostname_replace\\054searx.plugins.oa_doi_rewrite\\054searx.plugins.vim_hotkeys\"; tokens=; maintab=on; enginetab=on".to_string())
        ]))
        .change_context(EngineError::UnexpectedError)?;

        let document: Html = Html::parse_document(
            &Searx::fetch_html_from_upstream(self, &url, header_map, client).await?,
        );

        if let Some(no_result_msg) = self.parser.parse_for_no_results(&document).nth(1)
            && no_result_msg.inner_html()
                == "we didn't find any results. Please use another query or search in more categories"
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
                    &["searx"],
                ))
            })
    }
}
