//! Integration tests verifying security hardening and audit remediations.
//!
//! Tests covers HTTP 404 status codes, query length limits, security headers,
//! directory listing restrictions, and cross-site request forgery defenses.

use reqwest::header::HeaderMap;
use tokio::{net::TcpListener, sync::OnceCell};
use websurfx::{parser::Config, run};

/// A static cell holding the parsed server configuration.
static CONFIG: OnceCell<Config> = OnceCell::const_new();

/// Starts an instance of the Websurfx HTTP server bound to a random port.
///
/// # Returns
///
/// Returns the base URL string of the running test server instance.
async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let config = CONFIG
        .get_or_try_init(|| async move {
            Config::parse(false)
                .await
                .map_err(|e| std::io::Error::other(e.to_string()))
        })
        .await
        .unwrap();
    let server = run(listener, config).await.expect("Failed to bind address");

    tokio::spawn(server);
    format!("http://127.0.0.1:{}/", port)
}

/// Tests that accessing a nonexistent path returns HTTP 404 Not Found.
///
/// Remediates Audit Finding #6 where 404 handler incorrectly returned HTTP 200 OK.
#[tokio::test]
async fn test_404_handler_returns_not_found_status() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}nonexistent_route_404_check"))
        .send()
        .await
        .unwrap();

    assert_eq!(
        res.status(),
        reqwest::StatusCode::NOT_FOUND,
        "Expected HTTP 404 for nonexistent route, got {}",
        res.status()
    );
}

/// Tests that search queries exceeding 512 characters return HTTP 400 Bad Request.
///
/// Remediates Audit Finding #7 where unbounded query strings could consume excessive memory.
#[tokio::test]
async fn test_query_length_cap_exceeded_returns_bad_request() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let oversized_query = "a".repeat(513);

    // Test HTML request
    let res = client
        .get(format!("{address}search?q={oversized_query}"))
        .send()
        .await
        .unwrap();

    assert_eq!(
        res.status(),
        reqwest::StatusCode::BAD_REQUEST,
        "Expected HTTP 400 for oversized query, got {}",
        res.status()
    );

    // Test JSON request
    let json_res = client
        .get(format!("{address}search?q={oversized_query}&json=true"))
        .send()
        .await
        .unwrap();

    assert_eq!(
        json_res.status(),
        reqwest::StatusCode::BAD_REQUEST,
        "Expected HTTP 400 for oversized JSON query, got {}",
        json_res.status()
    );

    let body_str = json_res.text().await.unwrap();
    let body: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(body["code"], "400");
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("maximum allowed length of 512 characters")
    );
}

/// Tests that standard security headers are applied to HTTP responses.
///
/// Remediates Audit Findings #14 and #16 (CSP, nosniff, DENY, strict-origin).
#[tokio::test]
async fn test_security_headers_present() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let res = client.get(&address).send().await.unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::OK);

    let headers: &HeaderMap = res.headers();

    assert!(
        headers.contains_key("content-security-policy"),
        "Missing Content-Security-Policy header"
    );
    assert_eq!(
        headers
            .get("x-content-type-options")
            .map(|v| v.to_str().unwrap()),
        Some("nosniff"),
        "Missing or incorrect X-Content-Type-Options header"
    );
    assert_eq!(
        headers.get("x-frame-options").map(|v| v.to_str().unwrap()),
        Some("DENY"),
        "Missing or incorrect X-Frame-Options header"
    );
    assert_eq!(
        headers.get("referrer-policy").map(|v| v.to_str().unwrap()),
        Some("strict-origin-when-cross-origin"),
        "Missing or incorrect Referrer-Policy header"
    );
    assert!(
        headers.contains_key("cache-control"),
        "Missing Cache-Control header on index response"
    );
}

/// Tests that directory listing is disabled on static file endpoints.
///
/// Remediates Audit Finding #14 where directory indexing was enabled.
#[tokio::test]
async fn test_static_directory_listing_disabled() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}static/"))
        .send()
        .await
        .unwrap();

    // Directory listing must NOT return 200 with directory contents
    assert_ne!(
        res.status(),
        reqwest::StatusCode::OK,
        "Directory listing should not return HTTP 200 OK"
    );
}

/// Tests that cross-site requests to the settings modification endpoint are blocked.
///
/// Remediates Audit Finding #4 / CSRF defense on /settings.
#[tokio::test]
async fn test_csrf_rejection_on_cross_site_settings() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let boundary = "---------------------------974767299852498929531610575";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"settings.json\"\r\nContent-Type: application/json\r\n\r\n{{}}\r\n--{boundary}--\r\n"
    );

    let res = client
        .post(format!("{address}settings"))
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header("sec-fetch-site", "cross-site")
        .body(body)
        .send()
        .await
        .unwrap();

    assert_eq!(
        res.status(),
        reqwest::StatusCode::FORBIDDEN,
        "Expected HTTP 403 Forbidden for cross-site settings modification, got {}",
        res.status()
    );
}

/// Tests that multi-byte Unicode queries count characters rather than raw bytes.
///
/// Verifies that a 500-character multi-byte query (>1000 bytes) is accepted,
/// while a 513-character multi-byte query is rejected with HTTP 400 Bad Request.
#[tokio::test]
async fn test_multibyte_unicode_query_length_handling() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // 500 Arabic letters: each character is 2 bytes (1000 bytes total), but 500 characters.
    let valid_multibyte_query: String = "ع".repeat(500);
    assert_eq!(valid_multibyte_query.chars().count(), 500);
    assert!(valid_multibyte_query.len() > 512);

    let res = client
        .get(format!("{address}search?q={valid_multibyte_query}"))
        .send()
        .await
        .unwrap();

    // Should NOT be 400 Bad Request because character count <= 512
    assert_ne!(
        res.status(),
        reqwest::StatusCode::BAD_REQUEST,
        "500-character multi-byte query should not be rejected as oversized"
    );

    // 513 Arabic letters: 513 characters, exceeds limit.
    let oversized_multibyte_query: String = "ع".repeat(513);
    assert_eq!(oversized_multibyte_query.chars().count(), 513);

    let bad_res = client
        .get(format!("{address}search?q={oversized_multibyte_query}"))
        .send()
        .await
        .unwrap();

    assert_eq!(
        bad_res.status(),
        reqwest::StatusCode::BAD_REQUEST,
        "513-character multi-byte query must be rejected with HTTP 400"
    );
}

/// Tests that same-site and untrusted-origin requests to settings are rejected.
#[tokio::test]
async fn test_csrf_rejection_on_same_site_and_untrusted_origin() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let boundary = "---------------------------974767299852498929531610575";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"settings.json\"\r\nContent-Type: application/json\r\n\r\n{{}}\r\n--{boundary}--\r\n"
    );

    // 1. Same-site rejection
    let same_site_res = client
        .post(format!("{address}settings"))
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header("sec-fetch-site", "same-site")
        .body(body.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(
        same_site_res.status(),
        reqwest::StatusCode::FORBIDDEN,
        "Expected HTTP 403 Forbidden for same-site settings modification"
    );

    // 2. Untrusted origin without sec-fetch-site
    let untrusted_origin_res = client
        .post(format!("{address}settings"))
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header("origin", "https://untrusted.attacker.com")
        .body(body)
        .send()
        .await
        .unwrap();

    assert_eq!(
        untrusted_origin_res.status(),
        reqwest::StatusCode::FORBIDDEN,
        "Expected HTTP 403 Forbidden for untrusted origin settings modification"
    );
}
