//! Verifies the public library embedding API with host-provided data files.
#![cfg(feature = "embedded")]

use std::{ffi::OsString, fs, io};
use tokio::net::TcpListener;
use websurfx::{parser::Config, run, set_data_root};

#[tokio::test]
async fn embedded_library_uses_host_data_root() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    let app_dir = temp_dir.path().join("app");
    let other_dir = temp_dir.path().join("other");
    fs::create_dir_all(&app_dir).unwrap();
    fs::create_dir_all(&other_dir).unwrap();

    #[cfg(target_os = "linux")]
    let relative_root = {
        use std::os::unix::ffi::OsStringExt;
        OsString::from_vec(b"websurfx-data-\xff".to_vec())
    };
    #[cfg(not(target_os = "linux"))]
    let relative_root = OsString::from("websurfx-data");

    let data_root = app_dir.join(&relative_root);
    let config_dir = data_root.join("websurfx");
    let static_dir = data_root.join("public/static");
    let themes_dir = static_dir.join("themes");
    let colorschemes_dir = static_dir.join("colorschemes");
    let animations_dir = static_dir.join("animations");
    let favicon_dir = data_root.join("public/favicon");
    let images_dir = data_root.join("public/images");

    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&themes_dir).unwrap();
    fs::create_dir_all(&colorschemes_dir).unwrap();
    fs::create_dir_all(&animations_dir).unwrap();
    fs::create_dir_all(&favicon_dir).unwrap();
    fs::create_dir_all(&images_dir).unwrap();

    let host_config =
        include_str!("../websurfx/config.lua").replace("port = \"8080\"", "port = \"9191\"");
    assert!(host_config.contains("port = \"9191\""));
    fs::write(config_dir.join("config.lua"), host_config).unwrap();
    fs::write(config_dir.join("allowlist.txt"), "").unwrap();
    fs::write(config_dir.join("blocklist.txt"), "").unwrap();
    fs::write(data_root.join("public/robots.txt"), "host robots file\n").unwrap();
    fs::write(
        data_root.join("public/websurfx.xml"),
        "host OpenSearch description\n",
    )
    .unwrap();
    fs::write(static_dir.join("host-app.txt"), "host static file\n").unwrap();
    fs::write(favicon_dir.join("host-icon.txt"), "host favicon\n").unwrap();
    fs::write(images_dir.join("host-image.txt"), "host image\n").unwrap();
    fs::write(themes_dir.join("host-theme.css"), "").unwrap();
    fs::write(colorschemes_dir.join("host-colors.css"), "").unwrap();
    fs::write(animations_dir.join("host-animation.css"), "").unwrap();

    std::env::set_current_dir(&app_dir).unwrap();
    set_data_root(&relative_root).unwrap();
    std::env::set_current_dir(&other_dir).unwrap();
    let config = Box::leak(Box::new(Config::parse(false).await.unwrap()));
    assert_eq!(config.port, 9191);
    assert_eq!(
        set_data_root(data_root).unwrap_err().kind(),
        io::ErrorKind::AlreadyExists
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = run(listener, config).await.unwrap();
    let server_handle = server.handle();
    let server_task = tokio::spawn(server);

    let robots = reqwest::get(format!("http://{address}/robots.txt"))
        .await
        .unwrap();
    assert_eq!(robots.status(), reqwest::StatusCode::OK);
    assert_eq!(robots.text().await.unwrap(), "host robots file\n");

    let opensearch = reqwest::get(format!("http://{address}/websurfx.xml"))
        .await
        .unwrap();
    assert_eq!(opensearch.status(), reqwest::StatusCode::OK);
    assert_eq!(
        opensearch.text().await.unwrap(),
        "host OpenSearch description\n"
    );

    let static_file = reqwest::get(format!("http://{address}/static/host-app.txt"))
        .await
        .unwrap();
    assert_eq!(static_file.status(), reqwest::StatusCode::OK);
    assert_eq!(static_file.text().await.unwrap(), "host static file\n");

    let favicon = reqwest::get(format!("http://{address}/favicon/host-icon.txt"))
        .await
        .unwrap();
    assert_eq!(favicon.status(), reqwest::StatusCode::OK);
    assert_eq!(favicon.text().await.unwrap(), "host favicon\n");

    let image = reqwest::get(format!("http://{address}/images/host-image.txt"))
        .await
        .unwrap();
    assert_eq!(image.status(), reqwest::StatusCode::OK);
    assert_eq!(image.text().await.unwrap(), "host image\n");

    let settings = reqwest::get(format!("http://{address}/settings"))
        .await
        .unwrap();
    assert_eq!(settings.status(), reqwest::StatusCode::OK);
    assert!(settings.text().await.unwrap().contains("host-theme"));

    server_handle.stop(true).await;
    server_task.await.unwrap().unwrap();
    std::env::set_current_dir(original_dir).unwrap();
}
