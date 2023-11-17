//!

use maud::{html, Markup,PreEscaped};

///
pub fn bar(query: &str) -> Markup {
    html!(
        (PreEscaped("<div class=\"search_bar\">"))
            input type="search" name="search-box" value=(query) placeholder="Type to search";
            button type="submit" onclick="searchWeb()"{"search"}
    )
}
