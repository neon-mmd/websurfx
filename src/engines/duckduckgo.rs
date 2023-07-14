//! The `duckduckgo` module handles the scraping of results from the duckduckgo search engine
//! by querying the upstream duckduckgo search engine with user provided query and with a page
//! number if provided.

use std::collections::HashMap;

use reqwest::header::{HeaderMap, CONTENT_TYPE, COOKIE, REFERER, USER_AGENT};
use scraper::{Html, Selector};

use crate::search_results_handler::aggregation_models::RawSearchResult;

use super::engine_models::{EngineError, SearchEngine};

use error_stack::{IntoReport, Report, Result, ResultExt};

/// This function scrapes results from the upstream engine duckduckgo and puts all the scraped
/// results like title, visiting_url (href in html),engine (from which engine it was fetched from)
/// and description in a RawSearchResult and then adds that to HashMap whose keys are url and
/// values are RawSearchResult struct and then returns it within a Result enum.
///
/// # Arguments
///
/// * `query` - Takes the user provided query to query to the upstream search engine with.
/// * `page` - Takes an u32 as an argument.
/// * `user_agent` - Takes a random user agent string as an argument.
///
/// # Errors
///
/// Returns an `EngineErrorKind` if the user is not connected to the internet or if their is failure to
/// reach the above `upstream search engine` page or if the `upstream search engine` is unable to
/// provide results for the requested search query and also returns error if the scraping selector
/// or HeaderMap fails to initialize.

pub struct DuckDuckGo;

#[async_trait::async_trait]
impl SearchEngine for DuckDuckGo {
    async fn results(
        &self,
        query: String,
        page: u32,
        user_agent: String,
    ) -> Result<HashMap<String, RawSearchResult>, EngineError> {
        // Page number can be missing or empty string and so appropriate handling is required
        // so that upstream server recieves valid page number.
        let url: String = match page {
            1 => {
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
            &DuckDuckGo::fetch_html_from_upstream(&self, url, header_map).await?,
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
                RawSearchResult::new(
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
            .map(|search_result| (search_result.visiting_url.clone(), search_result))
            .collect())
    }
}
