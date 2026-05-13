//! This module provides the memory cache structures for enabling the use of memory caching.

use super::{
    Cacher,
    error::{CacheError, CacheResult},
};
use crate::models::aggregation::SearchResults;
use crate::parser::Config;
use futures::future::join_all;
use moka::future::Cache as MokaCache;
use std::sync::Arc;
use tokio::time::Duration;

/// Memory based cache backend.
pub struct InMemoryCache {
    /// The backend cache which stores data.
    cache: Arc<MokaCache<String, Vec<u8>>>,
}

impl Clone for InMemoryCache {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
        }
    }
}

#[async_trait::async_trait]
impl Cacher for InMemoryCache {
    async fn build(config: &Config) -> Self {
        log::info!("Initialising in-memory cache");

        InMemoryCache {
            cache: Arc::new(
                MokaCache::builder()
                    .time_to_live(Duration::from_secs(config.cache_expiry_time.into()))
                    .build(),
            ),
        }
    }

    async fn cached_results_exists(&mut self, urls: &[String]) -> CacheResult<Vec<bool>> {
        Ok(urls
            .iter()
            .map(|url| self.cache.contains_key(url))
            .collect())
    }

    async fn cached_results(&mut self, url: &str) -> CacheResult<SearchResults> {
        if let Some(res) = self.cache.get(url).await {
            return self.post_process_search_results(res).await;
        }

        return Err(CacheError::MissingValue);
    }

    async fn cache_results(
        &mut self,
        search_results: &[SearchResults],
        urls: &[String],
    ) -> CacheResult<()> {
        let mut tasks: Vec<_> = Vec::with_capacity(urls.len());

        for (hashed_url_string, search_result) in urls.iter().cloned().zip(search_results.iter()) {
            let bytes = self.pre_process_search_results(search_result).await?;
            let new_self = self.clone();
            tasks.push(tokio::spawn(async move {
                new_self.cache.insert(hashed_url_string, bytes).await
            }));
        }

        join_all(tasks).await;

        Ok(())
    }
}
