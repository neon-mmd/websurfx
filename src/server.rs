//! This module provides the HTTP plumbing shared by every route: response builders and the
//! middlewares that replaced the ones the `actix-web` ecosystem used to provide.

use std::borrow::Cow;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

use http_rate::Quota;
use tower_http::cors::{Any, CorsLayer};
use xitca_web::{
    WebContext,
    body::ResponseBody,
    error::Error,
    handler::cookie::Cookie,
    http::{HeaderValue, Method, Response, StatusCode, WebResponse, header},
    service::Service,
};

use crate::{models::search_route, parser::Config};

/// The media type of every HTML page.
pub const HTML: &str = "text/html; charset=utf-8";

/// The media type of a plain text document.
pub const PLAIN_TEXT: &str = "text/plain; charset=utf-8";

/// The media type of a JSON document.
pub const JSON: &str = "application/json";

/// The media type of an OpenSearch description document.
const OPENSEARCH: &str = "application/opensearchdescription+xml";

/// The name of the cookie the user's search preferences are stored in.
pub const APP_COOKIE: &str = "appCookie";

/// How long the preferences cookie is valid for, in seconds.
///
/// Fifty-two weeks, matching the lifetime the settings cookie used to be given.
const APP_COOKIE_MAX_AGE: i64 = 52 * 7 * 24 * 60 * 60;

/// Builds a response carrying `body` with the given status and content type.
///
/// # Arguments
///
/// * `status` - It takes the status code to send.
/// * `content_type` - It takes the media type to send.
/// * `body` - It takes the response body.
pub fn response(
    status: StatusCode,
    content_type: &'static str,
    body: impl Into<String>,
) -> WebResponse {
    let mut response: WebResponse = Response::new(ResponseBody::from(body.into()));

    *response.status_mut() = status;
    let _ = response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));

    response
}

/// Builds an `Ok` response carrying `body`.
pub fn ok(content_type: &'static str, body: impl Into<String>) -> WebResponse {
    response(StatusCode::OK, content_type, body)
}

/// Builds a `Bad Request` response carrying `body`.
pub fn bad_request(content_type: &'static str, body: impl Into<String>) -> WebResponse {
    response(StatusCode::BAD_REQUEST, content_type, body)
}

/// Builds an `Internal Server Error` response, logging `error` first.
pub fn internal_error(error: impl std::fmt::Display) -> WebResponse {
    log::error!("{error}");

    response(
        StatusCode::INTERNAL_SERVER_ERROR,
        PLAIN_TEXT,
        "Internal Server Error",
    )
}

/// Builds a response serving an OpenSearch description document.
pub fn opensearch(body: impl Into<String>) -> WebResponse {
    ok(OPENSEARCH, body)
}

/// Builds a redirect to `location`.
pub fn redirect(location: &str) -> WebResponse {
    let mut response: WebResponse = Response::new(ResponseBody::empty());

    *response.status_mut() = StatusCode::TEMPORARY_REDIRECT;
    let _ = response.headers_mut().insert(
        header::LOCATION,
        HeaderValue::from_str(location).unwrap_or_else(|_| HeaderValue::from_static("/")),
    );

    response
}

/// Records the user's preferences on `response` as a `Set-Cookie` header.
///
/// # Arguments
///
/// * `response` - It takes the response to attach the header to.
/// * `value` - It takes the JSON document to store.
pub fn with_app_cookie(mut response: WebResponse, value: String) -> WebResponse {
    let cookie = Cookie::build((APP_COOKIE, value))
        .max_age(time::Duration::seconds(APP_COOKIE_MAX_AGE))
        .build();

    if let Ok(value) = HeaderValue::try_from(cookie.encoded().to_string()) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }

    response
}

/// Finds the value of the preferences cookie in a raw `Cookie` header.
///
/// xitca's own `CookieJar` extractor answers with a blank `400` when a request carries no
/// `Cookie` header at all, which is exactly the case for a client that has never changed its
/// preferences, so the header is read directly instead.
///
/// # Arguments
///
/// * `raw` - It takes the value of the request's `Cookie` header, if it sent one.
/// * `name` - It takes the name of the cookie to look for.
pub fn cookie_value<'a>(raw: Option<&'a HeaderValue>, name: &str) -> Option<&'a str> {
    raw?.to_str()
        .ok()?
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
}

/// Restores the user's preferences from `value`, falling back to `config`.
///
/// The returned cookie borrows its strings out of `value` when the client sent usable
/// preferences, so that path allocates nothing.
///
/// # Arguments
///
/// * `value` - It takes the raw preferences cookie, if the client sent one.
/// * `config` - It supplies the defaults used when the client sent none.
/// * `safe_search_level` - It is the level used for the defaults, which differs per route.
pub fn preferences<'a, 'c: 'a>(
    value: Option<&'a str>,
    config: &'c Config,
    safe_search_level: u8,
) -> search_route::Cookie<'a> {
    /// Builds the preferences a client with no usable cookie gets: the server's own style
    /// with every enabled upstream engine.
    fn fallback<'c>(config: &'c Config, safe_search_level: u8) -> search_route::Cookie<'c> {
        search_route::Cookie::build(
            &config.style,
            config
                .upstream_search_engines
                .iter()
                .filter_map(|(engine, enabled)| {
                    (*enabled).then_some(Cow::Borrowed(engine.as_str()))
                })
                .collect(),
            safe_search_level,
        )
    }

    match value.and_then(|value| search_route::Cookie::from_json(value).ok()) {
        Some(preferences) => preferences,
        None => fallback(config, safe_search_level),
    }
}

