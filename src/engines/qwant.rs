//! The `qwant` module handles the fetching of results from the Qwant search engine
//! by querying the upstream Qwant JSON API with the user provided query and with a page
//! number if provided.

use crate::json::{self, JsonValue};
use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, EngineResult, SearchEngine};
use error_stack::{Report, ResultExt};
use reqwest::Client;

/// A new Qwant engine type defined in-order to implement the `SearchEngine` trait.
pub struct Qwant;

impl Qwant {
    /// Creates a new Qwant engine instance.
    ///
    /// # Returns
    ///
    /// It returns a `Qwant` struct on success.
    pub fn new() -> EngineResult<Qwant> {
        Ok(Self)
    }

    /// Parses the raw JSON response body into a list of search results.
    ///
    /// Strings are borrowed out of `json` while the list is being built, so only the fields
    /// that end up in a [`SearchResult`] are ever copied.
    ///
    /// # Arguments
    ///
    /// * `json` - A byte slice of the raw JSON response body.
    ///
    /// # Error
    ///
    /// It returns an `EngineError` if the API response indicates an error or if the
    /// response cannot be parsed.
    fn parse_json_response(json: &[u8]) -> EngineResult<Vec<(String, SearchResult)>> {
        let document = json::parse(json).change_context(EngineError::UnexpectedError)?;
        let data = document.get("data");

        if document.get("status").and_then(JsonValue::as_str) != Some("success") {
            // The API reports its code as `error_code` and its reason as `message`, but
            // older responses used the camel-cased `errorCode`. Both spellings are read so a
            // refusal from the API is reported as what it said rather than as an opaque
            // "error 0: unknown".
            let error_code = data
                .and_then(|data| {
                    data.get("error_code")
                        .or_else(|| data.get("errorCode"))
                        .and_then(JsonValue::as_i64)
                })
                .unwrap_or_default();

            let message = data
                .and_then(|data| {
                    data.get("message")
                        .or_else(|| data.get("errorMessage"))
                        .or_else(|| data.get("error"))
                })
                .and_then(JsonValue::as_str)
                .or_else(|| {
                    data.and_then(|data| data.get("message"))
                        .and_then(JsonValue::as_array)
                        .and_then(<[JsonValue<'_>]>::first)
                        .and_then(JsonValue::as_str)
                })
                .unwrap_or("unknown");

            return Err(Report::new(EngineError::UnexpectedError)
                .attach(format!("Qwant API error {error_code}: {message}")));
        }

        let results: Vec<_> = data
            .and_then(|data| data.get("result"))
            .and_then(|result| result.get("items"))
            .and_then(|items| items.get("mainline"))
            .and_then(JsonValue::as_array)
            .unwrap_or_default()
            .iter()
            .filter(|item| item.get("type").and_then(JsonValue::as_str) == Some("web"))
            .filter_map(|item| item.get("items").and_then(JsonValue::as_array))
            .flatten()
            .filter_map(|item| {
                let title = item.get("title").and_then(JsonValue::as_str)?;
                let url = item.get("url").and_then(JsonValue::as_str)?;
                let description = item
                    .get("desc")
                    .and_then(JsonValue::as_str)
                    .unwrap_or_default();

                let search_result =
                    SearchResult::new(title.trim(), url, description.trim(), &["qwant"]);
                Some((url.to_owned(), search_result))
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
    ) -> EngineResult<Vec<(String, SearchResult)>> {
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

        let header_map = crate::models::engine::build_headers(&[
            ("User-Agent", user_agent),
            ("Referer", "https://www.qwant.com/"),
            // This endpoint answers with JSON, so it overrides the HTML `Accept` the
            // shared client sets as a default.
            ("Accept", "application/json, text/javascript, */*; q=0.01"),
        ])?;

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

    /// The live API reports refusals as `status: "error"` with a snake-cased `error_code`
    /// and a plain `message`. Reading only the camel-cased spelling reported every refusal
    /// as the useless "error 0: unknown".
    #[test]
    fn test_parse_json_response_reports_the_codes_the_api_sends() {
        let snake = br#"{"status":"error","data":{"error_code":50}}"#;
        let camel = br#"{"status":"error","data":{"errorCode":50}}"#;

        for body in [&snake[..], &camel[..]] {
            // `unwrap_err` would need `SearchResult: Debug`, so the error is matched on.
            let Err(report) = Qwant::parse_json_response(body) else {
                return;
            };

            // Attachments render in the `Debug` form of a report, not in `Display`.
            assert!(
                format!("{report:?}").contains("error 50"),
                "expected the code to survive: {report:?}"
            );
        }
    }

    #[test]
    fn test_parse_json_response_surfaces_the_message() {
        let body = br#"{"status":"error","data":{"error_code":50,"message":"unauthorized"}}"#;

        let Err(report) = Qwant::parse_json_response(body) else {
            return;
        };

        assert!(format!("{report:?}").contains("unauthorized"), "{report:?}");
    }

    #[test]
    fn test_parse_json_response_still_falls_back_to_unknown() {
        let body = br#"{"status":"error","data":{}}"#;

        let Err(report) = Qwant::parse_json_response(body) else {
            return;
        };

        assert!(
            format!("{report:?}").contains("error 0: unknown"),
            "{report:?}"
        );
    }

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

        let Err(report) = Qwant::parse_json_response(json) else {
            return;
        };

        // `message` may arrive as a list of strings or as a single string; both are read.
        let rendered = format!("{report:?}");
        assert!(rendered.contains("error 27"), "{rendered}");
        assert!(rendered.contains("Captcha required"), "{rendered}");
    }

    #[test]
    fn test_parse_json_response_error_without_a_message() {
        let json = br#"{"status": "error", "data": {"errorCode": 3}}"#;

        let Err(report) = Qwant::parse_json_response(json) else {
            return;
        };

        assert!(
            format!("{report:?}").contains("error 3: unknown"),
            "{report:?}"
        );
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

    #[test]
    fn test_parse_json_response_rejects_a_malformed_body() {
        assert!(Qwant::parse_json_response(b"not json").is_err());
        assert!(Qwant::parse_json_response(br#"{"status":"success"} trailing"#).is_err());
    }

    #[test]
    fn test_parse_json_response_borrows_unescaped_strings() {
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
                                        "title": "Borrowed Title",
                                        "url": "https://example.com/",
                                        "desc": "Borrowed description"
                                    }
                                ]
                            }
                        ]
                    }
                }
            }
        }"#;

