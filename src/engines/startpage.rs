//! The `startpage` module handles scraping results from the Startpage search engine.

use std::collections::HashMap;

use reqwest::Client;
use reqwest::header::HeaderMap;
use scraper::Html;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, EngineResult, SearchEngine};

use error_stack::{Report, ResultExt};

use super::search_result_parser::SearchResultParser;
use super::url_encode;

/// A Startpage engine handler implementing the [`SearchEngine`] trait.
pub struct Startpage {
    /// The parser used to interpret search results.
    parser: SearchResultParser,
}

impl Startpage {
    /// Creates a new Startpage parser.
    ///
    /// # Errors
    ///
    /// Returns an [`EngineError`] if CSS selector compilation fails.
    pub fn new() -> EngineResult<Self> {
        Ok(Self {
            parser: SearchResultParser::new(
                ".no-results",
                ".w-gl__result__main",
                ".w-gl__result-second-line-container>.w-gl__result-title>h3",
                ".w-gl__result-url",
                ".w-gl__description",
            )?,
        })
    }
}

#[async_trait::async_trait]
impl SearchEngine for Startpage {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        safe_search: u8,
    ) -> EngineResult<Vec<(String, SearchResult)>> {
        let encoded_query = url_encode(query);
        let start = page.saturating_mul(10);
        let url: String =
            format!("https://startpage.com/do/dsearch?q={encoded_query}&num=10&start={start}",);

        let disable_family_filter = if safe_search == 0 { "1" } else { "0" };

        let header_map = HeaderMap::try_from(&HashMap::from([
            ("User-Agent".to_string(), user_agent.to_string()),
            ("Referer".to_string(), "https://google.com/".to_string()),
            (
                "Content-Type".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ),
            ("Cookie".to_string(), format!("preferences=connect_to_serverEEE0N1Ndate_timeEEEworldN1Ndisable_family_filterEEE{disable_family_filter}N1Ndisable_open_in_new_windowEEE0N1Nenable_post_methodEEE1N1Nenable_proxy_safety_suggestEEE1N1Nenable_stay_controlEEE0N1Ninstant_answersEEE1N1Nlang_homepageEEEs%2Fnight%2FenN1NlanguageEEEenglishN1Nlanguage_uiEEEenglishN1Nnum_of_resultsEEE10N1Nsearch_results_regionEEEallN1NsuggestionsEEE1N1Nwt_unitEEEcelsius")),
        ]))
        .change_context(EngineError::UnexpectedError)?;

        let document: Html = Html::parse_document(
            &Startpage::fetch_html_from_upstream(self, &url, header_map, client).await?,
        );

        if self.parser.parse_for_no_results(&document).next().is_some() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        self.parser
            .parse_for_results(&document, |title, url, desc| {
                let clean_url = url
                    .attr("href")
                    .map(|h| h.trim().to_string())
                    .unwrap_or_else(|| url.text().collect::<String>().trim().to_string());
                if clean_url.is_empty() {
                    return None;
                }
                let clean_title = title.text().collect::<String>();
                let clean_desc = desc.text().collect::<String>();
                Some(SearchResult::new(
                    clean_title.trim(),
                    &clean_url,
                    clean_desc.trim(),
                    &["startpage"],
                ))
            })
    }
}
