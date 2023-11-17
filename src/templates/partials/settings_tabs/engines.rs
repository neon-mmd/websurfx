//!

use maud::{html, Markup};

///
pub fn engines(engine_names: &[&String]) -> Markup {
    html!(
        div class="engines tab"{
           h1{"Engines"}
           h3{"select search engines"}
           p class="description"{
              "Select the search engines from the list of engines that you want results from"
           }
           .engine_selection{
               .toggle_btn{
                  label class="switch"{
                     input type="checkbox" class="select_all" onchange="toggleAllSelection()";
                     span class="slider round"{}
                  }
                  "Select All"
               }
               hr;
               @for engine_name in engine_names{
                   .toggle_btn{
                       label class="switch"{
                          input type="checkbox" class="engine";
                          span class="slider round"{}
                       }
                       (format!("{}{}",engine_name[..1].to_uppercase().to_owned(), engine_name[1..].to_owned()))
                   }
               }
           }
        }
    )
}
