//! This module provides the modules which provide the functionality to cache the aggregated
//! results fetched and aggregated from the upstream search engines in a json format.

use crate::{models::aggregation::SearchResults, parser::Config};
use arc_swap::ArcSwap;
use error::CacheError;
use error_stack::Report;
use std::convert::TryInto;

#[cfg(feature = "redis-cache")]
use redis::RedisCache;

#[cfg(feature = "memory-cache")]
use {memory::InMemoryCache, std::sync::Arc};

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
    async fn cached_results(&mut self, url: &str) -> Result<SearchResults, Report<CacheError>>;

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
    async fn cached_results_exists(
        &mut self,
        urls: &[String],
    ) -> Result<Vec<bool>, Report<CacheError>>;

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
        &mut self,
        mut bytes: Vec<u8>,
        encrypt: bool,
    ) -> Result<Vec<u8>, Report<CacheError>> {
        use chacha20poly1305::{
            ChaCha20Poly1305,
            aead::{Aead, AeadCore, KeyInit, OsRng},
        };

        Ok(
            tokio::task::spawn_blocking(move || -> Result<Vec<u8>, Report<CacheError>> {
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
    ///
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
    ///
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
    /// # Error
    ///
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
    /// # Error
    ///
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
    async fn cached_results_exists(
        &mut self,
        urls: &[String],
    ) -> Result<Vec<bool>, Report<CacheError>> {
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
    async fn cached_results(&mut self, url: &str) -> Result<SearchResults, Report<CacheError>> {
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
        &mut self,
        search_results: &[SearchResults],
        urls: &[String],
    ) -> Result<(), Report<CacheError>> {
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

/// TryInto implementation for SearchResults from Vec<u8>
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

/// A structure to efficiently share the cache between threads - as it is protected by a lock-free
/// ArcSwap structure.
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

    /// A lock-free, wait-free get operation.
    ///
    /// # Returns
    ///
    /// returns an owned copy of the `SwitchCache` struct.
    fn cache(&self) -> SwitchCache {
        (*self.0.load_full()).clone()
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
    pub async fn cached_results_exists(
        &self,
        urls: &[String],
    ) -> Result<Vec<bool>, Report<CacheError>> {
        self.cache().cached_results_exists(urls).await
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
        self.cache().cached_results(url).await
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
        let mut mut_cache = self.cache();
        let cache_results = mut_cache.cache_results(search_results, urls).await;

        #[cfg(feature = "memory-cache")]
        self.0.store(Arc::new(mut_cache));

        cache_results
    }
}
