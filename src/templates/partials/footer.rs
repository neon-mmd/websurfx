//!

use maud::{html, Markup, PreEscaped};

///
pub fn footer() -> Markup {
    html!(
        footer{
           div{
              span{"Powered By "b{"Websurfx"}}span{"-"}span{"a lightening fast, privacy respecting, secure meta
                  search engine"}
           }
           div{
              ul{
                  li{a href="https://github.com/neon-mmd/websurfx"{"Source Code"}}
                  li{a href="https://github.com/neon-mmd/websurfx/issues"{"Issues/Bugs"}}
              }
           }
        }
        script src="static/settings.js"{}
        (PreEscaped("</body>"))
        (PreEscaped("</html>"))
    )
}
