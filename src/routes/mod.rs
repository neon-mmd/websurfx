//! This module provides the functionality to handle different routes of the `websurfx`
//! meta search engine website and provide appropriate response to each route/page
//! when requested.

use std::sync::{Arc, OnceLock};

use tokio::fs::read_to_string;
use xitca_web::{
    WebContext,
    body::ResponseBody,
    handler::state::StateRef,
    http::{HeaderValue, Response, StatusCode, WebResponse, header},
};

use crate::{
    assets::{Directory, PublicFolder},
    handler::{FileType, file_path},
    parser::Config,
    server::{HTML, PLAIN_TEXT, internal_error, ok, opensearch, response},
};

pub mod export_import;
pub mod search;

/// Renders a page from the three style options the config holds.
///
/// # Arguments
///
/// * `render` - It takes a function that turns the colorscheme, theme and animation into the
///   markup of a page.
fn page(
    config: &'static Config,
    render: impl FnOnce(&str, &str, &Option<String>) -> maud::Markup,
) -> WebResponse {
    ok(
        HTML,
        render(
            &config.style.colorscheme,
            &config.style.theme,
            &config.style.animation,
        )
        .0,
    )
}

/// Handles the route of index page or main page of the `websurfx` meta search engine website.
pub async fn index(StateRef(config): StateRef<'_, &'static Config>) -> WebResponse {
    page(config, crate::templates::views::index::index)
}

/// Handles the route of any other accessed route/page which is not provided by the
/// website essentially the 404 error page.
pub async fn not_found(StateRef(config): StateRef<'_, &'static Config>) -> WebResponse {
    page(config, crate::templates::views::not_found::not_found)
}

/// Handles the route of robots.txt page of the `websurfx` meta search engine website.
pub async fn robots_data() -> WebResponse {
    let path = match file_path(FileType::Theme).await {
        Ok(path) => format!("{path}/robots.txt"),
        Err(error) => return internal_error(error),
    };

    match read_to_string(path).await {
        Ok(content) => ok(PLAIN_TEXT, content),
        Err(error) => internal_error(error),
    }
}

/// Handles the `/websurfx.xml` route that serves the OpenSearch description
/// document for the `websurfx` meta search engine, allowing browsers to
/// auto-discover and register it as a search provider.
pub async fn opensearch_description() -> WebResponse {
    let path = match file_path(FileType::Theme).await {
        Ok(path) => format!("{path}/websurfx.xml"),
        Err(error) => return internal_error(error),
    };

    match read_to_string(path).await {
        Ok(content) => opensearch(content),
        Err(error) => internal_error(error),
    }
}

/// Handles the route of about page of the `websurfx` meta search engine website.
pub async fn about(StateRef(config): StateRef<'_, &'static Config>) -> WebResponse {
    page(config, crate::templates::views::about::about)
}

/// Handles the route of settings page of the `websurfx` meta search engine website.
pub async fn settings(StateRef(config): StateRef<'_, &'static Config>) -> WebResponse {
    match crate::templates::views::settings::settings(
        config.safe_search,
        &config.style.colorscheme,
        &config.style.theme,
        &config.style.animation,
        &config.upstream_search_engines,
    )
    .await
    {
        Ok(markup) => ok(HTML, markup.0),
        Err(error) => internal_error(error),
    }
}

/// The public folder's assets, read once and served from memory.
///
/// The three static routes share this one store, so a file requested under more than one
/// prefix is only ever held once.
static ASSETS: OnceLock<Arc<PublicFolder>> = OnceLock::new();

/// Publishes the public folder to the static routes.
///
/// # Arguments
///
/// * `assets` - It is the store the static routes serve from.
pub fn set_assets(assets: Arc<PublicFolder>) {
    let _ = ASSETS.set(assets);
}

/// Serves a file from `/static`.
pub async fn static_files(ctx: &WebContext<'_, &'static Config>) -> WebResponse {
    static_file(Directory::Static, ctx).await
}

/// Serves a file from `/favicon`.
pub async fn favicon(ctx: &WebContext<'_, &'static Config>) -> WebResponse {
    static_file(Directory::Favicon, ctx).await
}

/// Serves a file from `/images`.
pub async fn images(ctx: &WebContext<'_, &'static Config>) -> WebResponse {
    static_file(Directory::Images, ctx).await
}

/// Serves one file from the public folder.
///
/// # Arguments
///
/// * `directory` - It is the directory the file was requested from.
async fn static_file(directory: Directory, ctx: &WebContext<'_, &'static Config>) -> WebResponse {
    let Some(assets) = ASSETS.get() else {
        return internal_error("the static assets have not been published");
    };

    // Everything after the route prefix names the file. A request that stops at the prefix
    // itself is naming the directory rather than a file.
    let path = ctx
        .req()
        .uri()
        .path()
        .strip_prefix(PublicFolder::prefix(directory))
        .unwrap_or_default();
    let path = path.trim_start_matches('/');

    if path.is_empty() {
        return response(StatusCode::NOT_FOUND, PLAIN_TEXT, "Not Found");
    }

    match assets.get(directory, path).await {
        Ok(Some(asset)) => {
            let mut response: WebResponse =
                Response::new(ResponseBody::bytes(asset.body().clone()));

            *response.status_mut() = StatusCode::OK;

            let _ = response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(asset.content_type()),
            );

            response
        }
        Ok(None) => response(StatusCode::NOT_FOUND, PLAIN_TEXT, "Not Found"),
        Err(error) => internal_error(error),
    }
}
