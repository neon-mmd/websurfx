//! A module that handles the user interface tab for setting page view in the `websurfx` frontend.

use crate::handler::{file_path, FileType};
use maud::{html, Markup};
use std::fs::read_dir;

/// A helper function that helps in building the list of all available colorscheme/theme/animation
/// names present in the colorschemes, animations and themes folder respectively by excluding the
/// ones that have already been selected via the config file.
///
/// # Arguments
///
/// * `style_type` - It takes the style type of the values `theme` and `colorscheme` as an
///   argument.
/// * `selected_style` - It takes the currently selected style value provided via the config file
///   as an argument.
///
/// # Error
///
/// Returns a list of colorscheme/theme names as a vector of tuple strings on success otherwise
/// returns a standard error message.
fn style_option_list(
    style_type: &str,
    selected_style: &str,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut style_option_names: Vec<(String, String)> = Vec::new();
    for file in read_dir(format!(
        "{}static/{}/",
        file_path(FileType::Theme)?,
        style_type,
    ))? {
        let style_name = file?.file_name().to_str().unwrap().replace(".css", "");
        if selected_style != style_name {
            style_option_names.push((style_name.clone(), style_name.replace('-', " ")));
        }
    }

    if style_type == "animations" {
        style_option_names.push((String::default(), "none".to_owned()))
    }

    Ok(style_option_names)
}

/// A functions that handles the html code for the user interface tab for the settings page for the search page.
///
/// # Error
///
/// It returns the compiled html markup code for the user interface tab on success otherwise
/// returns a standard error message.
pub fn user_interface(
    theme: &str,
    colorscheme: &str,
    animation: &Option<String>,
) -> Result<Markup, Box<dyn std::error::Error>> {
    Ok(html!(
        div class="user_interface tab"{
           h1{"User Interface"}
           h3{"select theme"}
           p class="description"{
               "Select the theme from the available themes to be used in user interface"
           }
           select name="themes"{
               // Sets the user selected theme name from the config file as the first option in the selection list.
               option value=(theme){(theme.replace('-', " "))}
               @for (k,v) in style_option_list("themes", theme)?{
                   option value=(k){(v)}
               }
           }
           h3{"select color scheme"}
           p class="description"{
               "Select the color scheme for your theme to be used in user interface"
           }
           select name="colorschemes"{
               // Sets the user selected colorscheme name from the config file as the first option in the selection list.
               option value=(colorscheme){(colorscheme.replace('-', " "))}
               @for (k,v) in style_option_list("colorschemes", colorscheme)?{
                   option value=(k){(v)}
               }
           }
           h3{"select animation"}
           p class="description"{
               "Select the animation for your theme to be used in user interface"
           }
           select name="animations"{
               @let default_animation = &String::default();
               @let animation = animation.as_ref().unwrap_or(default_animation);
               // Sets the user selected animation name from the config file as the first option in the selection list.
               option value=(animation){(animation.replace('-'," "))}
               @for (k,v) in style_option_list("animations", animation)?{
                   option value=(k){(v)}
               }
           }
        }
    ))
}
