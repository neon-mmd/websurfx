//!

use crate::handler::paths::{file_path, FileType};
use maud::{html, Markup};
use std::fs::read_dir;

fn style_option_list(
    style_type: &str,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error + '_>> {
    let mut style_option_names: Vec<(String, String)> = Vec::new();
    for file in read_dir(format!(
        "{}static/{}/",
        file_path(FileType::Theme)?,
        style_type,
    ))? {
        let style_name = file?.file_name().to_str().unwrap().replace(".css", "");
        style_option_names.push((style_name.clone(), style_name.replace("-", " ")));
    }

    Ok(style_option_names)
}

///
pub fn user_interface() -> Result<Markup, Box<dyn std::error::Error>> {
    Ok(html!(
        div class="user_interface tab"{
           h1{"User Interface"}
           h3{"select theme"}
           p class="description"{
               "Select the theme from the available themes to be used in user interface"
           }
           select name="themes"{
               @for (k,v) in style_option_list("themes")?{
                   option value=(k){(v)}
               }
           }
           h3{"select color scheme"}
           p class="description"{
               "Select the color scheme for your theme to be used in user interface"
           }
           select name="colorschemes"{
               @for (k,v) in style_option_list("colorschemes")?{
                   option value=(k){(v)}
               }
           }
        }
    ))
}
