//! This module handles the search route of the search engine website.

use crate::{
    aggregator::aggregate,
    handler::{FileType, file_path},
    json::JsonWriter,
    models::{
        aggregation::{SearchResults, SearchResultsArchive},
        engine::EngineHandler,
        search_route::{self, SearchParams},
    },
    parser::Config,
    user_agent::random_user_agent,
};

#[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
use {crate::cache::SharedCache, tokio::sync::OnceCell};

use regex::Regex;
use std::borrow::Cow;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};
use xitca_web::{
    WebContext,
    http::{WebResponse, header},
};

use crate::server::{
    APP_COOKIE, HTML, JSON, bad_request, cookie_value, internal_error, ok, preferences, redirect,
};

/// A static constant for holding the cache struct.
#[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
static SHARED_CACHE: OnceCell<SharedCache> = OnceCell::const_new();

/// A helper which builds the JSON body of an error response.
///
/// # Arguments
///
/// * `code` - It takes the status code reported to the client.
/// * `error` - It takes the human readable message reported to the client.
fn json_error(code: &str, error: &str) -> WebResponse {
    let mut writer = JsonWriter::with_capacity(error.len() + 32);

    writer.begin_object();
    writer.field("code").string(code);
    writer.field("error").string(error);
    writer.end_object();

    bad_request(JSON, writer.finish())
}

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
/// Detect `format=json` from the raw query string before parsing the parameters, so that
/// parse failures can still return a JSON-shaped 400 response.
pub async fn search(ctx: &WebContext<'_, &'static Config>) -> WebResponse {
    let config = ctx.state();
    let query_string = ctx.req().uri().query().unwrap_or_default();

    let params = match SearchParams::parse(query_string) {
        Ok(params) => params,
        Err(e) => {
            if query_string.contains("&json") {
                return json_error("400", &format!("Invalid query parameters: {e}"));
            }
            return internal_error(e);
        }
    };

    let settings = search_settings(ctx, config);

    let result = match fetch_results(config, settings, &params).await {
        Ok(result) => result,
        Err(e) => return internal_error(e),
    };

    let Some((current_results, query, page)) = result else {
        if params.json == Some(true) {
            return json_error("400", "Empty query provided");
        }
        return redirect("/");
    };

    if params.json == Some(true) {
        return match current_results.to_json() {
            Ok(body) => ok(JSON, body),
            Err(e) => internal_error(e),
        };
    }

    // Read the archive once: the archived results borrow straight out of the encoded
    // buffer, so rendering a page allocates nothing beyond the markup itself.
    let archived_results = match current_results.get() {
        Ok(archived) => archived,
        Err(e) => return internal_error(e),
    };

    ok(
        HTML,
        crate::templates::views::search::search(
            &config.style.colorscheme,
            &config.style.theme,
            &config.style.animation,
            &query,
            page,
            archived_results,
        )
        .0,
    )
}

/// Builds the search preferences from the `appCookie`, falling back to the server config.
///
/// The returned cookie borrows its strings from the request when the client sent one, so a
/// request that carries no preferences allocates nothing extra.
fn search_settings<'a>(
    ctx: &'a WebContext<'_, &'static Config>,
    config: &'a Config,
) -> search_route::Cookie<'a> {
    preferences(
        cookie_value(ctx.req().headers().get(header::COOKIE), APP_COOKIE),
        config,
        config.safe_search,
    )
}

