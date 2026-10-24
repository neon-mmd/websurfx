// Rust guideline compliant 2026-10-09

//! This module handles the search route of the search engine website.

use crate::{
    aggregator::aggregate,
    handler::{FileType, file_path},
    models::{
        aggregation::SearchResults,
        engine::EngineHandler,
        search_route::{self, SearchParams},
    },
    parser::Config,
    user_agent::random_user_agent,
};

#[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
use {crate::cache::SharedCache, tokio::sync::OnceCell};

use actix_web::{HttpRequest, HttpResponse, ResponseError, get, http::header::ContentType, web};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{borrow::Cow, time::Duration};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

/// A static constant for holding the cache struct.
#[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
static SHARED_CACHE: OnceCell<SharedCache> = OnceCell::const_new();

/// Handles the route of search page of the `websurfx` meta search engine website and it takes
/// two search url parameters `q` and `page` where `page` parameter is optional.
/// An optional `format` parameter can be provided to get results as JSON.
///
/// # Example
///
/// ```bash
/// # HTML response (default)
/// curl "http://127.0.0.1:8080/search?q=sweden&page=1"
///
/// # JSON API response
/// curl "http://127.0.0.1:8080/search?q=sweden&format=json=true"
/// ```
///
/// Detect `format=json` from the raw query string before deserialization,
/// so that parse failures can still return a JSON-shaped 400 response.
#[get("/search")]
pub async fn search(
    req: HttpRequest,
    config: web::Data<&'static Config>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let params_result = web::Query::<SearchParams>::from_query(req.query_string());
    let params = if let Err(e) = params_result {
        if req.query_string().contains("&json") {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "code": format!("{}", e.status_code()),
                "error": format!("Invalid query parameters: {}", e)
            })));
        }
        return Err(e.into());
    } else {
        params_result.unwrap()
    };

    if let Some(ref q) = params.q
        && q.len() > 512
    {
        if let Some(json) = &params.json
            && *json
        {
            return Ok(HttpResponse::BadRequest()
                .insert_header((
                    "Cache-Control",
                    "no-store, no-cache, must-revalidate, private",
                ))
                .json(serde_json::json!({
                    "code": "400",
                    "error": "Query exceeds maximum allowed length of 512 characters"
                })));
        }
        return Ok(HttpResponse::BadRequest()
            .insert_header((
                "Cache-Control",
                "no-store, no-cache, must-revalidate, private",
            ))
            .body("Query exceeds maximum allowed length of 512 characters"));
    }

    let result = fetch_results(req, &config, &params).await?;

    if let Some((current_results, query, page)) = result {
        if let Some(json) = &params.json
            && *json
        {
            return Ok(HttpResponse::Ok()
                .insert_header((
                    "Cache-Control",
                    "no-store, no-cache, must-revalidate, private",
                ))
                .content_type(ContentType::json())
                .json(&current_results));
        }
        Ok(HttpResponse::Ok()
            .insert_header((
                "Cache-Control",
                "no-store, no-cache, must-revalidate, private",
            ))
            .content_type(ContentType::html())
            .body(
                crate::templates::views::search::search(
                    &config.style.colorscheme,
                    &config.style.theme,
                    &config.style.animation,
                    &query,
                    page,
                    &current_results,
                )
                .0,
            ))
    } else {
        if let Some(json) = &params.json
            && *json
        {
            return Ok(HttpResponse::BadRequest()
                .insert_header((
                    "Cache-Control",
                    "no-store, no-cache, must-revalidate, private",
                ))
                .json(serde_json::json!({
                    "code": "400",
                    "error": "Empty query provided"
                })));
        }
        Ok(HttpResponse::TemporaryRedirect()
            .insert_header(("location", "/"))
            .finish())
    }
}

