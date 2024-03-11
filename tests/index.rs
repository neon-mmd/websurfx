use std::{net::TcpListener, sync::OnceLock};

use websurfx::{config::parser::Config, run, templates::views};

/// A static constant for holding the parsed config.
static CONFIG: OnceLock<Config> = OnceLock::new();

// Starts a new instance of the HTTP server, bound to a random available port
async fn spawn_app() -> String {
    // Binding to port 0 will trigger the OS to assign a port for us.
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let config = CONFIG.get_or_init(|| Config::parse(false).unwrap());
    let cache = websurfx::cache::cacher::create_cache(config).await;
    let server = run(listener, config, cache).expect("Failed to bind address");

    tokio::spawn(server);
    format!("http://127.0.0.1:{}/", port)
}

#[tokio::test]
async fn test_index() {
    let address = spawn_app().await;

    let client = reqwest::Client::new();
    let res = client.get(address).send().await.unwrap();
    assert_eq!(res.status(), 200);

    let config = Config::parse(true).unwrap();
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
