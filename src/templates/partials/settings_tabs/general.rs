//! A module that handles the general tab for setting page view in the `websurfx` frontend.

use maud::{html, Markup};

/// A constant holding the named safe search level options for the corresponding values 0, 1 and 2.
const SAFE_SEARCH_LEVELS: [(u8, &str); 3] = [(0, "None"), (1, "Low"), (2, "Moderate")];

/// A functions that handles the html code for the general tab for the settings page for the search page.
///
/// # Returns
///
/// It returns the compiled html markup code for the general tab.
pub fn general() -> Markup {
    html!(
        div class="general tab active"{
           h1{"General"}
           h3{"Select a safe search level"}
           p class="description"{
               "Select a safe search level from the menu below to filter content based on the level."
           }
           select name="safe_search_levels"{
               @for (k,v) in SAFE_SEARCH_LEVELS{
                 option value=(k){(v)}
               }
           }
        }
    )
}
