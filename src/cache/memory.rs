//! This module provides the memory cache structures for enabling the use of memory caching.

use super::{
    Cacher,
    error::{CacheError, CacheResult},
};
use crate::models::aggregation::SearchResultsArchive;
use crate::parser::Config;
use moka::future::Cache as MokaCache;
use rkyv::util::AlignedVec;
use std::sync::Arc;
use tokio::time::Duration;

/// Memory based cache backend.
///
/// Entries are stored as a shared encoded buffer so a cache hit costs a refcount bump
/// instead of copying the whole result set out of the store.
pub struct InMemoryCache {
    /// The backend cache which stores data.
    cache: Arc<MokaCache<String, Arc<AlignedVec>>>,
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

        // Calculate max capacity based on expected entry size (~5KB per entry) and memory limit
        let max_entries = (100 * 1024 * 1024) / 5_000; // ~20K entries for 100MB

        InMemoryCache {
            cache: Arc::new(
                MokaCache::builder()
                    .max_capacity(max_entries)
                    .time_to_live(Duration::from_secs(config.cache_expiry_time.into()))
                    .weigher(|_key: &String, value: &Arc<AlignedVec>| value.len() as u32)
                    .build(),
            ),
        }
    }

    async fn cached_results_exists(&self, urls: &[String]) -> CacheResult<Vec<bool>> {
        Ok(urls
            .iter()
            .map(|url| self.cache.contains_key(url))
            .collect())
    }

    async fn cached_results(&self, url: &str) -> CacheResult<SearchResultsArchive> {
        if let Some(res) = self.cache.get(url).await {
            return self.post_process_search_results(res).await;
        }

        return Err(CacheError::MissingValue);
    }

    async fn cache_results(
        &self,
        search_results: &[SearchResultsArchive],
        urls: &[String],
    ) -> CacheResult<()> {
        for (hashed_url_string, search_result) in urls.iter().zip(search_results.iter()) {
            let bytes = self.pre_process_search_results(search_result).await?;
            self.cache.insert(hashed_url_string.clone(), bytes).await;
        }

        Ok(())
    }
}
