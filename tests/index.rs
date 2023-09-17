use std::net::TcpListener;

use handlebars::Handlebars;
use websurfx::{config::parser::Config, run};

// Starts a new instance of the HTTP server, bound to a random available port
fn spawn_app() -> String {
    // Binding to port 0 will trigger the OS to assign a port for us.
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let config = Config::parse(false).unwrap();
    let server = run(
        listener,
        config,
        #[cfg(all(feature = "memory-cache", not(feature = "redis-cache")))]
        websurfx::cache::cacher::Cache::new_in_memory(),
    )
    .expect("Failed to bind address");

    tokio::spawn(server);
    format!("http://127.0.0.1:{}/", port)
}

// Creates a new instance of Handlebars and registers the templates directory.
// This is used to compare the rendered template with the response body.
fn handlebars() -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();

    handlebars
        .register_templates_directory(".html", "./public/templates")
        .unwrap();

    handlebars
}

#[tokio::test]
async fn test_index() {
    let address = spawn_app();

    let client = reqwest::Client::new();
    let res = client.get(address).send().await.unwrap();
    assert_eq!(res.status(), 200);

    let handlebars = handlebars();
    let config = Config::parse(true).unwrap();
    let template = handlebars.render("index", &config.style).unwrap();
    assert_eq!(res.text().await.unwrap(), template);
}

// TODO: Write tests for testing parameters for search function that if provided with something
// other than u32 like alphabets and special characters than it should panic
