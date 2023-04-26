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

impl SearchResult {
    pub fn new(
        title: String,
        visiting_url: String,
        url: String,
        description: String,
        engine: Vec<String>,
    ) -> Self {
        SearchResult {
            title,
            visiting_url,
            url,
            description,
            engine,
        }
    }
}

#[derive(Clone)]
pub struct RawSearchResult {
    pub title: String,
    pub visiting_url: String,
    pub description: String,
    pub engine: Vec<String>,
}

impl RawSearchResult {
    pub fn new(
        title: String,
        visiting_url: String,
        description: String,
        engine: Vec<String>,
    ) -> Self {
        RawSearchResult {
            title,
            visiting_url,
            description,
            engine,
        }
    }
    pub fn add_engines(&mut self, engine: String) {
        self.engine.push(engine)
    }

    pub fn engine(self) -> String {
        self.engine.get(0).unwrap().to_string()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub page_query: String,
}

impl SearchResults {
    pub fn new(results: Vec<SearchResult>, page_query: String) -> Self {
        SearchResults {
            results,
            page_query,
        }
    }
}
