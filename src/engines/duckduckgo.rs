//! The `duckduckgo` module handles the scraping of results from the duckduckgo search engine
//! by querying the upstream duckduckgo search engine with user provided query and with a page
//! number if provided.

use std::collections::HashMap;

use reqwest::header::{HeaderMap, CONTENT_TYPE, COOKIE, REFERER, USER_AGENT};
use scraper::{Html, Selector};

use crate::models::aggregation_models::SearchResult;

use crate::models::engine_models::{EngineError, SearchEngine};

use error_stack::{IntoReport, Report, Result, ResultExt};

/// A new DuckDuckGo engine type defined in-order to implement the `SearchEngine` trait which allows to
/// reduce code duplication as well as allows to create vector of different search engines easily.
pub struct DuckDuckGo;

#[async_trait::async_trait]
impl SearchEngine for DuckDuckGo {
    async fn results(
        &self,
        query: String,
        page: u32,
        user_agent: String,
        request_timeout: u8,
    ) -> Result<HashMap<String, SearchResult>, EngineError> {
        // Page number can be missing or empty string and so appropriate handling is required
        // so that upstream server recieves valid page number.
        let url: String = match page {
            1 | 0 => {
                format!("https://html.duckduckgo.com/html/?q={query}&s=&dc=&v=1&o=json&api=/d.js")
            }
            _ => {
                format!(
                    "https://duckduckgo.com/html/?q={}&s={}&dc={}&v=1&o=json&api=/d.js",
                    query,
                    (page / 2 + (page % 2)) * 30,
                    (page / 2 + (page % 2)) * 30 + 1
                )
            }
        };

        // initializing HeaderMap and adding appropriate headers.
        let mut header_map = HeaderMap::new();
        header_map.insert(
            USER_AGENT,
            user_agent
                .parse()
                .into_report()
                .change_context(EngineError::UnexpectedError)?,
        );
        header_map.insert(
            REFERER,
            "https://google.com/"
                .parse()
                .into_report()
                .change_context(EngineError::UnexpectedError)?,
        );
        header_map.insert(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded"
                .parse()
                .into_report()
                .change_context(EngineError::UnexpectedError)?,
        );
        header_map.insert(
            COOKIE,
            "kl=wt-wt"
                .parse()
                .into_report()
                .change_context(EngineError::UnexpectedError)?,
        );

        let document: Html = Html::parse_document(
            &DuckDuckGo::fetch_html_from_upstream(self, url, header_map, request_timeout).await?,
        );

        let no_result: Selector = Selector::parse(".no-results")
            .map_err(|_| Report::new(EngineError::UnexpectedError))
            .attach_printable_lazy(|| format!("invalid CSS selector: {}", ".no-results"))?;

        if document.select(&no_result).next().is_some() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        let results: Selector = Selector::parse(".result")
            .map_err(|_| Report::new(EngineError::UnexpectedError))
            .attach_printable_lazy(|| format!("invalid CSS selector: {}", ".result"))?;
        let result_title: Selector = Selector::parse(".result__a")
            .map_err(|_| Report::new(EngineError::UnexpectedError))
            .attach_printable_lazy(|| format!("invalid CSS selector: {}", ".result__a"))?;
        let result_url: Selector = Selector::parse(".result__url")
            .map_err(|_| Report::new(EngineError::UnexpectedError))
            .attach_printable_lazy(|| format!("invalid CSS selector: {}", ".result__url"))?;
        let result_desc: Selector = Selector::parse(".result__snippet")
            .map_err(|_| Report::new(EngineError::UnexpectedError))
            .attach_printable_lazy(|| format!("invalid CSS selector: {}", ".result__snippet"))?;

        // scrape all the results from the html
        Ok(document
            .select(&results)
            .map(|result| {
                SearchResult::new(
                    result
                        .select(&result_title)
                        .next()
                        .unwrap()
                        .inner_html()
                        .trim()
                        .to_string(),
                    format!(
                        "https://{}",
                        result
                            .select(&result_url)
                            .next()
                            .unwrap()
                            .inner_html()
                            .trim()
                    ),
                    result
                        .select(&result_desc)
                        .next()
                        .unwrap()
                        .inner_html()
                        .trim()
                        .to_string(),
                    vec!["duckduckgo".to_string()],
                )
            })
            .map(|search_result| (search_result.url.clone(), search_result))
            .collect())
    }
}
