//! This module provides the functionality to cache the aggregated results fetched and aggregated
//! from the upstream search engines in a json format.

use error_stack::Report;
use futures::future::join_all;
#[cfg(feature = "memory-cache")]
use moka::future::Cache as MokaCache;

#[cfg(feature = "memory-cache")]
use std::time::Duration;
use tokio::sync::Mutex;

use crate::{config::parser::Config, models::aggregation_models::SearchResults};

use super::error::CacheError;
#[cfg(feature = "redis-cache")]
use super::redis_cacher::RedisCache;

#[cfg(any(feature = "encrypt-cache-results", feature = "cec-cache-results"))]
use super::encryption::*;

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
        search_results: &[SearchResults],
        urls: &[String],
    ) -> Result<(), Report<CacheError>>;

    /// A helper function which computes the hash of the url and formats and returns it as string.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes an url as string.
    fn hash_url(&self, url: &str) -> String {
        blake3::hash(url.as_bytes()).to_string()
    }

    /// A helper function that returns  either encrypted or decrypted results.
    ///  Feature flags (**encrypt-cache-results or cec-cache-results**) are required  for this to work.
    ///
    /// # Arguments
    ///
    /// * `bytes` - It takes a slice of bytes as an argument.
    /// * `encrypt` - A boolean to choose whether to encrypt or decrypt the bytes

    ///
    /// # Error
    /// Returns  either encrypted or decrypted bytes on success otherwise it returns a CacheError
    /// on failure.
    #[cfg(any(
      //  feature = "compress-cache-results",
        feature = "encrypt-cache-results",
        feature = "cec-cache-results"
    ))]
    async fn encrypt_or_decrypt_results(
        &mut self,
        mut bytes: Vec<u8>,
        encrypt: bool,
    ) -> Result<Vec<u8>, Report<CacheError>> {
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

        bytes = if encrypt {
            cipher
                .encrypt(encryption_key, bytes.as_ref())
                .map_err(|_| CacheError::EncryptionError)?
        } else {
            cipher
                .decrypt(encryption_key, bytes.as_ref())
                .map_err(|_| CacheError::EncryptionError)?
        };

        Ok(bytes)
    }

    /// A helper function that returns compressed results.
    /// Feature flags (**compress-cache-results or cec-cache-results**) are required  for this to work.
    ///
    /// # Arguments
    ///
    /// * `bytes` - It takes a slice of bytes as an argument.

    ///
    /// # Error
    /// Returns the compressed bytes on success otherwise it returns a CacheError
    /// on failure.
    #[cfg(any(feature = "compress-cache-results", feature = "cec-cache-results"))]
    async fn compress_results(
        &mut self,
        mut bytes: Vec<u8>,
    ) -> Result<Vec<u8>, Report<CacheError>> {
        use tokio::io::AsyncWriteExt;
        let mut writer = async_compression::tokio::write::BrotliEncoder::new(Vec::new());
        writer
            .write_all(&bytes)
            .await
            .map_err(|_| CacheError::CompressionError)?;
        writer
            .shutdown()
            .await
            .map_err(|_| CacheError::CompressionError)?;
        bytes = writer.into_inner();
        Ok(bytes)
    }

    /// A helper function that returns compressed-encrypted results.
    /// Feature flag (**cec-cache-results**) is required  for this to work.
    ///
    /// # Arguments
    ///
    /// * `bytes` - It takes a slice of bytes as an argument.

    ///
    /// # Error
    /// Returns the compressed and encrypted bytes on success otherwise it returns a CacheError
    /// on failure.
    #[cfg(feature = "cec-cache-results")]
    async fn compress_encrypt_compress_results(
        &mut self,
        mut bytes: Vec<u8>,
    ) -> Result<Vec<u8>, Report<CacheError>> {
        // compress first
        bytes = self.compress_results(bytes).await?;
        // encrypt
        bytes = self.encrypt_or_decrypt_results(bytes, true).await?;

        // compress again;
        bytes = self.compress_results(bytes).await?;

        Ok(bytes)
    }

    /// A helper function that returns compressed results.
    /// Feature flags (**compress-cache-results or cec-cache-results**) are required  for this to work.
    /// If bytes where
    /// # Arguments
    ///
    /// * `bytes` - It takes a slice of bytes as an argument.

    ///
    /// # Error
    /// Returns the uncompressed bytes on success otherwise it returns a CacheError
    /// on failure.

    #[cfg(any(feature = "compress-cache-results", feature = "cec-cache-results"))]
    async fn decompress_results(&mut self, bytes: &[u8]) -> Result<Vec<u8>, Report<CacheError>> {
        cfg_if::cfg_if! {
             if #[cfg(feature = "compress-cache-results")]
            {
               decompress_util(bytes).await

            }
            else if  #[cfg(feature = "cec-cache-results")]
            {
                let decompressed = decompress_util(bytes)?;
                let decrypted = self.encrypt_or_decrypt_results(decompressed, false)?;

                decompress_util(&decrypted).await

            }
        }
    }

    /// A helper function that compresses or encrypts search results before they're inserted into a cache store

    /// # Arguments
    ///
    /// * `search_results` - A reference to the search_Results to process.
    ///

    ///
    /// # Error
    /// Returns a Vec of compressed or encrypted bytes on success otherwise it returns a CacheError
    /// on failure.
    async fn pre_process_search_results(
        &mut self,
        search_results: &SearchResults,
    ) -> Result<Vec<u8>, Report<CacheError>> {
        #[allow(unused_mut)] // needs to be mutable when any of the features is enabled
        let mut bytes: Vec<u8> = search_results.try_into()?;
        #[cfg(feature = "compress-cache-results")]
        {
            let compressed = self.compress_results(bytes).await?;
            bytes = compressed;
        }

        #[cfg(feature = "encrypt-cache-results")]
        {
            let encrypted = self.encrypt_or_decrypt_results(bytes, true).await?;
            bytes = encrypted;
        }

        #[cfg(feature = "cec-cache-results")]
        {
            let compressed_encrypted_compressed =
                self.compress_encrypt_compress_results(bytes).await?;
            bytes = compressed_encrypted_compressed;
        }

        Ok(bytes)
    }

    /// A helper function that decompresses or decrypts search results after they're fetched from the cache-store

    /// # Arguments
    ///
    /// * `bytes` - A Vec of bytes stores in the cache.
    ///

    ///
    /// # Error
    /// Returns the SearchResults struct on success otherwise it returns a CacheError
    /// on failure.

    #[allow(unused_mut)] // needs to be mutable when any of the features is enabled
    async fn post_process_search_results(
        &mut self,
        mut bytes: Vec<u8>,
    ) -> Result<SearchResults, Report<CacheError>> {
        #[cfg(feature = "compress-cache-results")]
        {
            let decompressed = self.decompress_results(&bytes).await?;
            bytes = decompressed
        }

        #[cfg(feature = "encrypt-cache-results")]
        {
            let decrypted = self.encrypt_or_decrypt_results(bytes, false).await?;
            bytes = decrypted
        }

        #[cfg(feature = "cec-cache-results")]
        {
            let decompressed_decrypted = self.decompress_results(&bytes).await?;
            bytes = decompressed_decrypted;
        }

        Ok(bytes.try_into()?)
    }
}

