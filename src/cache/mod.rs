//! This module provides the modules which provide the functionality to cache the aggregated
//! results fetched and aggregated from the upstream search engines in a json format.

use crate::{models::aggregation::SearchResultsArchive, parser::Config};
use arc_swap::ArcSwap;
use error::{CacheError, CacheResult};
use rkyv::util::AlignedVec;
use std::sync::Arc;

#[cfg(feature = "redis-cache")]
use redis::RedisCache;

#[cfg(feature = "memory-cache")]
use memory::InMemoryCache;

#[cfg(feature = "redis-cache")]
#[cfg(any(feature = "encrypt-cache-results", feature = "cec-cache-results"))]
use encryption::*;

pub mod error;

#[cfg(any(feature = "encrypt-cache-results", feature = "cec-cache-results"))]
/// encryption module contains encryption utils such the cipher and key
pub mod encryption;

#[cfg(feature = "redis-cache")]
pub mod redis;

#[cfg(feature = "memory-cache")]
pub mod memory;

/// Abstraction trait for common methods provided by a cache backend.
#[async_trait::async_trait]
trait Cacher: Send + Sync {
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
    async fn cached_results(&self, url: &str) -> CacheResult<SearchResultsArchive>;

    /// A function that checks if the cached results exists in the cache server or not.
    ///
    /// # Arguments
    ///
    /// * `urls` - It takes the hashed search urls as an argument which will be used as the key to
    ///   check whether the cache exists or not.
    ///
    /// # Error
    ///
    /// Returns a Vector containing booleans for each respective url if nothing goes wrong otherwise
    /// returns a `CacheError`.
    async fn cached_results_exists(&self, urls: &[String]) -> CacheResult<Vec<bool>>;

    /// A function which caches the results by using the `url` as the key and
    /// `json results` as the value and stores it in the cache
    ///
    /// # Arguments
    ///
    /// * `search_results` - It takes the json results string as an argument.
    /// * `url` - It takes the url as a String.
    ///
    /// # Error
    ///
    /// Returns a unit type if the program caches the given search results without a failure
    /// otherwise it returns a `CacheError` if the search results cannot be cached due to a
    /// failure.
    async fn cache_results(
        &self,
        search_results: &[SearchResultsArchive],
        urls: &[String],
    ) -> CacheResult<()>;

