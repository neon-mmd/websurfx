use std::net::TcpListener;

use handlebars::Handlebars;
use websurfx::run;


// Starts a new instance of the HTTP server, bound to a random available port
fn spawn_app() -> String {
    // Binding to port 0 will trigger the OS to assign a port for us.
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind address");

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
    let template = handlebars.render("index", &()).unwrap();
    assert_eq!(res.text().await.unwrap(), template);
}