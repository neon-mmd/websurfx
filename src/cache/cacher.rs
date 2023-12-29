//! This module provides the functionality to cache the aggregated results fetched and aggregated
//! from the upstream search engines in a json format.

use error_stack::Report;
#[cfg(feature = "memory-cache")]
use mini_moka::sync::Cache as MokaCache;

#[cfg(feature = "memory-cache")]
use std::time::Duration;
use tokio::sync::Mutex;

use crate::{config::parser::Config, models::aggregation_models::SearchResults};

use super::error::CacheError;
#[cfg(feature = "redis-cache")]
use super::redis_cacher::RedisCache;

/// Abstraction trait for common methods provided by a cache backend.
#[async_trait::async_trait]
pub trait Cacher: Send + Sync {
    // A function that builds the cache from the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - It takes the config struct as an argument.
    ///
    /// # Returns
    ///
    /// It returns a newly initialized backend based on the feature enabled by the user.
    async fn build(config: &Config) -> Self
    where
        Self: Sized;

    /// A function which fetches the cached json results as json string.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes an url as a string.
    ///
    /// # Error
    ///
    /// Returns the `SearchResults` from the cache if the program executes normally otherwise
    /// returns a `CacheError` if the results cannot be retrieved from the cache.
    async fn cached_results(&mut self, url: &str) -> Result<SearchResults, Report<CacheError>>;

    /// A function which caches the results by using the `url` as the key and
    /// `json results` as the value and stores it in the cache
    ///
    /// # Arguments
    ///
    /// * `json_results` - It takes the json results string as an argument.
    /// * `url` - It takes the url as a String.
    ///
    /// # Error
    ///
    /// Returns a unit type if the program caches the given search results without a failure
    /// otherwise it returns a `CacheError` if the search results cannot be cached due to a
    /// failure.
    async fn cache_results(
        &mut self,
        search_results: &SearchResults,
        url: &str,
    ) -> Result<(), Report<CacheError>>;

    /// A helper function which computes the hash of the url and formats and returns it as string.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes an url as string.
    fn hash_url(&self, url: &str) -> String {
        blake3::hash(url.as_bytes()).to_string()
    }
    
    /// A helper function that returns either compressed or encryption results
    /// Feature flags are required  for this to work
    ///
    /// # Arguments 
    /// 
    /// * `search_results` - It takes the json search results string as an argument.
    /// * `url` - It takes the url of the queried pages as an argument.
    ///
    /// # Error
    /// Returns the compressed or encrypted bytes on success otherwise it returns a CacheError
    /// on failure.
    #[cfg(any(
        feature = "compress-cache-results",
        feature = "encrypted-cache-results",
        feature = "cec-cache-results"
    ))]
    async fn compress_encrypt_results(
        &mut self,
        search_results: &str,
        url: &str,
    ) -> Result<Vec<u8>, Report<CacheError>> {
        let mut bytes = search_results.as_bytes();
        
        #[cfg(feature = "compress-cache-results")]
        {
	        use std::io::Write;
	        let mut writer = brotli::CompressorWriter::new(Vec::new(), 4096, 11, 22);
	        writer.write_all(&bytes);
	        bytes = writer.into_inner();
	        Ok(bytes)
        }
        #[cfg(feature = "encrypt-cache-results")]
        {
	        use chacha20poly1305::{
	            aead::{Aead, AeadCore, KeyInit, OsRng},
	            ChaCha20Poly1305,
	        };
	
            let cipher = CIPHER.get_or_init(|| {
                let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                ChaCha20Poly1305::new(&key)
            });
		
            let encryption_key = ENCRYPTION_KEY.get_or_init(
                || ChaCha20Poly1305::generate_nonce(&mut OsRng), // 96-bits; unique per message
            );
	
	        bytes = cipher
	            .encrypt(&encryption_key, bytes.as_ref())
	            .map_err(|_| CacheError::EncryptionError)?;
	        Ok(bytes)
        }
    }
}

#[cfg(feature = "redis-cache")]
#[async_trait::async_trait]
impl Cacher for RedisCache {
    async fn build(config: &Config) -> Self {
        log::info!(
            "Initialising redis cache. Listening to {}",
            &config.redis_url
        );
        RedisCache::new(&config.redis_url, 5, config.cache_expiry_time)
            .await
            .expect("Redis cache configured")
    }

    async fn cached_results(&mut self, url: &str) -> Result<SearchResults, Report<CacheError>> {
        let hashed_url_string: &str = &self.hash_url(url);
        let json = self.cached_json(hashed_url_string).await?;
        Ok(serde_json::from_str::<SearchResults>(&json)
            .map_err(|_| CacheError::SerializationError)?)
    }

    async fn cache_results(
        &mut self,
        search_results: &SearchResults,
        url: &str,
    ) -> Result<(), Report<CacheError>> {
        let json =
            serde_json::to_string(search_results).map_err(|_| CacheError::SerializationError)?;
        let hashed_url_string = self.hash_url(url);
        self.cache_json(&json, &hashed_url_string).await
    }
}

/// Memory based cache backend.
#[cfg(feature = "memory-cache")]
pub struct InMemoryCache {
    /// The backend cache which stores data.
    cache: MokaCache<String, SearchResults>,
}

#[cfg(feature = "memory-cache")]
#[async_trait::async_trait]
impl Cacher for InMemoryCache {
    async fn build(config: &Config) -> Self {
        log::info!("Initialising in-memory cache");

        InMemoryCache {
            cache: MokaCache::builder()
                .time_to_live(Duration::from_secs(config.cache_expiry_time.into()))
                .build(),
        }
    }