    /// A helper function that returns  either encrypted or decrypted results.
    ///  Feature flags (**encrypt-cache-results or cec-cache-results**) are required  for this to work.
    ///
    /// # Arguments
    ///
    /// * `bytes` - It takes a slice of bytes as an argument.
    /// * `encrypt` - A boolean to choose whether to encrypt or decrypt the bytes
    ///
    /// # Error
    ///
    /// Returns  either encrypted or decrypted bytes on success otherwise it returns a CacheError
    /// on failure.
    #[cfg(any(
      //  feature = "compress-cache-results",
        feature = "encrypt-cache-results",
        feature = "cec-cache-results"
    ))]
    async fn encrypt_or_decrypt_results(
        &self,
        mut bytes: Vec<u8>,
        encrypt: bool,
    ) -> CacheResult<Vec<u8>> {
        use chacha20poly1305::{
            ChaCha20Poly1305,
            Key,
            Nonce,
            KeyInit,
            aead::{Aead, Generate},
        };

        Ok(
            tokio::task::spawn_blocking(move || -> CacheResult<Vec<u8>> {
                let cipher = CIPHER.get_or_init(|| ChaCha20Poly1305::new(&Key::generate()));

                let encryption_key = ENCRYPTION_KEY.get_or_init(Nonce::generate);

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
            })
            .await
            .map_err(|_| CacheError::EncryptionError)??,
        )
    }

    /// A helper function that returns compressed results.
    /// Feature flags (**compress-cache-results or cec-cache-results**) are required  for this to work.
    ///
    /// # Arguments
    ///
    /// * `bytes` - It takes a slice of bytes as an argument.
    ///
    /// # Error
    ///
    /// Returns the compressed bytes on success otherwise it returns a CacheError
    /// on failure.
    #[cfg(any(feature = "compress-cache-results", feature = "cec-cache-results"))]
    async fn compress_results(&self, bytes: &[u8]) -> CacheResult<Vec<u8>> {
        use tokio::io::AsyncWriteExt;
        let mut writer = async_compression::tokio::write::BrotliEncoder::new(Vec::new());
        writer
            .write_all(bytes)
            .await
            .map_err(|_| CacheError::CompressionError)?;
        writer
            .shutdown()
            .await
            .map_err(|_| CacheError::CompressionError)?;
        Ok(writer.into_inner())
    }

    /// A helper function that expands previously compressed results.
    /// Feature flags (**compress-cache-results or cec-cache-results**) are required  for this to work.
    ///
    /// # Arguments
    ///
    /// * `bytes` - It takes a slice of bytes as an argument.
    ///
    /// # Error
    ///
    /// Returns the uncompressed bytes on success otherwise it returns a CacheError
    /// on failure.
    #[cfg(any(feature = "compress-cache-results", feature = "cec-cache-results"))]
    async fn decompress_results(&self, bytes: &[u8]) -> CacheResult<Vec<u8>> {
        decompress_util(bytes).await
    }

    /// A helper function that compresses then encrypts then compresses results again.
    /// Feature flag (**cec-cache-results**) is required  for this to work.
    ///
    /// # Arguments
    ///
    /// * `bytes` - It takes a slice of bytes as an argument.
    ///
    /// # Error
    ///
    /// Returns the compressed and encrypted bytes on success otherwise it returns a CacheError
    /// on failure.
    #[cfg(feature = "cec-cache-results")]
    async fn compress_encrypt_compress_results(&self, bytes: &[u8]) -> CacheResult<Vec<u8>> {
        let compressed = self.compress_results(bytes).await?;
        let encrypted = self.encrypt_or_decrypt_results(compressed, true).await?;
        self.compress_results(&encrypted).await
    }

    /// The exact inverse of [`Cacher::compress_encrypt_compress_results`].
    ///
    /// # Arguments
    ///
    /// * `bytes` - It takes a slice of bytes as an argument.
    ///
    /// # Error
    ///
    /// Returns the original bytes on success otherwise it returns a CacheError on failure.
    #[cfg(feature = "cec-cache-results")]
    async fn decompress_decrypt_decompress_results(&self, bytes: &[u8]) -> CacheResult<Vec<u8>> {
        let decompressed = self.decompress_results(bytes).await?;
        let decrypted = self.encrypt_or_decrypt_results(decompressed, false).await?;
        self.decompress_results(&decrypted).await
    }

    /// Runs the single compression/encryption pipeline selected by the feature flags.
    ///
    /// `forward = true` prepares bytes for storage and `forward = false` restores them. The
    /// **`cec-cache-results`** flag implies both the compress and the encrypt flag, so it is
    /// matched first to make sure exactly one pipeline is ever applied.
    ///
    /// # Arguments
    ///
    /// * `bytes` - It takes a slice of bytes as an argument.
    /// * `forward` - A boolean choosing whether to apply or to reverse the pipeline.
    ///
    /// # Error
    ///
    /// Returns the transformed bytes on success otherwise it returns a CacheError on failure.
    #[cfg(any(
        feature = "compress-cache-results",
        feature = "encrypt-cache-results",
        feature = "cec-cache-results"
    ))]
    async fn transform_bytes(&self, bytes: &[u8], forward: bool) -> CacheResult<Vec<u8>> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cec-cache-results")] {
                if forward {
                    self.compress_encrypt_compress_results(bytes).await
                } else {
                    self.decompress_decrypt_decompress_results(bytes).await
                }
            }
            else if #[cfg(feature = "compress-cache-results")] {
                if forward {
                    self.compress_results(bytes).await
                } else {
                    self.decompress_results(bytes).await
                }
            }
            else if #[cfg(feature = "encrypt-cache-results")] {
                self.encrypt_or_decrypt_results(bytes.to_vec(), forward).await
            }
            else {
                // Unreachable: this helper only exists once one of the flags above is on.
                Ok(bytes.to_vec())
            }
        }
    }

    /// Prepares an archive for insertion into a cache store.
    ///
    /// # Arguments
    ///
    /// * `search_results` - A reference to the archived search results to process.
    ///
    /// # Error
    ///
    /// Returns the encoded buffer to store on success otherwise it returns a CacheError
    /// on failure. Without any compress/encrypt feature the archive's own buffer is shared
    /// as-is, so storing a result never re-encodes it.
    async fn pre_process_search_results(
        &self,
        search_results: &SearchResultsArchive,
    ) -> CacheResult<Arc<AlignedVec>> {
        #[allow(unused_mut)] // only reassigned once a compress/encrypt feature is enabled
        let mut buffer = search_results.buffer();

        #[cfg(any(
            feature = "compress-cache-results",
            feature = "encrypt-cache-results",
            feature = "cec-cache-results"
        ))]
        {
            buffer = Arc::new(into_aligned_vec(self.transform_bytes(&buffer, true).await?));
        }

        Ok(buffer)
    }

    /// Restores an archive from the bytes handed back by a cache store.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The encoded buffer read back from the cache.
    ///
    /// # Error
    ///
    /// Returns the archived search results on success otherwise it returns a CacheError
    /// on failure.
    async fn post_process_search_results(
        &self,
        bytes: Arc<AlignedVec>,
    ) -> CacheResult<SearchResultsArchive> {
        #[cfg(any(
            feature = "compress-cache-results",
            feature = "encrypt-cache-results",
            feature = "cec-cache-results"
        ))]
        let bytes = Arc::new(into_aligned_vec(self.transform_bytes(&bytes, false).await?));

        SearchResultsArchive::from_buffer(bytes).map_err(|_| CacheError::SerializationError)
    }
}