/// A helper function that returns compressed results.
/// Feature flags (**compress-cache-results or cec-cache-results**) are required  for this to work.
/// If bytes where
/// # Arguments
///
/// * `bytes` - It takes a slice of bytes as an argument.

///
/// # Error
/// Returns the uncompressed bytes on success otherwise it returns a CacheError
/// on failure.

#[cfg(any(feature = "compress-cache-results", feature = "cec-cache-results"))]
async fn decompress_util(input: &[u8]) -> Result<Vec<u8>, Report<CacheError>> {
    use tokio::io::AsyncWriteExt;
    let mut writer = async_compression::tokio::write::BrotliDecoder::new(Vec::new());

    writer
        .write_all(input)
        .await
        .map_err(|_| CacheError::CompressionError)?;
    writer
        .shutdown()
        .await
        .map_err(|_| CacheError::CompressionError)?;
    let bytes = writer.into_inner();
    Ok(bytes)
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
        use base64::Engine;
        let hashed_url_string: &str = &self.hash_url(url);
        let base64_string = self.cached_json(hashed_url_string).await?;

        let bytes = base64::engine::general_purpose::STANDARD_NO_PAD
            .decode(base64_string)
            .map_err(|_| CacheError::Base64DecodingOrEncodingError)?;
        self.post_process_search_results(bytes).await
    }

    async fn cache_results(
        &mut self,
        search_results: &[SearchResults],
        urls: &[String],
    ) -> Result<(), Report<CacheError>> {
        use base64::Engine;

        // size of search_results is expected to be equal to size of urls -> key/value pairs  for cache;
        let search_results_len = search_results.len();

        let mut bytes = Vec::with_capacity(search_results_len);

        for result in search_results {
            let processed = self.pre_process_search_results(result).await?;
            bytes.push(processed);
        }

        let base64_strings = bytes
            .iter()
            .map(|bytes_vec| base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes_vec));

        let mut hashed_url_strings = Vec::with_capacity(search_results_len);

        for url in urls {
            let hash = self.hash_url(url);
            hashed_url_strings.push(hash);
        }
        self.cache_json(base64_strings, hashed_url_strings.into_iter())
            .await
    }
}
/// TryInto implementation for SearchResults from Vec<u8>
use std::{convert::TryInto, sync::Arc};

