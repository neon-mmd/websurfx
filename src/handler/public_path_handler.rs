//! This module provides the functionality to handle theme folder present on different paths and
//! provide one appropriate path on which it is present and can be used.

use std::io::Error;
use std::path::Path;

// ------- Constants --------
static PUBLIC_DIRECTORY_NAME: &str = "public";

/// A function which returns an appropriate theme directory path checking if the theme
/// directory exists on that path.
///
/// # Error
///
/// Returns a `Theme (public) folder not found!!` error if the theme folder is not present under following
/// paths which are:
/// 1. `/opt/websurfx` if it not present here then it fallbacks to the next one (2)
/// 2. Under project folder ( or codebase in other words) if it is not present
///    here then it returns an error as mentioned above.
pub fn handle_different_public_path() -> Result<String, Error> {
    if Path::new(format!("/opt/websurfx/{}/", PUBLIC_DIRECTORY_NAME).as_str()).exists() {
        Ok(format!("/opt/websurfx/{}", PUBLIC_DIRECTORY_NAME))
    } else if Path::new(format!("./{}/", PUBLIC_DIRECTORY_NAME).as_str()).exists() {
        Ok(format!("./{}", PUBLIC_DIRECTORY_NAME))
    } else {
        Err(Error::new(
            std::io::ErrorKind::NotFound,
            "Themes (public) folder not found!!",
        ))
    }
}