/// Copies a byte buffer into the alignment that rkyv needs to read archived values back out
/// of it. Only needed once a compress/encrypt feature reshapes the encoded bytes.
#[cfg(any(
    feature = "compress-cache-results",
    feature = "encrypt-cache-results",
    feature = "cec-cache-results"
))]
fn into_aligned_vec(bytes: Vec<u8>) -> AlignedVec {
    let mut aligned = AlignedVec::with_capacity(bytes.len());
    aligned.extend_from_slice(&bytes);
    aligned
}

/// A helper function that returns compressed results.
/// Feature flags (**compress-cache-results or cec-cache-results**) are required  for this to work.
/// If bytes where
///
/// # Arguments
///
/// * `bytes` - It takes a slice of bytes as an argument.
///
/// # Error
///
/// Returns the uncompressed bytes on success otherwise it returns a CacheError
/// on failure.
#[cfg(any(feature = "compress-cache-results", feature = "cec-cache-results"))]
async fn decompress_util(input: &[u8]) -> CacheResult<Vec<u8>> {
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

/// A named struct holding the cache configuration structs and provided when the respective
/// features or both enabled.
#[derive(Clone)]
pub struct SwitchCache {
    /// It holds the redis server configuration struct.
    #[cfg(feature = "redis-cache")]
    pub redis_cache: RedisCache,
    /// It holds the moka cache server configuration struct.
    #[cfg(feature = "memory-cache")]
    pub memory_cache: InMemoryCache,
}

impl SwitchCache {
    /// A function that builds/initializes the redis cache struct or memory cache according to the
    /// feature flags enabled.
    ///
    /// # Arguments
    ///
    /// * `config` - It takes the config struct value.
    ///
    /// # Error
    ///
    /// Returns the build struct containing the appropriate initialized caching server struct that
    /// is redis cache or memory cache on success otherwise throws an appropriate error message.
    async fn build(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            #[cfg(feature = "redis-cache")]
            redis_cache: RedisCache::build(config).await,
            #[cfg(feature = "memory-cache")]
            memory_cache: InMemoryCache::build(config).await,
        })
    }

    /// A function that checks if the cached results exists in the respective cache server or not.
    ///
    /// # Arguments
    ///
    /// * `urls` - It takes the hashed search urls as an argument which will be used as the key to
    ///   check whether the cache exists or not.
    ///
    /// # Error
    ///
    /// Returns a Vector containing booleans for each respective url if nothing goes wrong otherwise
    /// returns a `CacheError`.
    async fn cached_results_exists(&self, urls: &[String]) -> CacheResult<Vec<bool>> {
        #[cfg(all(feature = "redis-cache", not(feature = "memory-cache")))]
        {
            self.redis_cache.cached_results_exists(urls).await
        }

        #[cfg(all(feature = "memory-cache", not(feature = "redis-cache")))]
        {
            self.memory_cache.cached_results_exists(urls).await
        }

        #[cfg(all(feature = "memory-cache", feature = "redis-cache"))]
        {
            match self.redis_cache.cached_results_exists(urls).await {
                Ok(res) => Ok(res),
                Err(_) => self.memory_cache.cached_results_exists(urls).await,
            }
        }
    }

    /// A function that fetches the cached data from the respective cache servers.
    ///
    /// # Arguments
    ///
    /// * `url` - takes the url parameter as string which will be used as key to fetch the data
    ///   from the cache.
    ///
    /// # Error
    ///
    /// Returns the cached data on success otherwise returns a custom CacheError on failure.
    async fn cached_results(&self, url: &str) -> CacheResult<SearchResultsArchive> {
        #[cfg(all(feature = "redis-cache", not(feature = "memory-cache")))]
        {
            self.redis_cache.cached_results(url).await
        }

        #[cfg(all(feature = "memory-cache", not(feature = "redis-cache")))]
        {
            self.memory_cache.cached_results(url).await
        }

        #[cfg(all(feature = "memory-cache", feature = "redis-cache"))]
        {
            if let Ok(res) = self.memory_cache.cached_results(url).await {
                Ok(res)
            } else if let Ok(res) = self.redis_cache.cached_results(url).await {
                if let Err(err) = self
                    .memory_cache
                    .cache_results(std::slice::from_ref(&res), &[url.to_string()])
                    .await
                {
                    log::warn!("memory cache populate failed for {url}: {err}");
                }
                Ok(res)
            } else {
                self.memory_cache.cached_results(url).await
            }
        }
    }

    /// A function that caches the results to the respective cache servers.
    ///
    /// # Arguments
    ///
    /// * `urls` - takes the list of urls for each page which will be used as key for the results
    ///   to be cached.
    /// * `search_results` - takes the list of search_results for each page as the value for the
    ///   respective url key for that page.
    ///
    /// # Error
    ///
    /// Returns the cached data on success otherwise returns a custom CacheError on failure.
    async fn cache_results(
        &self,
        search_results: &[SearchResultsArchive],
        urls: &[String],
    ) -> CacheResult<()> {
        #[cfg(all(feature = "redis-cache", not(feature = "memory-cache")))]
        {
            self.redis_cache.cache_results(search_results, urls).await
        }

        #[cfg(all(feature = "memory-cache", not(feature = "redis-cache")))]
        {
            self.memory_cache.cache_results(search_results, urls).await
        }

        #[cfg(all(feature = "memory-cache", feature = "redis-cache"))]
        {
            match self.redis_cache.cache_results(search_results, urls).await {
                Ok(res) => Ok(res),
                Err(_) => self.memory_cache.cache_results(search_results, urls).await,
            }
        }
    }
}

