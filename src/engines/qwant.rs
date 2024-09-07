//! The `qwant` module handles the scraping of results from the qwant search engine
//! by querying the upstream qwant search engine with user provided query and with a page
//! number if provided.

use std::collections::HashMap;

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation_models::SearchResult;

use crate::models::engine_models::{EngineError, SearchEngine};

use error_stack::{Report, Result, ResultExt};

/// A new Qwant engine type defined in-order to implement the `SearchEngine` trait which allows to
/// reduce code duplication as well as allows to create vector of different search engines easily.
pub struct Qwant;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/// Web page search result
struct QwantSearchResult {
    // NOTE: This object also contains `favicon`, `url_ping_suffix`, `thumbnail_url`,
    //       `source`, and `is_family_friendly` attributes,
    //       which we currently don't care about.
    /// Title of the result
    title: String,
    /// Url of the result
    url: String,
    /// Description of the result
    desc: String,
}

impl From<&QwantSearchResult> for SearchResult {
    fn from(value: &QwantSearchResult) -> Self {
        SearchResult::new(&value.title, &value.url, &value.desc, &["qwant"])
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
/// A result which should be shown to the user
enum QwantItem {
    /// Results containing web pages relevant to the query
    Web {
        // NOTE: This object also contains `count` and `serpContextId` attributes,
        //       which we currently don't care about.
        /// List of web page search results
        items: Vec<QwantSearchResult>,
    },
    #[serde(other)]
    /// Other item type like "related_searches", which aren't relevant.
    Other,
}

#[derive(Deserialize, Debug)]
struct QwantItems {
    // NOTE: This object also contains `headline`, `sidebar`, and `bottomline` attributes,
    //       which we currently don't care about.
    /// Results which should be shown in the main section of the page
    mainline: Vec<QwantItem>,
}

#[derive(Deserialize, Debug)]
struct QwantResult {
    // NOTE: This object also contains `denied`, `total`, `items`, `filters`, `lastPage`,
    //       `instrumentation`, `onlyProductAds`, and `topClassification` attributes,
    //       which we currently don't care about.
    /// Entries that should be shown to the user
    items: QwantItems,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "status", content = "data")]
enum QwantApiResponse {
    /// Success response
    Success {
        // NOTE: This object also contains `query` and `cache` attributes,
        //       which we currently don't care about.
        /// Actual results the search produced
        result: QwantResult,
    },
    // TODO: Use the reported error messages
    #[allow(unused)]
    /// Error response
    Error {
        /// Machine-readable error code
        error_code: i32,
        #[serde(default)]
        /// List of human-readable error messages
        message: Vec<String>,
    },
}

impl From<QwantApiResponse> for Result<QwantResult, EngineError> {
    fn from(value: QwantApiResponse) -> Self {
        match value {
            QwantApiResponse::Success { result } => Ok(result),
            QwantApiResponse::Error { .. } => Err(Report::new(EngineError::RequestError)),
        }
    }
}

#[async_trait::async_trait]
impl SearchEngine for Qwant {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let results_per_page = 10;
        let start_result = results_per_page * page;

        let url: String =  format!("https://api.qwant.com/v3/search/web?q={query}&count={results_per_page}&locale=en_US&offset={start_result}&safesearch={safe_search}&device=desktop&tgp=2&displayed=true");

        let header_map = HeaderMap::try_from(&HashMap::from([
            ("User-Agent".to_string(), user_agent.to_string()),
            ("Referer".to_string(), "https://qwant.com/".to_string()),
            ("Origin".to_string(), "https://qwant.com".to_string()),
        ]))
        .change_context(EngineError::UnexpectedError)?;

        let result: QwantApiResponse = client
            .get(url)
            .headers(header_map)
            .send()
            .await
            .change_context(EngineError::RequestError)?
            .json()
            .await
            .change_context(EngineError::RequestError)?;

        let result = Result::from(result)?;

        let results: Vec<_> = result
            .items
            .mainline
            .into_iter()
            .filter_map(|item| match item {
                QwantItem::Web { items } => Some(items),
                _ => None,
            })
            .flatten()
            .map(|result| {
                let search_result = SearchResult::from(&result);
                (result.url, search_result)
            })
            .collect();

        if results.is_empty() {
            Err(Report::new(EngineError::EmptyResultSet))
        } else {
            Ok(results)
        }
    }
}