impl TryInto<SearchResults> for Vec<u8> {
    type Error = CacheError;

    fn try_into(self) -> Result<SearchResults, Self::Error> {
        bincode::deserialize_from(self.as_slice()).map_err(|_| CacheError::SerializationError)
    }
}

impl TryInto<Vec<u8>> for &SearchResults {
    type Error = CacheError;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        bincode::serialize(self).map_err(|_| CacheError::SerializationError)
    }
}

/// Memory based cache backend.
#[cfg(feature = "memory-cache")]
pub struct InMemoryCache {
    /// The backend cache which stores data.
    cache: Arc<MokaCache<String, Vec<u8>>>,
}

#[cfg(feature = "memory-cache")]
impl Clone for InMemoryCache {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
        }
    }
}

#[cfg(feature = "memory-cache")]
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

    async fn cached_results(&mut self, url: &str) -> Result<SearchResults, Report<CacheError>> {
        let hashed_url_string = self.hash_url(url);
        match self.cache.get(&hashed_url_string).await {
            Some(res) => self.post_process_search_results(res).await,
            None => Err(Report::new(CacheError::MissingValue)),
        }
    }

    async fn cache_results(
        &mut self,
        search_results: &[SearchResults],
        urls: &[String],
    ) -> Result<(), Report<CacheError>> {
        let mut tasks: Vec<_> = Vec::with_capacity(urls.len());
        for (url, search_result) in urls.iter().zip(search_results.iter()) {
            let hashed_url_string = self.hash_url(url);
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
        search_results: &[SearchResults],
        urls: &[String],
    ) -> Result<(), Report<CacheError>> {
        self.redis_cache.cache_results(search_results, urls).await?;
        self.memory_cache
            .cache_results(search_results, urls)
            .await?;

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
        _search_results: &[SearchResults],
        _urls: &[String],
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
    ///   cached results from the cache.
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
    ///   needs to be cached.
    /// * `url` - It takes the search url as an argument which will be used as the key for storing
    ///   results in the cache.
    ///
    /// # Error
    ///
    /// Returns an unit type if the results are cached succesfully otherwise returns a `CacheError`
    /// on a failure.
    pub async fn cache_results(
        &self,
        search_results: &[SearchResults],
        urls: &[String],
    ) -> Result<(), Report<CacheError>> {
        let mut mut_cache = self.cache.lock().await;
        mut_cache.cache_results(search_results, urls).await
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