/// Fetches search results from cache or upstream engines.
///
/// # Arguments
///
/// * `req` - The HTTP request used to extract cookies and pass to upstream calls.
/// * `config` - A reference to the application configuration.
/// * `params` - The parsed search parameters including query string and page number.
///
/// # Returns
///
/// Returns `Ok(Some((SearchResults, query, page)))` on success with the results,
/// query string, and zero-based page number. Returns `Ok(None)` for empty queries.
/// Returns an error if results could not be fetched from cache or upstream engines.
///
/// # Examples
///
/// ```rust,ignore
/// let result = fetch_results(req, &config, params).await?;
/// ```
async fn fetch_results(
    req: HttpRequest,
    config: &web::Data<&'static Config>,
    params: &SearchParams,
) -> Result<Option<(SearchResults, String, u32)>, Box<dyn std::error::Error>> {
    // Validate the query early, before touching the cache or doing any setup.
    if params.q.as_ref().is_none_or(|q| q.trim().is_empty()) {
        return Ok(None);
    }

    #[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
    let cache = SHARED_CACHE
        .get_or_try_init(|| SharedCache::new(config))
        .await?;

    // Safe to unwrap: we validated q is Some and non-empty above.
    let query = params.q.as_deref().unwrap();

    let cookie = req.cookie("appCookie");

    // Get search settings using the user's cookie or from the server's config
    let mut search_settings: search_route::Cookie<'_> = cookie
        .as_ref()
        .and_then(|cookie_value| serde_json::from_str(cookie_value.value()).ok())
        .unwrap_or_else(|| {
            search_route::Cookie::build(
                &config.style,
                config
                    .upstream_search_engines
                    .iter()
                    .filter_map(|(engine, enabled)| {
                        enabled.then_some(Cow::Borrowed(engine.as_str()))
                    })
                    .collect(),
                config.safe_search,
            )
        });

    // Enforce server policy: filter requested engines against enabled engines in config
    let effective_engines: Vec<Cow<'_, str>> = search_settings
        .engines
        .iter()
        .filter(|engine| {
            config
                .upstream_search_engines
                .get(engine.as_ref())
                .copied()
                .unwrap_or(false)
        })
        .cloned()
        .collect();
    search_settings.engines = Cow::Owned(effective_engines);

    search_settings.safe_search_level = get_safesearch_level(
        params.safesearch,
        search_settings.safe_search_level,
        config.safe_search,
    );

    // Add a random delay before making the request.
    if config.aggregator.random_delay || config.debug {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos();
        let delay = nanos % 10 + 1;
        tokio::time::sleep(Duration::from_secs(delay as u64)).await;
    }

    let user_agent: &'static str = random_user_agent(config.threads).await?;

    // Clamp page to valid range 1..=50
    let page = params.page.unwrap_or(1).clamp(1, 50) - 1;
    let query_owned = query.to_owned();

    let current_results: SearchResults;

    #[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
    {
        let mut sorted_engines: Vec<String> = search_settings
            .engines
            .iter()
            .map(|e| e.to_string())
            .collect();
        sorted_engines.sort();
        sorted_engines.dedup();

        let current_page_cache_key = canonical_cache_key(
            query,
            page,
            search_settings.safe_search_level,
            &sorted_engines,
        );

        current_results = match cache.cached_results(&current_page_cache_key).await {
            Ok(cached) => cached,
            Err(_) => {
                let fetched_results =
                    results(config, &query_owned, page, &search_settings, user_agent).await?;
                let fetched_results_clone = fetched_results.clone();
                if !fetched_results.no_engines_selected()
                    && !fetched_results.disallowed
                    && !fetched_results.results().is_empty()
                {
                    tokio::spawn(async move {
                        let _ = cache
                            .cache_results(&[fetched_results], &[current_page_cache_key])
                            .await;
                    });
                }
                fetched_results_clone
            }
        };
    }

    #[cfg(not(any(feature = "redis-cache", feature = "memory-cache")))]
    {
        current_results = results(config, &query_owned, page, &search_settings, user_agent).await?;
    }

    Ok(Some((current_results, query_owned, page)))
}

