//! The `qwant` module handles the fetching of results from the Qwant search engine
//! by querying the upstream Qwant JSON API with the user provided query and with a page
//! number if provided.

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};
use reqwest::{Client, header::HeaderMap};
use serde::Deserialize;
use std::collections::HashMap;

/// A new Qwant engine type defined in-order to implement the `SearchEngine` trait.
pub struct Qwant;

/// A single web page search result from the Qwant API.
#[derive(Deserialize)]
struct QwantSearchResult {
    /// Title of the result.
    title: String,
    /// URL of the result.
    url: String,
    /// Description snippet of the result.
    desc: String,
}

/// A categorized result item from the Qwant API.
#[derive(Deserialize)]
#[serde(tag = "type")]
enum QwantItem {
    /// Results containing web pages relevant to the query.
    #[serde(rename = "web")]
    Web {
        /// List of web page search results.
        items: Vec<QwantSearchResult>,
    },
    /// Other item types (related searches, ads, etc.) which are not relevant.
    #[serde(other)]
    Other,
}

/// The mainline container holding categorized result items.
#[derive(Deserialize)]
struct QwantMainline {
    /// Results displayed in the main section.
    mainline: Vec<QwantItem>,
}

/// The top-level result object from a successful Qwant API response.
#[derive(Deserialize)]
struct QwantResult {
    /// Categorized result items.
    items: QwantMainline,
}

/// The full Qwant API response, tagged by status.
#[derive(Deserialize)]
#[serde(tag = "status", content = "data", rename_all = "lowercase")]
enum QwantApiResponse {
    /// A successful response containing search results.
    Success {
        /// The actual search results.
        result: QwantResult,
    },
    /// An error response from the Qwant API.
    Error {
        /// Machine-readable error code.
        #[serde(rename = "errorCode")]
        error_code: i32,
        /// Human-readable error messages.
        #[serde(default)]
        message: Vec<String>,
    },
}

impl Qwant {
    /// Creates a new Qwant engine instance.
    ///
    /// # Returns
    ///
    /// It returns a `Qwant` struct on success.
    pub fn new() -> Result<Qwant, EngineError> {
        Ok(Self)
    }

    /// Parses the raw JSON response body into a list of search results.
    ///
    /// # Arguments
    ///
    /// * `json` - A byte slice of the raw JSON response body.
    ///
    /// # Error
    ///
    /// It returns an `EngineError` if the API response indicates an error or if the
    /// response cannot be parsed.
    fn parse_json_response(json: &[u8]) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let response: QwantApiResponse =
            serde_json::from_slice(json).change_context(EngineError::UnexpectedError)?;

        let result = match response {
            QwantApiResponse::Success { result } => result,
            QwantApiResponse::Error {
                error_code,
                message,
            } => {
                let msg = message.first().map(|s| s.as_str()).unwrap_or("unknown");
                return Err(Report::new(EngineError::UnexpectedError)
                    .attach(format!("Qwant API error {error_code}: {msg}")));
            }
        };

        let results: Vec<_> = result
            .items
            .mainline
            .into_iter()
            .filter_map(|item| match item {
                QwantItem::Web { items } => Some(items),
                _ => None,
            })
            .flatten()
            .map(|item| {
                let search_result = SearchResult::new(
                    item.title.trim(),
                    item.url.as_str(),
                    item.desc.trim(),
                    &["qwant"],
                );
                (item.url, search_result)
            })
            .collect();

        Ok(results)
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
        // Qwant uses 0-based offset with 10 results per page.
        let count: u32 = 10;
        let offset = page.checked_mul(count).ok_or_else(|| {
            Report::new(EngineError::UnexpectedError)
                .attach("Qwant pagination overflow while computing offset")
        })?;

        // Build the API URL with properly encoded query parameters.
        let query_encoded: String = form_urlencoded::byte_serialize(query.as_bytes()).collect();
        let url = format!(
            "https://api.qwant.com/v3/search/web?q={query_encoded}&count={count}&locale=en_US&offset={offset}&safesearch={safe_search}&device=desktop&tgb=2&displayed=true"
        );

        let header_map = HeaderMap::try_from(&HashMap::from([
            ("User-Agent".to_string(), user_agent.to_string()),
            ("Referer".to_string(), "https://www.qwant.com/".to_string()),
        ]))
        .change_context(EngineError::UnexpectedError)?;

        let json_bytes =
            Qwant::fetch_json_as_bytes_from_upstream(self, &url, header_map, client).await?;

        let results = Self::parse_json_response(&json_bytes)?;

        if results.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_response_success() {
        let json = br#"{
            "status": "success",
            "data": {
                "result": {
                    "items": {
                        "mainline": [
                            {
                                "type": "web",
                                "items": [
                                    {
                                        "title": "Rust Programming Language",
                                        "url": "https://www.rust-lang.org/",
                                        "desc": "A language empowering everyone to build reliable software."
                                    }
                                ]
                            }
                        ]
                    }
                }
            }
        }"#;

        let results = Qwant::parse_json_response(json).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.title, "Rust Programming Language");
        assert_eq!(results[0].1.url, "https://www.rust-lang.org/");
        assert_eq!(
            results[0].1.description,
            "A language empowering everyone to build reliable software."
        );
        assert_eq!(results[0].1.engine, vec!["qwant"]);
    }

    #[test]
    fn test_parse_json_response_filters_non_web_items() {
        let json = br#"{
            "status": "success",
            "data": {
                "result": {
                    "items": {
                        "mainline": [
                            {
                                "type": "web",
                                "items": [
                                    {
                                        "title": "Result 1",
                                        "url": "https://example.com/1",
                                        "desc": "Description 1"
                                    }
                                ]
                            },
                            {
                                "type": "related_searches",
                                "items": []
                            },
                            {
                                "type": "web",
                                "items": [
                                    {
                                        "title": "Result 2",
                                        "url": "https://example.com/2",
                                        "desc": "Description 2"
                                    }
                                ]
                            }
                        ]
                    }
                }
            }
        }"#;

        let results = Qwant::parse_json_response(json).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_parse_json_response_error() {
        let json = br#"{
            "status": "error",
            "data": {
                "errorCode": 27,
                "message": ["Captcha required"]
            }
        }"#;

        let result = Qwant::parse_json_response(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json_response_empty_results() {
        let json = br#"{
            "status": "success",
            "data": {
                "result": {
                    "items": {
                        "mainline": []
                    }
                }
            }
        }"#;

        let results = Qwant::parse_json_response(json).unwrap();
        assert!(results.is_empty());
    }
}