/// Adds the configured `Cache-Control: max-age` header to every response.
pub async fn with_cache_control<S>(
    service: &S,
    ctx: WebContext<'_, &'static Config>,
) -> Result<WebResponse, Error>
where
    S: for<'r> Service<WebContext<'r, &'static Config>, Response = WebResponse, Error = Error>,
{
    // The expiry never changes while the server runs, so the header value is rendered once
    // instead of formatting and re-parsing it on every response.
    static CACHE_CONTROL: OnceLock<HeaderValue> = OnceLock::new();

    let value = CACHE_CONTROL.get_or_init(|| {
        let seconds = CACHE_CONTROL_SECONDS.get().copied().unwrap_or_default();

        HeaderValue::from_str(&format!("max-age={seconds}"))
            .unwrap_or_else(|_| HeaderValue::from_static("max-age=0"))
    });

    let mut response = service.call(ctx).await?;
    let _ = response
        .headers_mut()
        .insert(header::CACHE_CONTROL, value.clone());

    Ok(response)
}

/// Logs the method, path and status of every request.
///
/// This is the `log`-flavoured counterpart of xitca-web's tracing based `Logger` middleware,
/// which would have forced the whole project onto a second logging facade.
pub async fn access_log<S, C, B>(
    service: &S,
    ctx: WebContext<'_, C, B>,
) -> Result<WebResponse, Error>
where
    S: for<'r> Service<WebContext<'r, C, B>, Response = WebResponse, Error = Error>,
{
    // The request is consumed by the inner service, so anything still wanted afterwards has
    // to be taken first. Cloning the method is free for the standard verbs, and the path is
    // only copied when the log line will actually be emitted, which is off by default.
    let logged = log::log_enabled!(log::Level::Info);
    let method = ctx.req().method().clone();
    let path = logged.then(|| ctx.req().uri().path().to_owned());

    let response = service.call(ctx).await?;

    if let Some(path) = path {
        log::info!("\"{method} {path}\" {}", response.status());
    }

    Ok(response)
}

/// Records the configured cache expiry once, so the header above is rendered a single time.
static CACHE_CONTROL_SECONDS: OnceLock<u8> = OnceLock::new();

/// Publishes the configured cache expiry to [`with_cache_control`].
///
/// # Arguments
///
/// * `seconds` - It is `http_cache_expiry_time` from the parsed config.
pub fn set_cache_expiry(seconds: u8) {
    let _ = CACHE_CONTROL_SECONDS.set(seconds);
}

/// Builds the permissive cross-origin policy the JSON API is served with.
///
/// xitca-web ships no CORS middleware of its own, so this is the `tower-http` layer bridged
/// in through xitca's `tower-http-compat` feature. It allows any origin, `GET` only, and the
/// four request headers a browser sets when a client calls the JSON API from a page.
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET])
        .allow_headers([
            header::ORIGIN,
            header::CONTENT_TYPE,
            header::REFERER,
            header::COOKIE,
        ])
}

/// Builds the quota for the configured window.
///
/// `http_rate` keeps the burst size and the replenishment period as separate knobs, which is
/// exactly what "N requests per T seconds" needs: a burst of the whole allowance, refilling
/// one cell every `T / N` seconds so the sustained rate is `N / T`. xitca's own `RateLimit`
/// only exposes constructors that take one knob or the other, never both, which is why the
/// limiter is built on `http_rate` directly.
///
/// # Arguments
///
/// * `time_limit` - It takes the number of seconds the rate is measured over.
/// * `number_of_requests` - It takes the number of requests allowed in that window.
///
/// # Returns
///
/// Returns the quota, or [`None`] when the configured window describes no allowance at all.
pub fn rate_quota(time_limit: u8, number_of_requests: u8) -> Option<Quota> {
    if time_limit == 0 || number_of_requests == 0 {
        return None;
    }

    let burst = u32::from(number_of_requests);

    // One cell is replenished every `time_limit / number_of_requests` seconds. The period is
    // kept at nanosecond resolution so a window such as 20 requests over 3 seconds keeps the
    // configured average instead of collapsing to whole seconds.
    let period = Duration::from_nanos(
        // `time_limit` is at most 255 seconds and the divisor at least 1, so the quotient
        // stays well inside a `u64` number of nanoseconds.
        ((u128::from(time_limit) * NANOS_PER_SECOND) / u128::from(number_of_requests)) as u64,
    );

    Quota::with_period(period).map(|quota| quota.allow_burst(burst))
}

