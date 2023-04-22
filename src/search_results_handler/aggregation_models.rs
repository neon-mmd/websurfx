use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub title: String,
    pub visiting_url: String,
    pub url: String,
    pub description: String,
    pub engine: Vec<String>,
}

pub struct RawSearchResult {
    pub title: String,
    pub visiting_url: String,
    pub description: String,
    pub engine: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub page_query: String,
}
