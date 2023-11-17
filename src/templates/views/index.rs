//!

use maud::{html, Markup, PreEscaped};

use crate::templates::partials::{bar::bar, footer::footer, header::header};

///
pub fn index(colorscheme: &str, theme: &str, query: &str) -> Markup {
    html!(
        (header(colorscheme, theme))
        main class="search-container"{
            img src="../images/websurfx_logo.png" alt="Websurfx meta-search engine logo";
            (bar(query))
            (PreEscaped("</div>"))
        }
        script src="static/index.js"{}
        (footer())
    )
}
