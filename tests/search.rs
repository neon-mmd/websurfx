//! Integration tests for the search endpoint and user agent stability.

use tokio::{net::TcpListener, sync::OnceCell};
use websurfx::{parser::Config, run};

/// A static constant for holding the parsed config.
static CONFIG: OnceCell<Config> = OnceCell::const_new();

/// Starts a new instance of the HTTP server bound to a random available port.
async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let config = CONFIG
        .get_or_try_init(|| async move {
            Config::parse(false)
                .await
                .map_err(|e| tokio::io::Error::other(e.to_string()))
        })
        .await
        .unwrap();
    let server = run(listener, config).await.expect("Failed to bind address");

    tokio::spawn(server);
    format!("http://127.0.0.1:{}/", port)
}

/// Tests that sending an unauthenticated search query does not crash or abort the server.
///
/// This reproduces the conditions of Audit Finding #1 where `GET /search?q=rust&json=true`
/// previously crashed the server due to empty user agent lists and release panic aborts.
#[tokio::test]
async fn test_search_unauthenticated_does_not_abort_server() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // 1. Initial health check: server is up and returns 200 for index
    let res = client.get(&address).send().await.unwrap();
    assert_eq!(res.status(), 200);

    // 2. Unauthenticated search request: must not crash or abort the server
    let search_res = client
        .get(format!("{address}search?q=rust&json=true"))
        .send()
        .await;

    // The request should succeed in reaching the server (not connection reset / empty reply)
    assert!(search_res.is_ok());

    // 3. Post-search health check: verify the server is still running and responsive
    let post_res = client.get(&address).send().await.unwrap();
    assert_eq!(post_res.status(), 200);
}
