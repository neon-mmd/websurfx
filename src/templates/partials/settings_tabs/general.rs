//! A module that handles the general tab for setting page view in the `websurfx` frontend.

use maud::{html, Markup};

/// A constant holding the named safe search level options for the corresponding values 0, 1 and 2.
const SAFE_SEARCH_LEVELS: [(u8, &str); 3] = [(0, "None"), (1, "Low"), (2, "Moderate")];

/// A functions that handles the html code for the general tab for the settings page for the search page.
///
/// # Arguments
///
/// * `safe_search_level` - It takes the safe search level as an argument.
///
/// # Returns
///
/// It returns the compiled html markup code for the general tab.
pub fn general(safe_search_level: u8) -> Markup {
    html!(
        div class="general tab active"{
           h1{"General"}
           h3{"Select a safe search level"}
           p class="description"{
               "Select a safe search level from the menu below to filter content based on the level."
           }
           @if safe_search_level < 3 {
               select name="safe_search_levels" {
                   // Sets the user selected safe_search_level name from the config file as the first option in the selection list.
                   option value=(safe_search_level){(SAFE_SEARCH_LEVELS.iter().find(|level| level.0 == safe_search_level).unwrap().1)}
                   @for (k,v) in SAFE_SEARCH_LEVELS.iter().filter(|level| level.0 != safe_search_level){
                     option value=(k){(v)}
                   }
               }
           }
           @else {
               p class="admin_warning" {"⚠️  This setting is being managed by the server administrator."}
               select name="safe_search_levels" disabled {
                     option value=(SAFE_SEARCH_LEVELS[2].0){(SAFE_SEARCH_LEVELS[2].1)}
               }
           }
        }
    )
}
