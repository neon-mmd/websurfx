//! The `sepiasearch` module handles the fetching of results from the SepiaSearch video search
//! engine (PeerTube search index) by querying its JSON API with the user provided query.

use reqwest::{Client, header::HeaderMap};
use serde::Deserialize;
use std::collections::HashMap;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, EngineResult, SearchEngine};
use error_stack::{Report, ResultExt};

/// A new SepiaSearch engine type defined in-order to implement the `SearchEngine` trait.
pub struct SepiaSearch;

/// The JSON response structure returned by the SepiaSearch API.
#[derive(Deserialize)]
struct SepiaSearchResponse {
    /// The list of video results returned by the API.
    data: Vec<SepiaSearchVideo>,
    /// The total number of results available.
    total: Option<u32>,
    /// An error message, if the API returned one.
    error: Option<String>,
}

/// A single video result from the SepiaSearch API.
#[derive(Deserialize)]
struct SepiaSearchVideo {
    /// The title of the video.
    name: String,
    /// The URL to watch the video.
    url: String,
    /// An optional description of the video.
    description: Option<String>,
}

impl SepiaSearch {
    /// Creates a new SepiaSearch engine instance.
    pub fn new() -> EngineResult<SepiaSearch> {
        Ok(Self)
    }

    /// Parses the raw JSON response body into a list of search results.
    fn parse_json_response(json: &[u8]) -> EngineResult<Vec<(String, SearchResult)>> {
        let response: SepiaSearchResponse =
            serde_json::from_slice(json).change_context(EngineError::UnexpectedError)?;

        if let Some(err) = &response.error {
            return Err(Report::new(EngineError::UnexpectedError)
                .attach(format!("SepiaSearch API error: {err}")));
        }

        let results = response
            .data
            .into_iter()
            .map(|video| {
                let description = video.description.unwrap_or_default().trim().to_string();
                let search_result = SearchResult::new(
                    video.name.trim(),
                    video.url.as_str(),
                    description.as_str(),
                    &["sepiasearch"],
                );
                (search_result.url.clone(), search_result)
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

        let header_map = HeaderMap::try_from(&HashMap::from([
            ("User-Agent".to_string(), user_agent.to_string()),
            (
                "Referer".to_string(),
                "https://sepiasearch.org/".to_string(),
            ),
            (
                "Content-Type".to_string(),
                "text/html; charset=utf-8".to_string(),
            ),
        ]))
        .change_context(EngineError::UnexpectedError)?;

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

    #[test]
    fn test_parse_json_response() {
        let json = br#"{
            "total": 1,
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
            "total": 1,
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
    fn test_parse_json_response_empty_results() {
        let json = br#"{
            "total": 0,
            "data": []
        }"#;

        let results = SepiaSearch::parse_json_response(json).unwrap();
        assert!(results.is_empty());
    }
}