/// Fetches search results from cache or upstream engines.
///
/// # Arguments
///
/// * `config` - The parsed application config.
/// * `mut search_settings` - The user's preferences, taken from the `appCookie` when present.
/// * `params` - The parsed search parameters including query string and page number.
///
/// # Returns
///
/// Returns `Ok(Some((SearchResultsArchive, query, page)))` on success with the results,
/// query string, and zero-based page number. Returns `Ok(None)` for empty queries.
/// Returns an error if results could not be fetched from cache or upstream engines.
async fn fetch_results(
    config: &'static Config,
    mut search_settings: search_route::Cookie<'_>,
    params: &SearchParams,
) -> Result<Option<(SearchResultsArchive, String, u32)>, Box<dyn std::error::Error>> {
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

    // .max(1) makes sure that the page >= 0.
    let page = params.page.unwrap_or(1).max(1) - 1;
    let query_owned = query.to_owned();

    let current_results: SearchResultsArchive;

    #[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
    {
        let previous_page = page.saturating_sub(1);

        let next_page = page + 1;

        let mut pages = vec![next_page, previous_page, page];
        pages.dedup();

        // Built once rather than per page: the engine list is the same for every URL.
        let engines = search_settings.engines.join(",");

        let urls: Vec<String> = pages
            .iter()
            .map(|page| {
                format!(
                    "http://{}:{}/search?q={}&page={}&safesearch={}&engines={}",
                    config.binding_ip,
                    config.port,
                    query,
                    page,
                    search_settings.safe_search_level,
                    engines
                )
            })
            .collect();

        // Hashing three URLs of a few dozen bytes costs well under a microsecond in total.
        // Handing them to a blocking pool and waiting for a thread round trip cost several
        // times more than the hashing itself, on every request, cache hit included.
        let mut cache_keys: Vec<String> = urls.iter().map(|url| hash_url(url)).collect();

        let current_page_cache_key = cache_keys.pop().unwrap();

        // Use match to avoid eagerly evaluating the upstream fetch on cache hits.
        current_results = match cache.cached_results(&current_page_cache_key).await {
            Ok(cached) => cached,
            Err(_) => {
                let fetched_results = results(
                    config,
                    &query_owned,
                    page,
                    &search_settings.engines,
                    search_settings.safe_search_level,
                    user_agent,
                )
                .await
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
                // Cloning an archive only bumps a refcount, so the same encoded buffer can
                // be rendered here and stored in the background.
                let fetched_results_clone = fetched_results.clone();
                tokio::spawn(async move {
                    if let Some(cache) = SHARED_CACHE.get() {
                        let _ = cache
                            .cache_results(
                                std::slice::from_ref(&fetched_results_clone),
                                std::slice::from_ref(&current_page_cache_key),
                            )
                            .await;
                    }
                });
                fetched_results
            }
        };

        if let Ok(resolved_results) = cache.cached_results_exists(&cache_keys).await {
            // The two branches below are exclusive and the current page's key was already
            // popped, so the keys are moved rather than cloned on the way to the background
            // task.
            let missing: Vec<(String, u32)> = resolved_results
                .iter()
                .zip(cache_keys)
                .zip(pages.iter())
                .filter(|resolved_result| !*resolved_result.0.0)
                .map(|resolved_result| (resolved_result.0.1, *resolved_result.1))
                .collect();

            // Fetch and cache adjacent pages in the background so the current
            // response isn't blocked on them.
            if !missing.is_empty() {
                let (missing_keys, missing_pages): (Vec<String>, Vec<u32>) =
                    missing.into_iter().unzip();
                let query_for_bg = query_owned.clone();
                // The background task outlives the request, so the names are detached from
                // the borrow. This runs once per request on the prefetch path, next to a
                // full upstream fetch, so owning them costs nothing measurable.
                let engines_for_bg = own_engines(&search_settings.engines);
                let safe_search_bg = search_settings.safe_search_level;
                tokio::spawn(async move {
                    let tasks = missing_pages.iter().map(|page| {
                        results(
                            config,
                            &query_for_bg,
                            *page,
                            &engines_for_bg,
                            safe_search_bg,
                            user_agent,
                        )
                    });
                    let fetched = futures::future::try_join_all(tasks).await.ok();
                    if let (Some(fetched_results), Some(cache)) = (fetched, SHARED_CACHE.get()) {
                        let _ = cache.cache_results(&fetched_results, &missing_keys).await;
                    }
                });
            }
        } else {
            let cache_keys_for_bg = cache_keys;
            let pages_for_bg = pages.clone();
            let query_for_bg = query_owned.clone();
            // See the note on the other background-prefetch site above.
            let engines_for_bg = own_engines(&search_settings.engines);
            let safe_search_bg = search_settings.safe_search_level;
            tokio::spawn(async move {
                let tasks = pages_for_bg.iter().map(|page| {
                    results(
                        config,
                        &query_for_bg,
                        *page,
                        &engines_for_bg,
                        safe_search_bg,
                        user_agent,
                    )
                });
                let fetched = futures::future::try_join_all(tasks).await.ok();
                if let (Some(fetched_results), Some(cache)) = (fetched, SHARED_CACHE.get()) {
                    let _ = cache
                        .cache_results(&fetched_results, &cache_keys_for_bg)
                        .await;
                }
            });
        }
    }

    #[cfg(not(any(feature = "redis-cache", feature = "memory-cache")))]
    {
        current_results = results(
            config,
            &query_owned,
            page,
            &search_settings.engines,
            search_settings.safe_search_level,
            user_agent,
        )
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    }

    Ok(Some((current_results, query_owned, page)))
}