/// Fetches the results for a query and page. It First checks the redis cache, if that
/// fails it gets proper results by requesting from the upstream search engines.
///
/// # Arguments
///
/// * `url` - It takes the url of the current page that requested the search results for a
///   particular search query.
/// * `config` - It takes a parsed config struct.
/// * `query` - It takes the page number as u32 value.
/// * `req` - It takes the `HttpRequest` struct as a value.
///
/// # Error
///
/// It returns the `SearchResults` struct if the search results could be successfully fetched from
/// the cache or from the upstream search engines otherwise it returns an appropriate error.
async fn results(
    config: &'static Config,
    query: &str,
    page: u32,
    search_settings: &search_route::Cookie<'_>,
    user_agent: &'static str,
) -> Result<SearchResults, Box<dyn std::error::Error>> {
    // eagerly parse cookie value to evaluate safe search level
    let safe_search_level = search_settings.safe_search_level;

    // check if fetched cache results was indeed fetched or it was an error and if so
    // handle the data accordingly.
    if safe_search_level == 4 {
        let mut results: SearchResults = SearchResults::default();

        let is_blocked: bool =
            is_match_from_filter_list(&file_path(FileType::BlockList).await?, query).await?;
        // Return early when query contains disallowed words
        if is_blocked {
            results.set_disallowed();
            results.set_safe_search_level(safe_search_level);
            return Ok(results);
        }
    }

    // check if the cookie value is empty or not if it is empty then use the
    // default selected upstream search engines from the config file otherwise
    // parse the non-empty cookie and grab the user selected engines from the
    // UI and use that.
    let mut results: SearchResults = if !search_settings.engines.is_empty() {
        aggregate(
            query,
            page,
            config,
            &search_settings
                .engines
                .iter()
                .filter(|engine| {
                    config
                        .upstream_search_engines
                        .get(engine.as_ref())
                        .copied()
                        .unwrap_or(false)
                })
                .filter_map(|engine| EngineHandler::new(engine).ok())
                .collect::<Vec<EngineHandler>>(),
            safe_search_level,
            user_agent,
        )
        .await?
    } else {
        let mut search_results = SearchResults::default();
        search_results.set_no_engines_selected();
        search_results
    };

    let (engine_errors_info, results_empty_check, no_engines_selected) = (
        results.engine_errors_info().is_empty(),
        results.results().is_empty(),
        results.no_engines_selected(),
    );
    results.set_filtered(engine_errors_info & results_empty_check & !no_engines_selected);
    results.set_safe_search_level(safe_search_level);
    Ok(results)
}

/// Computes an unambiguous canonical cache key for a search request.
///
/// Encodes query, page number, safe search level, and sorted enabled engines
/// using length prefixes to prevent collision and delimiter-injection attacks.
///
/// # Arguments
///
/// * `query` - Search query string.
/// * `page` - Zero-based page index.
/// * `safe_search_level` - Active SafeSearch level.
/// * `sorted_engines` - Slice of sorted upstream engine names.
///
/// # Returns
///
/// Returns a hex-encoded BLAKE3 hash representing the canonical cache key.
#[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
pub fn canonical_cache_key(
    query: &str,
    page: u32,
    safe_search_level: u8,
    sorted_engines: &[String],
) -> String {
    let mut key = format!("v2|q:{query}|p:{page}|s:{safe_search_level}|e:");
    for engine in sorted_engines {
        key.push_str(&format!("{}:{}|", engine.len(), engine));
    }
    blake3::hash(key.as_bytes()).to_string()
}

/// A helper function which checks whether the search query contains any keywords which should be
/// disallowed/allowed based on the regex based rules present in the blocklist and allowlist files.
///
/// # Arguments
///
/// * `file_path` - It takes the file path of the list as the argument.
/// * `query` - It takes the search query to be checked against the list as an argument.
///
/// # Error
///
/// Returns a bool indicating whether the results were found in the list or not on success
/// otherwise returns a standard error type on a failure.
async fn is_match_from_filter_list(
    file_path: &str,
    query: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let reader = BufReader::new(File::open(file_path).await?);
    let mut lines = reader.lines();
    let mut patterns = Vec::new();
    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            patterns.push(trimmed);
        }
    }
    if patterns.is_empty() {
        return Ok(false);
    }
    let set = regex::RegexSet::new(&patterns)?;
    Ok(set.is_match(query))
}

