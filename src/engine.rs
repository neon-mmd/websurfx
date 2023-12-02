use std::{collections::HashMap, sync::Arc};

use crate::models::{
    aggregation_models::SearchResult,
    client_models::HttpClient,
    engine_models::{EngineError, EngineErrorType, SearchEngine},
};
use actix_web::rt::spawn;
use error_stack::Report;
use error_stack::Result;

pub struct EngineHandler {
    engines: Arc<Vec<Arc<Box<dyn SearchEngine>>>>,
    client: Arc<HttpClient>,
}

pub type RawResults = Vec<Result<HashMap<String, SearchResult>, EngineError>>;

impl EngineHandler {
    /// Parses names of engines and initialises them for use.
    ///
    /// # Arguments
    ///
    /// * `engine_names - It takes the names of the engines.
    ///
    /// # Returns
    ///
    /// It returns an option either containing the value or a none if the engine is unknown
    pub fn new(engine_names: Vec<String>, client: HttpClient) -> Result<Self, EngineError> {
        let mut engines: Vec<Arc<Box<dyn SearchEngine>>> = vec![];

        for engine_name in engine_names {
            let engine: Box<dyn SearchEngine> = match engine_name.to_lowercase().as_str() {
                "duckduckgo" => Box::new(crate::engines::duckduckgo::DuckDuckGo::new()?),
                "searx" => Box::new(crate::engines::searx::Searx::new()?),
                "brave" => Box::new(crate::engines::brave::Brave::new()?),
                _ => {
                    return Err(Report::from(EngineError {
                        error_type: EngineErrorType::NoSuchEngineFound,
                        engine: engine_name.to_string(),
                    }))
                }
            };
            engines.push(Arc::new(engine));
        }

        Ok(Self {
            engines: Arc::new(engines),
            client: Arc::new(client)
        })
    }

    pub async fn search(
        &self,
        engine_names: Option<Vec<String>>,
        query: &str,
        // category: QueryType,
        // query_relevance: Option<QueryRelavancy>,
        page: u32,
        safe_search: u8,
    ) -> RawResults {
        let mut tasks = Vec::with_capacity(self.engines.len());
        for engine in &*self.engines {
            if let Some(ref engine_names) = engine_names {
                // TODO: Handle invalid engine names provided by the user, currently it silently ignores
                if !engine_names.contains(&engine.get_name().to_owned()) {
                    continue;
                }
            }
            let engine = engine.clone();
            // let query_relevance = query_relevance.clone();
            let client = self.client.clone();
            let query = query.to_owned();
            tasks.push(spawn(async move {
                engine
                    .fetch_results(&query, page, client, safe_search)
                    .await
            }));
        }

        let mut responses = Vec::with_capacity(tasks.len());
        for task in tasks {
            // An error will only be raised when the task panics, here it should technically never panic
            if let Ok(result) = task.await {
                responses.push(result);
            }
        }

        responses
    }
}