/// Fetches the results for a query and page and encodes them into a zero-copy archive.
///
/// # Arguments
///
/// * `config` - It takes a parsed config struct.
/// * `query` - It takes the search query.
/// * `page` - It takes the page number as u32 value.
/// * `engines` - It takes the selected upstream search engines.
/// * `safe_search_level` - It takes the safe search level value.
/// * `user_agent` - It takes the user agent string used for the upstream requests.
///
/// # Error
///
/// It returns the `SearchResultsArchive` if the search results could be successfully gathered
/// from the upstream search engines and encoded, otherwise it returns a `String` describing the
/// failure.
async fn results(
    config: &Config,
    query: &str,
    page: u32,
    engines: &[Cow<'_, str>],
    safe_search_level: u8,
    user_agent: &'static str,
) -> Result<SearchResultsArchive, String> {
    // Encoding once here means the rendered page, the JSON API response and the cache write
    // all share a single buffer instead of each rebuilding their own copy of the results.
    let search_results =
        collect_results(config, query, page, engines, safe_search_level, user_agent).await?;

    SearchResultsArchive::encode(&search_results).map_err(|error| error.to_string())
}

/// Aggregates (or synthesises) the owned results for a query and page.
///
/// # Arguments
///
/// * `config` - It takes a parsed config struct.
/// * `query` - It takes the search query.
/// * `page` - It takes the page number as u32 value.
/// * `engines` - It takes the selected upstream search engines.
/// * `safe_search_level` - It takes the safe search level value.
/// * `user_agent` - It takes the user agent string used for the upstream requests.
///
/// # Error
///
/// Returns a `String` describing the failure if the upstream search engines could not be
/// queried.
async fn collect_results(
    config: &Config,
    query: &str,
    page: u32,
    engines: &[Cow<'_, str>],
    safe_search_level: u8,
    user_agent: &'static str,
) -> Result<SearchResults, String> {
    // check if fetched cache results was indeed fetched or it was an error and if so
    // handle the data accordingly.
    if safe_search_level == 4 {
        let mut results: SearchResults = SearchResults::default();

        let flag: bool = !is_match_from_filter_list(
            &file_path(FileType::BlockList)
                .await
                .map_err(|e| e.to_string())?,
            query,
        )
        .await
        .map_err(|e| e.to_string())?;
        // Return early when query contains disallowed words,
        if flag {
            results.set_disallowed();
            results.set_safe_search_level(safe_search_level);
            return Ok(results);
        }
    }

    // check if the cookie value is empty or not if it is empty then use the
    // default selected upstream search engines from the config file otherwise
    // parse the non-empty cookie and grab the user selected engines from the
    // UI and use that.
    let mut results: SearchResults = if !engines.is_empty() {
        // The handlers are shared process-wide, so this no longer compiles an engine's CSS
        // selectors on every request.
        let handlers = engines
            .iter()
            .filter_map(|engine| EngineHandler::shared(engine.as_ref()).ok())
            .collect::<Vec<&'static EngineHandler>>();

        aggregate(
            query,
            page,
            config,
            &handlers,
            safe_search_level,
            user_agent,
        )
        .await
        .map_err(|e| e.to_string())?
    } else {
        let mut search_results = SearchResults::default();
        search_results.set_no_engines_selected();
        search_results
    };

    let engine_errors_empty = results.engine_errors_info.is_empty();
    let results_empty = results.results.is_empty();
    let no_engines_selected = results.no_engines_selected();

    results.set_filtered(engine_errors_empty & results_empty & !no_engines_selected);
    results.set_safe_search_level(safe_search_level);
    Ok(results)
}

/// Detaches engine names from a borrow so a background task can own them.
///
/// # Arguments
///
/// * `engines` - It takes the borrowed engine names.
#[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
fn own_engines(engines: &[Cow<'_, str>]) -> Vec<Cow<'static, str>> {
    engines
        .iter()
        .map(|engine| Cow::Owned(engine.clone().into_owned()))
        .collect()
}

/// A helper function which computes the hash of the url and formats and returns it as string.
///
/// # Arguments
///
/// * `url` - It takes an url as string.
#[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
fn hash_url(url: &str) -> String {
    blake3::hash(url.as_bytes()).to_string()
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
    // Use the same cached patterns as filter_with_lists
    use crate::aggregator::FILTER_PATTERNS_CACHE;

    let cache = FILTER_PATTERNS_CACHE.get_or_init(|| {
        std::sync::Mutex::new(std::collections::HashMap::with_hasher(
            ahash::RandomState::new(),
        ))
    });

    let patterns = {
        let cache_guard = cache.lock().unwrap();
        if let Some(cached) = cache_guard.get(file_path) {
            cached.clone()
        } else {
            Vec::new()
        }
    };

    let patterns = if patterns.is_empty() {
        let reader = BufReader::new(File::open(file_path).await?);
        let mut lines = reader.lines();
        let mut compiled: Vec<Regex> = Vec::new();
        while let Some(line) = lines.next_line().await? {
            compiled.push(Regex::new(&line)?);
        }
        {
            let mut cache_guard = cache.lock().unwrap();
            cache_guard.insert(file_path.to_string(), compiled.clone());
        }
        compiled
    } else {
        patterns
    };

    for re in &patterns {
        if re.is_match(query) {
            return Ok(true);
        }
    }

    Ok(false)
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
    (u8::from(safe_search_level_from_url.is_some())
        * ((u8::from(config_safe_search_level >= 3) * config_safe_search_level)
            + (u8::from(config_safe_search_level < 3) * safe_search_level_from_url.unwrap_or(0))))
        + (u8::from(safe_search_level_from_url.is_none())
            * ((u8::from(config_safe_search_level >= 3) * config_safe_search_level)
                + (u8::from(config_safe_search_level < 3) * cookie_safe_search_level)))
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
}
