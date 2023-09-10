//! This module provides the functionality to handle theme folder present on different paths and
//! provide one appropriate path on which it is present and can be used.

use std::collections::HashMap;
use std::io::Error;
use std::path::Path;
use std::sync::OnceLock;

// ------- Constants --------
static PUBLIC_DIRECTORY_NAME: &str = "public";
static COMMON_DIRECTORY_NAME: &str = "websurfx";
static CONFIG_FILE_NAME: &str = "config.lua";
static ALLOWLIST_FILE_NAME: &str = "allowlist.txt";
static BLOCKLIST_FILE_NAME: &str = "blocklist.txt";

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum FileType {
    Config,
    AllowList,
    BlockList,
    Theme,
}

static FILE_PATHS_FOR_DIFF_FILE_TYPES: OnceLock<HashMap<FileType, Vec<String>>> = OnceLock::new();

/// A helper function which returns an appropriate config file path checking if the config
/// file exists on that path.
///
/// # Error
///
/// Returns a `config file not found!!` error if the config file is not present under following
/// paths which are:
/// 1. `~/.config/websurfx/` if it not present here then it fallbacks to the next one (2)
/// 2. `/etc/xdg/websurfx/config.lua` if it is not present here then it fallbacks to the next
///    one (3).
/// 3. `websurfx/` (under project folder ( or codebase in other words)) if it is not present
///    here then it returns an error as mentioned above.

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
pub fn file_path(file_type: FileType) -> Result<&'static str, Error> {
    let file_path: &Vec<String> = FILE_PATHS_FOR_DIFF_FILE_TYPES
        .get_or_init(|| {
            HashMap::from([
                (
                    FileType::Config,
                    vec![
                        format!(
                            "{}/.config/{}/{}",
                            std::env::var("HOME").unwrap(),
                            COMMON_DIRECTORY_NAME,
                            CONFIG_FILE_NAME
                        ),
                        format!("/etc/xdg/{}/{}", COMMON_DIRECTORY_NAME, CONFIG_FILE_NAME),
                        format!("./{}/{}", COMMON_DIRECTORY_NAME, CONFIG_FILE_NAME),
                    ],
                ),
                (
                    FileType::Theme,
                    vec![
                        format!("/opt/websurfx/{}/", PUBLIC_DIRECTORY_NAME),
                        format!("./{}/", PUBLIC_DIRECTORY_NAME),
                    ],
                ),
                (
                    FileType::AllowList,
                    vec![
                        format!(
                            "{}/.config/{}/{}",
                            std::env::var("HOME").unwrap(),
                            COMMON_DIRECTORY_NAME,
                            ALLOWLIST_FILE_NAME
                        ),
                        format!("/etc/xdg/{}/{}", COMMON_DIRECTORY_NAME, ALLOWLIST_FILE_NAME),
                        format!("./{}/{}", COMMON_DIRECTORY_NAME, ALLOWLIST_FILE_NAME),
                    ],
                ),
                (
                    FileType::BlockList,
                    vec![
                        format!(
                            "{}/.config/{}/{}",
                            std::env::var("HOME").unwrap(),
                            COMMON_DIRECTORY_NAME,
                            BLOCKLIST_FILE_NAME
                        ),
                        format!("/etc/xdg/{}/{}", COMMON_DIRECTORY_NAME, BLOCKLIST_FILE_NAME),
                        format!("./{}/{}", COMMON_DIRECTORY_NAME, BLOCKLIST_FILE_NAME),
                    ],
                ),
            ])
        })
        .get(&file_type)
        .unwrap();

    for (idx, _) in file_path.iter().enumerate() {
        if Path::new(file_path[idx].as_str()).exists() {
            return Ok(std::mem::take(&mut &*file_path[idx]));
        }
    }

    // if no of the configs above exist, return error
    Err(Error::new(
        std::io::ErrorKind::NotFound,
        format!("{:?} file not found!!", file_type),
    ))
}
