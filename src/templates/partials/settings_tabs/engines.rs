//! The engines settings tab view for configuring search engines.

use std::collections::HashMap;

use maud::{Markup, html};

/// Formats an engine name with an uppercase first letter without panicking on empty or UTF-8 strings.
fn format_engine_name(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// A functions that handles the html code for the engines tab for the settings page for the search page.
///
/// # Arguments
///
/// * `engine_names` - It takes the key value pair list of all available engine names and there corresponding
///   selected (enabled/disabled) value as an argument.
///
/// # Returns
///
/// It returns the compiled html markup code for the engines tab.
pub fn engines(engine_names: &HashMap<String, bool>) -> Markup {
    html!(
        div class="engines tab"{
           h1{"Engines"}
           h3{"select search engines"}
           p class="description"{
              "Select the search engines from the list of engines that you want results from"
           }
           .engine_selection{
               // Checks whether all the engines are selected or not if they are then the
               // checked `select_all` button is rendered otherwise the unchecked version
               // is rendered.
               @if engine_names.values().all(|selected| *selected){
                   .toggle_btn{
                      label class="switch"{
                         input type="checkbox" class="select_all" onchange="toggleAllSelection()" checked;
                         span class="slider round"{}
                      }
                      "Select All"
                   }
               }
               @else{
                   .toggle_btn {
                      label class="switch"{
                         input type="checkbox" class="select_all" onchange="toggleAllSelection()";
                         span class="slider round"{}
                      }
                      "Select All"
                   }
               }
               hr;
               @for (engine_name, selected) in engine_names{
                   @let label_text = format_engine_name(engine_name);
                   // Checks whether the `engine_name` is selected or not if they are then the
                   // checked `engine` button is rendered otherwise the unchecked version is
                   // rendered.
                   @if *selected {
                       .toggle_btn{
                           label class="switch"{
                              input type="checkbox" class="engine" checked;
                              span class="slider round"{}
                           }
                           (label_text)
                       }
                   }
                   @else {
                       .toggle_btn {
                           label class="switch"{
                              input type="checkbox" class="engine";
                              span class="slider round"{}
                           }
                           (label_text)
                       }
                   }
               }
           }
        }
    )
}
