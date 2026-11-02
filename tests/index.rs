//! This module provides the tests for each page whether they work as intended or not.

use std::time::Duration;

use tokio::net::TcpListener;
use websurfx::{parser::Config, serve, templates::views};

/// How long to wait for the freshly spawned server to start accepting connections.
const STARTUP_DELAY: Duration = Duration::from_millis(250);

/// Starts a new instance of the HTTP server, bound to a random available port.
///
/// The server is driven on its own thread with its own runtime rather than being spawned onto
/// the test's runtime, so that a test can serve requests while its own async work continues.
///
/// # Returns
///
/// Returns the base address the server is reachable at.
async fn spawn_app() -> String {
    // Binding to port 0 will trigger the OS to assign a port for us.
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener
        .local_addr()
        .expect("Failed to read local address")
        .port();

    let address = format!("http://127.0.0.1:{port}/");

    std::thread::spawn(move || {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build the server runtime")
            .block_on(async move {
                // `true` skips the logger install, which can only happen once per process
                // and would otherwise panic for every test after the first.
                let config = Config::parse(true).await.expect("Failed to parse config");

                // The server only returns when it stops, so an error here means the test
                // cannot make its requests. Asserting keeps the failure next to the cause.
                let result = serve(listener, config).await;
                assert!(result.is_ok(), "the server stopped: {result:?}");
            })
    });

    // Give the server a moment to bind before the first request.
    tokio::time::sleep(STARTUP_DELAY).await;

    address
}

#[tokio::test]
async fn test_index() {
    let address = spawn_app().await;

    let res = reqwest::Client::new().get(&address).send().await.unwrap();
    assert_eq!(res.status(), 200);

    let config = Config::parse(true).await.unwrap();
    let template = views::index::index(
        &config.style.colorscheme,
        &config.style.theme,
        &config.style.animation,
    )
    .0;
    assert_eq!(res.text().await.unwrap(), template);
}

#[tokio::test]
async fn test_index_sends_a_cache_control_header() {
    let address = spawn_app().await;

    let res = reqwest::Client::new().get(&address).send().await.unwrap();

    let cache_control = res
        .headers()
        .get(reqwest::header::CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    assert!(
        cache_control.starts_with("max-age="),
        "expected a max-age directive, got {cache_control:?}"
    );
}

#[tokio::test]
async fn test_index_sends_cors_headers() {
    let address = spawn_app().await;

    let res = reqwest::Client::new()
        .get(&address)
        .header(reqwest::header::ORIGIN, "https://example.com")
        .send()
        .await
        .unwrap();

    assert_eq!(
        res.headers()
            .get(reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok()),
        Some("*")
    );
}

#[tokio::test]
async fn test_unknown_route_renders_the_404_page() {
    let address = spawn_app().await;

    let res = reqwest::Client::new()
        .get(format!("{address}does-not-exist"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);

    let body = res.text().await.unwrap();
    assert!(body.contains("404 Page Not Found!"), "got {body}");
    assert!(body.contains("error_container"), "got {body}");
}

#[tokio::test]
async fn test_robots_txt_is_served_as_plain_text() {
    let address = spawn_app().await;

    let res = reqwest::Client::new()
        .get(format!("{address}robots.txt"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    assert_eq!(
        res.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("text/plain; charset=utf-8")
    );
}

#[tokio::test]
async fn test_search_rejects_an_unusable_page_parameter() {
    let address = spawn_app().await;

    let res = reqwest::Client::new()
        .get(format!("{address}search?q=rust&page=zero&json=true"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);

    let body = res.text().await.unwrap();
    assert!(body.contains(r#""code":"400""#), "got {body}");
    assert!(body.contains("page"), "got {body}");
}

#[tokio::test]
async fn test_search_with_an_empty_query_is_a_bad_request_in_json_mode() {
    let address = spawn_app().await;

    let res = reqwest::Client::new()
        .get(format!("{address}search?q=&json=true"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
    assert_eq!(
        res.text().await.unwrap(),
        r#"{"code":"400","error":"Empty query provided"}"#
    );
}

#[tokio::test]
async fn test_download_returns_the_preferences_as_json() {
    let address = spawn_app().await;

    let res = reqwest::Client::new()
        .get(format!("{address}download"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    assert_eq!(
        res.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/json")
    );

    let body = res.text().await.unwrap();
    for member in [
        "theme",
        "colorscheme",
        "engines",
        "safe_search_level",
        "animation",
    ] {
        assert!(
            body.contains(&format!("\"{member}\"")),
            "{member} missing from {body}"
        );
    }
}

#[tokio::test]
async fn test_static_assets_are_served() {
    let address = spawn_app().await;

    // `index.js` is a file the search page always references.
    let res = reqwest::Client::new()
        .get(format!("{address}static/index.js"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
}