/// A structure to efficiently share the cache between threads - as it is protected by a lock-free
/// ArcSwap structure. Reads are lock-free and wait-free (using ArcSwap::load). Writes use
/// copy-on-write: load the current cache, clone it, mutate the clone, then atomically swap.
pub struct SharedCache(ArcSwap<SwitchCache>);

impl SharedCache {
    /// A function that creates a new `SharedCache` from a Cache implementation.
    ///
    /// # Arguments
    ///
    /// * `cache` - It takes the `Cache` enum variant as an argument with the prefered cache type.
    ///
    /// Returns a newly constructed `SharedCache` struct.
    pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self(ArcSwap::from_pointee(
            SwitchCache::build(config).await?,
        )))
    }

    /// A lock-free, wait-free read operation.
    ///
    /// Returns a guard that provides access to the current cache without cloning.
    fn read(&self) -> arc_swap::Guard<Arc<SwitchCache>> {
        self.0.load()
    }

    /// A function that checks if the cached results exists in the cache server or not.
    ///
    /// # Arguments
    ///
    /// * `urls` - It takes the hashed search urls as an argument which will be used as the key to
    ///   check whether the cache exists or not.
    ///
    /// # Error
    ///
    /// Returns a Vector containing booleans for each respective url if nothing goes wrong otherwise
    /// returns a `CacheError`.
    pub async fn cached_results_exists(&self, urls: &[String]) -> CacheResult<Vec<bool>> {
        let cache = self.read();
        cache.cached_results_exists(urls).await
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
    pub async fn cached_results(&self, url: &str) -> CacheResult<SearchResultsArchive> {
        let cache = self.read();
        cache.cached_results(url).await
    }

    /// A setter function which caches the results by using the `url` as the key and
    /// `SearchResultsArchive` as the value. Uses copy-on-write: clones the cache, mutates the
    /// clone, then atomically swaps.
    ///
    /// # Arguments
    ///
    /// * `search_results` - It takes the `SearchResultsArchive` as an argument which are results
    ///   that needs to be cached.
    /// * `url` - It takes the search url as an argument which will be used as the key for storing
    ///   results in the cache.
    ///
    /// # Error
    ///
    /// Returns an unit type if the results are cached succesfully otherwise returns a `CacheError`
    /// on a failure.
    pub async fn cache_results(
        &self,
        search_results: &[SearchResultsArchive],
        urls: &[String],
    ) -> CacheResult<()> {
        // Load current cache, clone the inner SwitchCache for mutation (copy-on-write)
        let new_cache = (**self.read()).clone();
        let result = new_cache.cache_results(search_results, urls).await;

        // Only store the new cache if the write succeeded
        if result.is_ok() {
            self.0.store(Arc::new(new_cache));
        }

        result
    }
}
