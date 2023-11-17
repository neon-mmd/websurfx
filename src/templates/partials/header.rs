//!

use crate::templates::partials::navbar::navbar;
use maud::{html, Markup, PreEscaped, DOCTYPE};

///
pub fn header(colorscheme: &str, theme: &str) -> Markup {
    html!(
        (DOCTYPE)
        html lang="en"

        head{
            title{"Websurfx"}
            meta charset="UTF-8";
            meta name="viewport" content="width=device-width, initial-scale=1";
            link href=(format!("static/colorschemes/{colorscheme}.css")) rel="stylesheet" type="text/css";
            link href=(format!("static/themes/{theme}.css")) rel="stylesheet" type="text/css";
        }

        (PreEscaped("<body onload=\"getClientSettings()\">"))
            header{
                h1{a href="/"{"Websurfx"}}
                (navbar())
            }
    )
}
