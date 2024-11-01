//! A module that handles `bar` partial for the `search_bar` partial and the home/index/main page in the `websurfx` frontend.

use maud::{html, Markup, PreEscaped};

/// A functions that handles the html code for the bar for the `search_bar` partial and the
/// home/index/main page in the search engine frontend.
///
/// # Arguments
///
/// * `query` - It takes the current search query provided by user as an argument.
///
/// # Returns
///
/// It returns the compiled html code for the search bar as a result.
pub fn bar(query: &str) -> Markup {
    html!(
        (PreEscaped("<form action=\"/search\">"))
        (PreEscaped("<div class=\"search_bar\">"))
            input type="search" name="q" value=(query) placeholder="Type to search";
            button type="button" onclick="clearSearchText()" {
                img src="./images/close.svg" alt="Clear button icon for clearing search input text";
            }
            button type="submit" {
                img src="./images/magnifying_glass.svg" alt="Info icon for error box";
            }
    )
}
