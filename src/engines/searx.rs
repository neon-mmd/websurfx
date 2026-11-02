//! The `searx` module handles the scraping of results from the searx search engine instance
//! by querying the upstream searx search engine instance with user provided query and with a page
//! number if provided.

use reqwest::Client;
use scraper::Html;

use super::search_result_parser::SearchResultParser;
use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, EngineResult, SearchEngine};
use error_stack::Report;

/// A new Searx engine type defined in-order to implement the `SearchEngine` trait which allows to
/// reduce code duplication as well as allows to create vector of different search engines easily.
pub struct Searx {
    /// The parser, used to interpret the search result.
    parser: SearchResultParser,
    /// The base URL of the SearXNG instance to query.
    ///
    /// These instances are run by volunteers and go offline without notice, so this comes
    /// from the config rather than being named here.
    url: String,
}

impl Searx {
    /// Creates a Searx parser.
    ///
    /// # Arguments
    ///
    /// * `url` - It is the base URL of the SearXNG instance to query.
    pub fn new(url: String) -> EngineResult<Searx> {
        Ok(Self {
            parser: SearchResultParser::new(
                "#urls>.dialog-error>p",
                ".result",
                "h3>a",
                "h3>a",
                ".content",
            )?,
            url,
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
        mut safe_search: u8,
    ) -> EngineResult<Vec<(String, SearchResult)>> {
        // A branchless condition to check whether the `safe_search` parameter has the
        // value greater than equal to three or not. If it is, then it modifies the
        // `safesearch` parameters value to 2.
        //
        // Moreover, the below branchless code is equivalent to the following code below:
        //
        // ```rust
        // safe_search = u8::from(safe_search == 3) * 2;
        // ```
        //
        // For more information on branchless programming. See:
        //
        // * https://piped.video/watch?v=bVJ-mWWL7cE
        safe_search = u8::from(safe_search >= 3) * 2;

        let url: String = format!(
            "{}/search?q={query}&pageno={}&safesearch={safe_search}",
            self.url.trim_end_matches('/'),
            page + 1
        );

        // initializing headers and adding appropriate headers.
        let header_map = crate::models::engine::build_headers(&[
            ("User-Agent", user_agent),
            ("Referer", "https://google.com/"),
            ("Content-Type", "application/x-www-form-urlencoded"),
            (
                "Cookie",
                "categories=general; language=auto; locale=en; autocomplete=duckduckgo; image_proxy=1; method=POST; safesearch=2; theme=simple; results_on_new_tab=1; doi_resolver=oadoi.org; simple_style=auto; center_alignment=1; query_in_title=1; infinite_scroll=0; disabled_engines=; enabled_engines=\"archive is__general\\054yep__general\\054curlie__general\\054currency__general\\054ddg definitions__general\\054wikidata__general\\054duckduckgo__general\\054tineye__general\\054lingva__general\\054startpage__general\\054yahoo__general\\054wiby__general\\054marginalia__general\\054alexandria__general\\054wikibooks__general\\054wikiquote__general\\054wikisource__general\\054wikiversity__general\\054wikivoyage__general\\054dictzone__general\\054seznam__general\\054mojeek__general\\054naver__general\\054wikimini__general\\054brave__general\\054petalsearch__general\\054goo__general\"; disabled_plugins=; enabled_plugins=\"searx.plugins.hostname_replace\\054searx.plugins.oa_doi_rewrite\\054searx.plugins.vim_hotkeys\"; tokens=; maintab=on; enginetab=on",
            ),
        ])?;

        let document: Html = Html::parse_document(
            &Searx::fetch_html_from_upstream(self, &url, header_map, client).await?,
        );

        if let Some(no_result_msg) = self.parser.parse_for_no_results(&document).nth(1)
            && no_result_msg.inner_html()
                == "we didn't find any results. Please use another query or search in more categories"
        {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        // scrape all the results from the html
        let results = self
            .parser
            .parse_for_results(&document, |title, url, desc| {
                url.value().attr("href").map(|url| {
                    SearchResult::new(
                        title.inner_html().trim(),
                        url,
                        desc.inner_html().trim(),
                        &["searx"],
                    )
                })
            })?;

        // An empty result set is reported rather than returned as an empty list, because an
        // instance that is rate limiting us or serving a block page also produces nothing
        // to scrape, and the two should not look alike.
        if results.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The instance URL comes from the config, so an operator can replace a public instance
    /// that has gone offline without touching the code.
    #[test]
    fn the_instance_url_is_kept_and_trimmed_for_the_request() {
        let engine = Searx::new("https://searx.example/".to_owned()).unwrap();

        assert_eq!(engine.url, "https://searx.example/");
        // The request path is appended to the trimmed base, so a trailing slash from the
        // config cannot turn into a doubled separator.
        assert_eq!(engine.url.trim_end_matches('/'), "https://searx.example");
    }
}