/// The number of nanoseconds in a second, used to turn the rate limiter's window into a period.
const NANOS_PER_SECOND: u128 = 1_000_000_000;

/// Rejects requests that exceed the configured rate, per client address.
///
/// The decision comes from `http_rate`; this only plumbs the request through it and writes the
/// `x-ratelimit-*` headers back onto the response.
pub async fn rate_limit<S, C, B>(
    service: &S,
    ctx: WebContext<'_, C, B>,
) -> Result<WebResponse, Error>
where
    S: for<'r> Service<WebContext<'r, C, B>, Response = WebResponse, Error = Error>,
{
    let Some(quota) = rate_quota(
        RATE_WINDOW.load(Ordering::Relaxed),
        RATE_BUDGET.load(Ordering::Relaxed),
    ) else {
        return service.call(ctx).await;
    };

    let limiter = limiter(quota);

    let headers = ctx.req().headers();
    let addr = ctx.req().body().socket_addr();

    let Ok(snapshot) = limiter.rate_limit(headers, addr) else {
        // Over the limit: answer without running the route, still carrying the counters.
        let mut response: WebResponse = Response::new(ResponseBody::empty());

        *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
        let _ = response
            .headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static(PLAIN_TEXT));

        return Ok(response);
    };

    let mut response = service.call(ctx).await?;
    snapshot.extend_response(&mut response);

    Ok(response)
}

/// The process-wide limiter, built once from the configured quota.
///
/// Rebuilding it per request would throw away every client's accumulated state.
fn limiter(quota: Quota) -> &'static http_rate::RateLimit {
    static LIMITER: OnceLock<http_rate::RateLimit> = OnceLock::new();

    LIMITER.get_or_init(|| http_rate::RateLimit::new(quota))
}

/// The configured window in seconds, published by [`set_rate_limit`].
static RATE_WINDOW: AtomicU8 = AtomicU8::new(1);

/// The configured request budget, published by [`set_rate_limit`].
static RATE_BUDGET: AtomicU8 = AtomicU8::new(1);

/// Publishes the configured rate limit so the middleware can build its quota.
///
/// # Arguments
///
/// * `time_limit` - It is `rate_limiter.time_limit` from the parsed config.
/// * `number_of_requests` - It is `rate_limiter.number_of_requests` from the parsed config.
pub fn set_rate_limit(time_limit: u8, number_of_requests: u8) {
    RATE_WINDOW.store(time_limit, Ordering::Relaxed);
    RATE_BUDGET.store(number_of_requests, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The quota `rate_quota` is expected to build for a window.
    fn expected(time_limit: u8, number_of_requests: u8) -> Quota {
        let period = Duration::from_nanos(
            ((u128::from(time_limit) * NANOS_PER_SECOND) / u128::from(number_of_requests)) as u64,
        );

        Quota::with_period(period)
            .expect("the period is never zero")
            .allow_burst(u32::from(number_of_requests))
    }

    #[test]
    fn builds_the_configured_window_as_burst_plus_replenishment() {
        // The whole allowance is available at once and drains over the window, which is what
        // the previous limiter did and what `with_period` alone could not express.
        assert_eq!(rate_quota(3, 20), Some(expected(3, 20)));
    }

    #[test]
    fn keeps_sub_second_windows_precise() {
        // A window shorter than the burst must not collapse to whole seconds, which would
        // turn a permissive limit into a much stricter one: 255 requests over one second is
        // one per 3.9 ms, not one per second.
        assert_eq!(rate_quota(1, 255), Some(expected(1, 255)));
        assert_eq!(
            rate_quota(1, 255),
            Some(Quota::per_second(255).allow_burst(255))
        );
    }

    #[test]
    fn a_window_of_three_seconds_is_stricter_than_the_same_count_in_one() {
        // The same allowance spread over a longer window must be a different quota.
        assert_ne!(rate_quota(3, 60), rate_quota(1, 60));
    }

    #[test]
    fn windows_that_divide_evenly_need_no_rounding() {
        assert_eq!(rate_quota(3, 30), Some(expected(3, 30)));
        assert_eq!(rate_quota(60, 60), Some(expected(60, 60)));
    }

    #[test]
    fn a_window_with_no_allowance_produces_no_quota() {
        // Either value being zero means the limiter has nothing to enforce, and it must not
        // be turned into an accidental "one request per period" restriction.
        assert_eq!(rate_quota(0, 20), None);
        assert_eq!(rate_quota(3, 0), None);
        assert_eq!(rate_quota(0, 0), None);
    }

    #[test]
    fn reads_the_preferences_cookie_out_of_a_raw_header() {
        let raw = HeaderValue::from_static("theme=simple; appCookie={\"engines\":[\"brave\"]}");
        assert_eq!(
            cookie_value(Some(&raw), APP_COOKIE),
            Some(r#"{"engines":["brave"]}"#)
        );

        let other = HeaderValue::from_static("theme=simple");
        assert_eq!(cookie_value(Some(&other), APP_COOKIE), None);
        assert_eq!(cookie_value(None, APP_COOKIE), None);
    }
}
