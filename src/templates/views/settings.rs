//!

use maud::{html, Markup};

use crate::templates::partials::{
    footer::footer,
    header::header,
    settings_tabs::{
        cookies::cookies, engines::engines, general::general, user_interface::user_interface,
    },
};

///
pub fn settings(
    colorscheme: &str,
    theme: &str,
    engine_names: &[&String],
) -> Result<Markup, Box<dyn std::error::Error>> {
    Ok(html!(
        (header(colorscheme, theme))
        main class="settings"{
           h1{"Settings"}
           hr;
           .settings_container{
              .sidebar{
                  div class="btn active" onclick="setActiveTab(this)"{"general"}
                  .btn onclick="setActiveTab(this)"{"user interface"}
                  .btn onclick="setActiveTab(this)"{"engines"}
                  .btn onclick="setActiveTab(this)"{"cookies"}
              }
              .main_container{
                  (general())
                  (user_interface()?)
                  (engines(engine_names))
                  (cookies())
                  p class="message"{}
                  button type="submit" onclick="setClientSettings()"{"Save"}
              }
           }
        }
        script src="static/settings.js"{}
        script src="static/cookies.js"{}
        (footer())
    ))
}
