//!

use maud::{html, Markup};

const SAFE_SEARCH_LEVELS: [(u8, &'static str); 3] = [(0, "None"), (1, "Low"), (2, "Moderate")];

///
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
