//!

use maud::{html, Markup, PreEscaped};

use crate::{models::aggregation_models::EngineErrorInfo, templates::partials::bar::bar};

const SAFE_SEARCH_LEVELS_NAME: [&str; 3] = ["None", "Low", "Moderate"];

///
pub fn search_bar(
    engine_errors_info: &[EngineErrorInfo],
    safe_search_level: u8,
    query: &str,
) -> Markup {
    html!(
        .search_area{
            (bar(query))
                .error_box {
                   @if !engine_errors_info.is_empty(){
                      button onclick="toggleErrorBox()" class="error_box_toggle_button"{
                         img src="./images/warning.svg" alt="Info icon for error box";
                      }
                      .dropdown_error_box{
                         @for errors in engine_errors_info{
                            .error_item{
                               span class="engine_name"{(errors.engine)}
                               span class="engine_name"{(errors.error)}
                               span class="severity_color" style="background: {{{this.severity_color}}};"{}
                            }
                         }
                      }
                   }
                   @else {
                      button onclick="toggleErrorBox()" class="error_box_toggle_button"{
                         img src="./images/info.svg" alt="Warning icon for error box";
                      }
                      .dropdown_error_box {
                         .no_errors{
                            "Everything looks good 🙂!!"
                         }
                      }
                  }
                }
            (PreEscaped("</div>"))
            .search_options {
               @if safe_search_level >= 3 {
                   (PreEscaped("<select name=\"safe_search_levels\" disabled>"))
               }
               @else{
                   (PreEscaped("<select name=\"safe_search_levels\">"))
               }
               @for (idx, name) in SAFE_SEARCH_LEVELS_NAME.iter().enumerate() {
                   @if (safe_search_level as usize) == idx {
                       option value=(idx) selected {(format!("SafeSearch: {name}"))}
                   }
                   @else{
                       option value=(idx) {(format!("SafeSearch: {name}"))}
                   }
               }
               (PreEscaped("</select>"))
            }
        }
    )
}
