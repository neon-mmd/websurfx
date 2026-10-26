//! This module provides the tests for each page whether they work as intended or not.

use tokio::{net::TcpListener, sync::OnceCell};
use websurfx::{parser::Config, run, templates::views};

/// A static constant for holding the parsed config.
static CONFIG: OnceCell<Config> = OnceCell::const_new();

// Starts a new instance of the HTTP server, bound to a random available port
async fn spawn_app() -> String {
    // Binding to port 0 will trigger the OS to assign a port for us.
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

#[tokio::test]
async fn test_index() {
    let address = spawn_app().await;

    let client = reqwest::Client::new();
    let res = client.get(address).send().await.unwrap();
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

// TODO: Write tests for testing parameters for search function that if provided with something
// other than u32 like alphabets and special characters than it should panic