    async fn cached_results(&mut self, url: &str) -> Result<SearchResults, Report<CacheError>> {
        let hashed_url_string = self.hash_url(url);
        match self.cache.get(&hashed_url_string) {
            Some(res) => Ok(res),
            None => Err(Report::new(CacheError::MissingValue)),
        }
    }

    async fn cache_results(
        &mut self,
        search_results: &SearchResults,
        url: &str,
    ) -> Result<(), Report<CacheError>> {
        let hashed_url_string = self.hash_url(url);
        self.cache.insert(hashed_url_string, search_results.clone());
        Ok(())
    }
}

/// Cache backend which utilises both memory and redis based caches.
///
/// The hybrid cache system uses both the types of cache to ensure maximum availability.
/// The set method sets the key, value pair in both the caches. Therefore in a case where redis
/// cache becomes unavailable, the backend will retreive the value from in-memory cache.
#[cfg(all(feature = "memory-cache", feature = "redis-cache"))]
pub struct HybridCache {
    /// The in-memory backend cache which stores data.
    memory_cache: InMemoryCache,
    /// The redis backend cache which stores data.
    redis_cache: RedisCache,
}

#[cfg(all(feature = "memory-cache", feature = "redis-cache"))]
#[async_trait::async_trait]
impl Cacher for HybridCache {
    async fn build(config: &Config) -> Self {
        log::info!("Initialising hybrid cache");
        HybridCache {
            memory_cache: InMemoryCache::build(config).await,
            redis_cache: RedisCache::build(config).await,
        }
    }

    async fn cached_results(&mut self, url: &str) -> Result<SearchResults, Report<CacheError>> {
        match self.redis_cache.cached_results(url).await {
            Ok(res) => Ok(res),
            Err(_) => self.memory_cache.cached_results(url).await,
        }
    }

    async fn cache_results(
        &mut self,
        search_results: &SearchResults,
        url: &str,
    ) -> Result<(), Report<CacheError>> {
        self.redis_cache.cache_results(search_results, url).await?;
        self.memory_cache.cache_results(search_results, url).await?;

        Ok(())
    }
}

/// Dummy cache backend
pub struct DisabledCache;

#[async_trait::async_trait]
impl Cacher for DisabledCache {
    async fn build(_config: &Config) -> Self {
        log::info!("Caching is disabled");
        DisabledCache
    }

    async fn cached_results(&mut self, _url: &str) -> Result<SearchResults, Report<CacheError>> {
        Err(Report::new(CacheError::MissingValue))
    }

    async fn cache_results(
        &mut self,
        _search_results: &SearchResults,
        _url: &str,
    ) -> Result<(), Report<CacheError>> {
        Ok(())
    }
}

/// A structure to efficiently share the cache between threads - as it is protected by a Mutex.
pub struct SharedCache {
    /// The internal cache protected from concurrent access by a mutex
    cache: Mutex<Box<dyn Cacher>>,
}

impl SharedCache {
    /// A function that creates a new `SharedCache` from a Cache implementation.
    ///
    /// # Arguments
    ///
    /// * `cache` - It takes the `Cache` enum variant as an argument with the prefered cache type.
    ///
    /// Returns a newly constructed `SharedCache` struct.
    pub fn new(cache: impl Cacher + 'static) -> Self {
        Self {
            cache: Mutex::new(Box::new(cache)),
        }
    }

    /// A getter function which retrieves the cached SearchResulsts from the internal cache.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes the search url as an argument which will be used as the key to fetch the
    /// cached results from the cache.
    ///
    /// # Error
    ///
    /// Returns a `SearchResults` struct containing the search results from the cache if nothing
    /// goes wrong otherwise returns a `CacheError`.
    pub async fn cached_results(&self, url: &str) -> Result<SearchResults, Report<CacheError>> {
        let mut mut_cache = self.cache.lock().await;
        mut_cache.cached_results(url).await
    }

    /// A setter function which caches the results by using the `url` as the key and
    /// `SearchResults` as the value.
    ///
    /// # Arguments
    ///
    /// * `search_results` - It takes the `SearchResults` as an argument which are results that
    /// needs to be cached.
    /// * `url` - It takes the search url as an argument which will be used as the key for storing
    /// results in the cache.
    ///
    /// # Error
    ///
    /// Returns an unit type if the results are cached succesfully otherwise returns a `CacheError`
    /// on a failure.
    pub async fn cache_results(
        &self,
        search_results: &SearchResults,
        url: &str,
    ) -> Result<(), Report<CacheError>> {
        let mut mut_cache = self.cache.lock().await;
        mut_cache.cache_results(search_results, url).await
    }
}

/// A function to initialise the cache backend.
pub async fn create_cache(config: &Config) -> impl Cacher {
    #[cfg(all(feature = "redis-cache", feature = "memory-cache"))]
    return HybridCache::build(config).await;

    #[cfg(all(feature = "memory-cache", not(feature = "redis-cache")))]
    return InMemoryCache::build(config).await;

    #[cfg(all(feature = "redis-cache", not(feature = "memory-cache")))]
    return RedisCache::build(config).await;

    #[cfg(not(any(feature = "memory-cache", feature = "redis-cache")))]
    return DisabledCache::build(config).await;
}

//#[cfg(feature = "Compress-cache-results")]
