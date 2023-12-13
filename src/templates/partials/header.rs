//! A module that handles the header for all the pages in the `websurfx` frontend.

use crate::templates::partials::navbar::navbar;
use maud::{html, Markup, PreEscaped, DOCTYPE};

/// A function that handles the html code for the header for all the pages in the search engine frontend.
///
/// # Arguments
///
/// * `colorscheme` - It takes the colorscheme name as an argument.
/// * `theme` - It takes the theme name as an argument.
///
/// # Returns
///
/// It returns the compiled html markup code for the header as a result.
pub fn header(colorscheme: &str, theme: &str, animation: &Option<String>) -> Markup {
    html!(
        (DOCTYPE)
        html lang="en"

        head{
            title{"Websurfx"}
            meta charset="UTF-8";
            meta name="viewport" content="width=device-width, initial-scale=1";
            link href=(format!("static/colorschemes/{colorscheme}.css")) rel="stylesheet" type="text/css";
            link href=(format!("static/themes/{theme}.css")) rel="stylesheet" type="text/css";
            @if animation.is_some() {
                    link href=(format!("static/animations/{}.css", animation.as_ref().unwrap())) rel="stylesheet" type="text/css";
            }
        }

        (PreEscaped("<body onload=\"getClientSettings()\">"))
            header{
                h1{a href="/"{"Websurfx"}}
                (navbar())
            }
    )
}