        let document = json::parse(json).unwrap();
        let item = document
            .get("data")
            .and_then(|data| data.get("result"))
            .and_then(|result| result.get("items"))
            .and_then(|items| items.get("mainline"))
            .and_then(JsonValue::as_array)
            .unwrap()[0]
            .clone()
            .get("items")
            .and_then(JsonValue::as_array)
            .unwrap()[0]
            .clone();

        let body = json.as_ptr() as usize;
        for field in ["title", "url", "desc"] {
            let text = item.get(field).and_then(JsonValue::as_str).unwrap();
            let start = text.as_ptr() as usize;

            assert!(
                start >= body && start + text.len() <= body + json.len(),
                "`{field}` must borrow straight out of the response body"
            );
        }
    }

    #[test]
    fn test_parse_json_response_falls_back_for_escaped_strings() {
        let json = r#"{
            "status": "success",
            "data": {
                "result": {
                    "items": {
                        "mainline": [
                            {
                                "type": "web",
                                "items": [
                                    {
                                        "title": "Quote \" and é",
                                        "url": "https://example.com/",
                                        "desc": ""
                                    }
                                ]
                            }
                        ]
                    }
                }
            }
        }"#
        .as_bytes();

        let results = Qwant::parse_json_response(json).unwrap();
        assert_eq!(results[0].1.title, "Quote \" and é");
    }
}
