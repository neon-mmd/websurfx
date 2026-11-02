//! The `sepiasearch` module handles the fetching of results from the SepiaSearch video search
//! engine (PeerTube search index) by querying its JSON API with the user provided query.

use crate::json::{self, JsonValue};
use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, EngineResult, SearchEngine};
use error_stack::{Report, ResultExt};
use reqwest::Client;

/// A new SepiaSearch engine type defined in-order to implement the `SearchEngine` trait.
pub struct SepiaSearch;

impl SepiaSearch {
    /// Creates a new SepiaSearch engine instance.
    pub fn new() -> EngineResult<SepiaSearch> {
        Ok(Self)
    }

    /// Parses the raw JSON response body into a list of search results.
    ///
    /// Strings are borrowed out of `json` while the list is being built, so only the fields
    /// that end up in a [`SearchResult`] are ever copied.
    ///
    /// # Arguments
    ///
    /// * `json` - It takes the raw bytes of the response body.
    ///
    /// # Error
    ///
    /// Returns an `EngineError` if the body is not valid JSON or carries an API error.
    fn parse_json_response(json: &[u8]) -> EngineResult<Vec<(String, SearchResult)>> {
        let document = json::parse(json).change_context(EngineError::UnexpectedError)?;

        if let Some(message) = document.get("error").and_then(JsonValue::as_str) {
            return Err(Report::new(EngineError::UnexpectedError)
                .attach(format!("SepiaSearch API error: {message}")));
        }

        let results = document
            .get("data")
            .and_then(JsonValue::as_array)
            .unwrap_or_default()
            .iter()
            .filter_map(|video| {
                let name = video.get("name").and_then(JsonValue::as_str)?;
                let url = video.get("url").and_then(JsonValue::as_str)?;
                let description = video
                    .get("description")
                    .and_then(JsonValue::as_str)
                    .unwrap_or_default();

                let search_result =
                    SearchResult::new(name.trim(), url, description.trim(), &["sepiasearch"]);
                Some((search_result.url.clone(), search_result))
            })
            .collect();

        Ok(results)
    }
}

#[async_trait::async_trait]
impl SearchEngine for SepiaSearch {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        safe_search: u8,
    ) -> EngineResult<Vec<(String, SearchResult)>> {
        let nsfw = if safe_search == 0 { "both" } else { "false" };

        // Pagination: 0-based offset, 10 results per page
        let start = page * 10;

        let encoded_query = form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
        let url = format!(
            "https://sepiasearch.org/api/v1/search/videos?search={encoded_query}&start={start}&count=10&sort=-match&nsfw={nsfw}"
        );

        let header_map = crate::models::engine::build_headers(&[
            ("User-Agent", user_agent),
            ("Referer", "https://sepiasearch.org/"),
            ("Content-Type", "text/html; charset=utf-8"),
        ])?;

        let json_bytes =
            SepiaSearch::fetch_json_as_bytes_from_upstream(self, &url, header_map, client).await?;

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
    use crate::json::JsonValue;

    #[test]
    fn test_parse_json_response() {
        let json = br#"{
            "data": [
                {
                    "name": "Test Video",
                    "url": "https://video.example.org/videos/watch/abc123",
                    "description": "A test video description"
                }
            ]
        }"#;

        let results = SepiaSearch::parse_json_response(json).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.title, "Test Video");
        assert_eq!(
            results[0].1.url,
            "https://video.example.org/videos/watch/abc123"
        );
        assert_eq!(results[0].1.description, "A test video description");
        assert_eq!(results[0].1.engine, vec!["sepiasearch"]);
    }

    #[test]
    fn test_parse_json_response_no_description() {
        let json = br#"{
            "data": [
                {
                    "name": "No Desc Video",
                    "url": "https://video.example.org/videos/watch/def456"
                }
            ]
        }"#;

        let results = SepiaSearch::parse_json_response(json).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.description, "");
    }

    #[test]
    fn test_parse_json_response_null_description() {
        let json = br#"{
            "data": [
                {
                    "name": "Null Desc Video",
                    "url": "https://video.example.org/videos/watch/ghi789",
                    "description": null
                }
            ]
        }"#;

        let results = SepiaSearch::parse_json_response(json).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.description, "");
    }

    #[test]
    fn test_parse_json_response_empty_results() {
        let json = br#"{
            "data": []
        }"#;

        let results = SepiaSearch::parse_json_response(json).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_json_response_rejects_an_api_error() {
        let json = br#"{"error": "Instance unreachable", "data": []}"#;

        assert!(SepiaSearch::parse_json_response(json).is_err());
    }

    #[test]
    fn test_parse_json_response_rejects_a_malformed_body() {
        assert!(SepiaSearch::parse_json_response(b"not json").is_err());
        assert!(SepiaSearch::parse_json_response(br#"{"data": [}"#).is_err());
    }

    #[test]
    fn test_parse_json_response_borrows_unescaped_strings() {
        let json = br#"{
            "data": [
                {
                    "name": "Borrowed Video",
                    "url": "https://video.example.org/videos/watch/abc123",
                    "description": "Borrowed description"
                }
            ]
        }"#;

        let document = json::parse(json).unwrap();
        let video = document.get("data").and_then(JsonValue::as_array).unwrap()[0].clone();

        let body = json.as_ptr() as usize;
        for field in ["name", "url", "description"] {
            let text = video.get(field).and_then(JsonValue::as_str).unwrap();
            let start = text.as_ptr() as usize;

            assert!(
                start >= body && start + text.len() <= body + json.len(),
                "`{field}` must borrow straight out of the response body"
            );
        }
    }

    #[test]
    fn test_parse_json_response_falls_back_for_escaped_strings() {
        let json = br#"{
            "data": [
                {
                    "name": "Quote \" and newline \n",
                    "url": "https://video.example.org/videos/watch/abc123",
                    "description": "Escaped \" description"
                }
            ]
        }"#;

        let results = SepiaSearch::parse_json_response(json).unwrap();

        assert_eq!(results[0].1.title, "Quote \" and newline");
        assert_eq!(results[0].1.description, "Escaped \" description");
    }
}