/// A helper function to choose the safe search level value based on the URL parameters,
/// cookie value and config value.
///
/// # Argurments
///
/// * `safe_search_level_from_url` - Safe search level from the URL parameters.
/// * `cookie_safe_search_level` - Safe search level value from the cookie.
/// * `config_safe_search_level` - Safe search level value from the config file.
///
/// # Returns
///
/// Returns an appropriate safe search level value based on the safe search level values
/// from the URL parameters, cookie and the config file.
fn get_safesearch_level(
    safe_search_level_from_url: Option<u8>,
    cookie_safe_search_level: u8,
    config_safe_search_level: u8,
) -> u8 {
    let raw = (u8::from(safe_search_level_from_url.is_some())
        * ((u8::from(config_safe_search_level >= 3) * config_safe_search_level)
            + (u8::from(config_safe_search_level < 3) * safe_search_level_from_url.unwrap_or(0))))
        + (u8::from(safe_search_level_from_url.is_none())
            * ((u8::from(config_safe_search_level >= 3) * config_safe_search_level)
                + (u8::from(config_safe_search_level < 3) * cookie_safe_search_level)));
    raw.min(4)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    /// A helper function which creates a random mock safe search level value.
    ///
    /// # Returns
    ///
    /// Returns an optional u8 value.
    fn mock_safe_search_level_value() -> Option<u8> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        let delay = (nanos % 10) as i8 - 1;

        if delay == -1 {
            return None;
        }

        Some(if delay > 4 { delay - 4 } else { delay } as u8)
    }

    #[test]
    /// A test function to test whether the output of the branchless and branched code
    /// for the code to choose the appropriate safe search level is same or not.
    fn get_safesearch_level_branched_branchless_code_test() {
        // Get mock values for the safe search level values for URL parameters, cookie
        // and config.
        let safe_search_level_from_url = mock_safe_search_level_value();
        let cookie_safe_search_level = mock_safe_search_level_value().unwrap_or(0);
        let config_safe_search_level = mock_safe_search_level_value().unwrap_or(0);

        // Branched code
        let safe_search_level_value_from_branched_code =
            if let Some(safe_search_level_from_url_parsed) = safe_search_level_from_url {
                if config_safe_search_level >= 3 {
                    config_safe_search_level
                } else {
                    safe_search_level_from_url_parsed
                }
            } else if config_safe_search_level >= 3 {
                config_safe_search_level
            } else {
                cookie_safe_search_level
            };

        // branchless code
        let safe_search_level_value_from_branchless_code =
            (u8::from(safe_search_level_from_url.is_some())
                * ((u8::from(config_safe_search_level >= 3) * config_safe_search_level)
                    + (u8::from(config_safe_search_level < 3)
                        * safe_search_level_from_url.unwrap_or(0))))
                + (u8::from(safe_search_level_from_url.is_none())
                    * ((u8::from(config_safe_search_level >= 3) * config_safe_search_level)
                        + (u8::from(config_safe_search_level < 3) * cookie_safe_search_level)));

        assert_eq!(
            safe_search_level_value_from_branched_code,
            safe_search_level_value_from_branchless_code
        );
    }

    #[test]
    fn test_get_safesearch_level_clamping() {
        // SafeSearch levels 0..=4 should be preserved without truncation
        assert_eq!(super::get_safesearch_level(Some(0), 0, 0), 0);
        assert_eq!(super::get_safesearch_level(Some(1), 0, 0), 1);
        assert_eq!(super::get_safesearch_level(Some(2), 0, 0), 2);
        assert_eq!(super::get_safesearch_level(Some(3), 0, 0), 3);
        assert_eq!(super::get_safesearch_level(Some(4), 0, 0), 4);
        // Level 5 and above clamped to 4
        assert_eq!(super::get_safesearch_level(Some(5), 0, 0), 4);
        assert_eq!(super::get_safesearch_level(Some(99), 0, 0), 4);
    }

    #[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
    #[test]
    fn test_canonical_cache_key_unambiguous() {
        let engines_separate = vec!["bing".to_string(), "duckduckgo".to_string()];
        let engines_combined = vec!["bing,duckduckgo".to_string()];

        let key1 = super::canonical_cache_key("cats", 0, 1, &engines_separate);
        let key2 = super::canonical_cache_key("cats", 0, 1, &engines_combined);

        // Different engine arrays must produce distinct hashes even if comma-separated representation looks similar
        assert_ne!(key1, key2);

        // Deterministic
        assert_eq!(
            key1,
            super::canonical_cache_key("cats", 0, 1, &engines_separate)
        );

        // Different query
        assert_ne!(
            key1,
            super::canonical_cache_key("dogs", 0, 1, &engines_separate)
        );

        // Different page
        assert_ne!(
            key1,
            super::canonical_cache_key("cats", 1, 1, &engines_separate)
        );

        // Different safe search level
        assert_ne!(
            key1,
            super::canonical_cache_key("cats", 0, 2, &engines_separate)
        );
    }
}
