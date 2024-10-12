//! A module that handles `search bar` partial for the search page in the `websurfx` frontend.

use maud::{html, Markup, PreEscaped};

use crate::{models::aggregation_models::EngineErrorInfo, templates::partials::bar::bar};

/// A constant holding the named safe search level options for the corresponding values 0, 1 and 2.
const SAFE_SEARCH_LEVELS_NAME: [&str; 3] = ["None", "Low", "Moderate"];

/// A functions that handles the html code for the search bar for the search page.
///
/// # Arguments
///
/// * `engine_errors_info` - It takes the engine errors list containing errors for each upstream
///   search engine which failed to provide results as an argument.
/// * `safe_search_level` - It takes the safe search level with values from 0-2 as an argument.
/// * `query` - It takes the current search query provided by user as an argument.
///
/// # Returns
///
/// It returns the compiled html code for the search bar as a result.
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
                      button type="button" onclick="toggleErrorBox()" class="error_box_toggle_button"{
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
                      button type="button" onclick="toggleErrorBox()" class="error_box_toggle_button"{
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
                   (PreEscaped("<select name=\"safesearch\" disabled>"))
               }
               @else{
                   (PreEscaped(format!("<select name=\"safesearch\" value=\"{}\">", safe_search_level)))
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
            (PreEscaped("</form>"))
        }
    )
}
