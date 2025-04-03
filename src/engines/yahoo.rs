//! The `yahoo` module handles the scraping of results from the yahoo search engine
//! by querying the upstream yahoo search engine with user provided query and with a page

use std::collections::HashMap;

use reqwest::header::HeaderMap;

// use reqwest::{Client, Error};

use reqwest::Client;

use scraper::Html;

use crate::models::aggregation_models::SearchResult;

use crate::models::engine_models::{EngineError, SearchEngine};

use error_stack::{Report, Result, ResultExt};

use super::search_result_parser::SearchResultParser;

/// A new Yahoo engine type defined in-order to implement the `SearchEngine` trait which allows to
/// reduce code duplication as well as allows to create vector of different search engines easily.
pub struct Yahoo {
    /// The parser, used to interpret the search result.
    parser: SearchResultParser,
    // Used to retrieve the original link from redirect_url.
    // client: Client,
}

impl Yahoo {
    /// Creates the Yahoo parser.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            parser: SearchResultParser::new(
                ".compNoResult",
                "div.algo",
                "h3.title a",
                "h3 a",
                ".compText",
            )?,
            // client: Client::new(),
        })
    }
    //TODO: Function not implemented yet
    //
    // Function to fetch the final destination URL after handling redirects
    // Yahoo search results provide a redirect link when scraping HTML. This function helps retrieve the final URL.
    // async fn get_final_url(&self, redirect_url: &str) -> Result<String, Error> {
    //     // Send a GET request and let it follow redirects
    //     let response = self.client.get(redirect_url).send().await?;
    //
    //     // Extract the final destination URL (after following redirects)
    //     let final_url = response.url().as_str().to_string();
    //
    //     Ok(final_url)
    // }
}

#[async_trait::async_trait]
impl SearchEngine for Yahoo {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        // Page number can be missing or empty string and so appropriate handling is required
        // so that upstream server recieves valid page number.
        let url: String = match page {
            0 => format!("https://search.yahoo.com/search/?p={}", query),

            _ => format!(
                "https://search.yahoo.com/search/?p={}&b={}",
                query,
                (page * 10) + 1
            ),
        };

        // initializing HeaderMap and adding appropriate headers.
        let header_map = HeaderMap::try_from(&HashMap::from([
            ("User-Agent".to_string(), user_agent.to_string()),
            ("Referer".to_string(), "https://google.com/".to_string()),
            (
                "Content-Type".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ),
            ("Cookie".to_string(), "kl=wt-wt".to_string()),
        ]))
        .change_context(EngineError::UnexpectedError)?;

        let document: Html = Html::parse_document(
            &Yahoo::fetch_html_from_upstream(self, &url, header_map, client).await?,
        );

        if self.parser.parse_for_no_results(&document).next().is_some() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        self.parser
            .parse_for_results(&document, |title, url, desc| {
                // Scrape the HTML to extract and clean the data.
                let cleaned_title = title
                    .attr("aria-label")
                    .unwrap_or("No Title Found")
                    .trim()
                    .to_owned();
                let cleaned_url = url
                    .value()
                    .attr("href")
                    .unwrap_or("No Link Found")
                    .to_owned();

                let cleaned_description = desc.inner_html().trim().to_owned();
                Some(SearchResult::new(
                    &cleaned_title,
                    &cleaned_url,
                    &cleaned_description,
                    &["yahoo"],
                ))
            })
    }
}
