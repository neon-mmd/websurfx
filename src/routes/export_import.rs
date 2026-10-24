// Rust guideline compliant 2026-10-09

//! This module handles the settings and download route of the search engine website.

use crate::{
    Config,
    handler::{FileType, file_path},
    models::{self, search_route},
};
use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::{
    HttpRequest, HttpResponse,
    cookie::{
        Cookie, SameSite,
        time::{Duration, OffsetDateTime},
    },
    get, post, web,
};
use std::borrow::Cow;

use tokio::fs::read_dir;

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
async fn style_option_list(
    style_type: &str,
) -> Result<Box<[Cow<'_, str>]>, Box<dyn std::error::Error>> {
    let mut style_options = Vec::new();
    let mut dir = read_dir(format!(
        "{}static/{}/",
        file_path(FileType::Theme).await?,
        style_type,
    ))
    .await?;
    while let Some(file) = dir.next_entry().await? {
        if let Some(name_str) = file.file_name().to_str() {
            let style_name = name_str.replace(".css", "");
            style_options.push(Cow::Owned(style_name));
        }
    }

    if style_type == "animations" {
        style_options.push(Cow::default());
    }

    Ok(style_options.into_boxed_slice())
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
/// returns a standard error message on failure otherwise it returns the unit type.
pub(crate) async fn sanitize(
    config: web::Data<&'static Config>,
    setting_value: &mut models::search_route::Cookie<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check whether the theme, colorscheme and animation option is valid by matching it against
    // the available option list. If the option provided by the user via the JSON file is invalid
    // then replace the user provided by the default one used by the server via the config file.

    if !style_option_list("themes")
        .await?
        .contains(&setting_value.theme)
    {
        setting_value.theme = Cow::Borrowed(&config.style.theme);
    }

    if !style_option_list("colorschemes")
        .await?
        .contains(&setting_value.colorscheme)
    {
        setting_value.colorscheme = Cow::Borrowed(&config.style.colorscheme);
    }

    if let Some(animation) = setting_value.animation.as_ref()
        && !style_option_list("animations").await?.contains(animation)
    {
        setting_value.animation = config
            .style
            .animation
            .as_ref()
            .map(|str| Cow::Borrowed(str.as_str()));
    }

    // Filters out any engines in the list that are disabled or invalid by matching each engine against the
    // server-enabled engine list.
    let engines: Vec<_> = setting_value
        .engines
        .iter()
        .filter(|&engine| {
            config
                .upstream_search_engines
                .get(engine.as_ref())
                .copied()
                .unwrap_or(false)
        })
        .cloned()
        .collect();
    setting_value.engines = Cow::Owned(engines);

    setting_value.safe_search_level = setting_value.safe_search_level.min(4);

    Ok(())
}

/// A multipart struct which stores user provided input file data in memory.
#[derive(MultipartForm)]
struct File {
    /// It stores the input file data in memory with a 16 KiB size cap.
    #[multipart(limit = "16 KiB")]
    file: TempFile,
}

/// Handles the route of the post settings page.
#[post("/settings")]
pub async fn set_settings(
    req: HttpRequest,
    config: web::Data<&'static Config>,
    MultipartForm(form): MultipartForm<File>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    // CSRF defense: reject cross-site requests to settings modification endpoint
    if let Some(sec_fetch_site) = req.headers().get("sec-fetch-site")
        && sec_fetch_site == "cross-site"
    {
        return Ok(HttpResponse::Forbidden().body("Cross-site request blocked"));
    }

    if let Some(file_name) = form.file.file_name {
        let mut file_name_parts = file_name.split('.');
        if file_name_parts.clone().count() >= 2 && file_name_parts.next_back() == Some("json") {
            if form.file.size == 0 {
                return Ok(HttpResponse::BadRequest().finish());
            }

            let data = match tokio::fs::read_to_string(form.file.file.path()).await {
                Ok(content) => content,
                Err(_) => return Ok(HttpResponse::BadRequest().finish()),
            };

            let mut unsanitized_json_data: models::search_route::Cookie<'_> =
                match serde_json::from_str(&data) {
                    Ok(parsed) => parsed,
                    Err(_) => return Ok(HttpResponse::BadRequest().finish()),
                };

            sanitize(config, &mut unsanitized_json_data).await?;

            let sanitized_json_data: String = serde_json::json!(unsanitized_json_data).to_string();

            return Ok(HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-store, private"))
                .cookie(
                    Cookie::build("appCookie", sanitized_json_data)
                        .path("/")
                        .same_site(SameSite::Lax)
                        .expires(OffsetDateTime::now_utc().saturating_add(Duration::weeks(52)))
                        .finish(),
                )
                .finish());
        }
    }

    Ok(HttpResponse::Ok().finish())
}

/// Handles the route of the download page.
#[get("/download")]
pub async fn download(
    config: web::Data<&'static Config>,
    req: HttpRequest,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let cookie = req.cookie("appCookie");

    // Get search settings using the user's cookie or from the server's config
    let preferences: search_route::Cookie<'_> = cookie
        .as_ref()
        .and_then(|cookie_value| serde_json::from_str(cookie_value.value()).ok())
        .unwrap_or_else(|| {
            search_route::Cookie::build(
                &config.style,
                config
                    .upstream_search_engines
                    .iter()
                    .filter_map(|(engine, enabled)| {
                        enabled.then_some(Cow::Borrowed(engine.as_str()))
                    })
                    .collect(),
                u8::default(),
            )
        });

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store, private"))
        .json(preferences))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[tokio::test]
    async fn test_sanitize_preserves_safesearch_and_filters_disabled_engines() {
        let config = Box::leak(Box::new(Config::parse(true).await.unwrap()));
        let data = web::Data::new(config as &'static Config);

        // Test safe_search_level 0..=4 are preserved (not reset to 0)
        for level in 0..=4 {
            let mut cookie = models::search_route::Cookie {
                theme: Cow::Borrowed("simple"),
                colorscheme: Cow::Borrowed("catppuccin-mocha"),
                animation: None,
                engines: Cow::Owned(vec![
                    Cow::Borrowed("bing"),
                    Cow::Borrowed("nonexistent_engine_xyz"),
                ]),
                safe_search_level: level,
            };
            sanitize(data.clone(), &mut cookie).await.unwrap();
            assert_eq!(
                cookie.safe_search_level, level,
                "SafeSearch level {level} was incorrectly modified"
            );
            // nonexistent_engine_xyz should have been stripped out
            assert!(
                !cookie
                    .engines
                    .contains(&Cow::Borrowed("nonexistent_engine_xyz")),
                "Disabled or invalid engine was not filtered"
            );
        }

        // Test level 5 is clamped to 4
        let mut cookie = models::search_route::Cookie {
            theme: Cow::Borrowed("simple"),
            colorscheme: Cow::Borrowed("catppuccin-mocha"),
            animation: None,
            engines: Cow::Owned(vec![Cow::Borrowed("duckduckgo")]),
            safe_search_level: 5,
        };
        sanitize(data.clone(), &mut cookie).await.unwrap();
        assert_eq!(cookie.safe_search_level, 4);
    }
}
