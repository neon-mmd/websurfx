//! This module handles the settings and download route of the search engine website.

use crate::{
    Config,
    handler::{FileType, file_path},
    models::search_route,
    server::{
        APP_COOKIE, HTML, JSON, bad_request, cookie_value, internal_error, ok, preferences,
        with_app_cookie,
    },
};

use std::borrow::Cow;
use std::pin::pin;
use tokio::fs::read_dir;
use xitca_web::{
    WebContext,
    handler::multipart::Multipart,
    http::{WebResponse, header},
};

/// The file extension the uploaded settings file has to have.
const JSON_EXTENSION: &str = "json";

/// A helper function that helps in building the list of all available colorscheme/theme/animation
/// names present in the colorschemes, animations and themes folder respectively by excluding the
/// ones that have already been selected via the config file.
///
/// # Arguments
///
/// * `style_type` - It takes the style type of the values `theme` and `colorscheme` as an
///   argument.
///
/// # Error
///
/// Returns a list of colorscheme/theme names as a vector of tuple strings on success otherwise
/// returns a standard error message.
async fn style_option_list(style_type: &str) -> Result<Vec<String>, std::io::Error> {
    let mut style_options = Vec::new();
    let mut dir = read_dir(format!(
        "{}static/{}/",
        file_path(FileType::Theme).await?,
        style_type,
    ))
    .await?;

    while let Some(file) = dir.next_entry().await? {
        let style_name = file
            .file_name()
            .to_str()
            .unwrap_or_default()
            .replace(".css", "");
        style_options.push(style_name);
    }

    if style_type == "animations" {
        style_options.push(String::new());
    }

    Ok(style_options)
}

/// A helper function which santizes user provided json data from the input file.
///
/// # Arguments
///
/// * `config` - It takes the config struct as an argument.
/// * `setting_value` - It takes the cookie struct as an argument.
///
/// # Error
///
/// Returns a standard error message on failure.
async fn sanitize<'a>(
    config: &'a Config,
    setting_value: &mut search_route::Cookie<'a>,
) -> Result<(), std::io::Error> {
    // Check whether the theme, colorscheme and animation option is valid by matching it against
    // the available option list. If the option provided by the user via the JSON file is invalid
    // then replace the user provided by the default one used by the server via the config file.

    let themes = style_option_list("themes").await?;
    if !themes.iter().any(|theme| theme == &setting_value.theme) {
        setting_value.theme = Cow::Borrowed(&config.style.theme);
    } else {
        let colorschemes = style_option_list("colorschemes").await?;
        if !colorschemes
            .iter()
            .any(|colorscheme| colorscheme == &setting_value.colorscheme)
        {
            setting_value.colorscheme = Cow::Borrowed(&config.style.colorscheme);
        } else if let Some(animation) = setting_value.animation.clone() {
            let animations = style_option_list("animations").await?;
            if !animations.iter().any(|known| known == &animation) {
                setting_value.animation = config
                    .style
                    .animation
                    .as_ref()
                    .map(|style| Cow::Borrowed(style.as_str()));
            }
        }
    }

    // Filters out any engines in the list that are invalid by matching each engine against the
    // available engine list.
    let engines: Vec<_> = setting_value
        .engines
        .iter()
        .filter(|engine| {
            config
                .upstream_search_engines
                .iter()
                .any(|(name, _)| *engine == name.as_str())
        })
        .cloned()
        .collect();
    setting_value.engines = Cow::Owned(engines);

    setting_value.safe_search_level = if setting_value.safe_search_level < 2 {
        setting_value.safe_search_level
    } else {
        u8::default()
    };

    Ok(())
}

/// Reads the uploaded settings file out of the multipart body.
///
/// # Arguments
///
/// * `body` - It takes the multipart body of the request.
///
/// # Returns
///
/// Returns the file contents on success, or [`None`] when the body holds no usable upload.
async fn read_upload(body: Multipart) -> Option<String> {
    let mut body = pin!(body);

    while let Ok(Some(mut field)) = body.as_mut().try_next().await {
        let is_json = field
            .file_name()
            .is_some_and(|name| name.ends_with(JSON_EXTENSION));

        if !is_json {
            continue;
        }

        let mut data = String::new();
        while let Ok(Some(bytes)) = field.try_next().await {
            data.push_str(&String::from_utf8_lossy(bytes.as_ref()));
        }

        if !data.is_empty() {
            return Some(data);
        }
    }

    None
}

/// Handles the route of the post settings page.
pub async fn set_settings(
    ctx: &WebContext<'_, &'static Config>,
    multipart: Multipart,
) -> WebResponse {
    let config = ctx.state();

    let Some(data) = read_upload(multipart).await else {
        return bad_request(HTML, "No settings file was uploaded");
    };

    let mut preferences = match search_route::Cookie::from_json(&data) {
        Ok(preferences) => preferences,
        Err(e) => return bad_request(HTML, format!("Invalid settings file: {e}")),
    };

    if let Err(e) = sanitize(config, &mut preferences).await {
        return internal_error(e);
    }

    with_app_cookie(ok(HTML, String::new()), preferences.to_json())
}

/// Handles the route of the download page.
pub async fn download(ctx: &WebContext<'_, &'static Config>) -> WebResponse {
    // Get search settings using the user's cookie or from the server's config
    let preferences = preferences(
        cookie_value(ctx.req().headers().get(header::COOKIE), APP_COOKIE),
        ctx.state(),
        u8::default(),
    );

    ok(JSON, preferences.to_json())
}
