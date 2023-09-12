//! This module provides the functionality to handle theme folder present on different paths and
//! provide one appropriate path on which it is present and can be used.

use std::collections::HashMap;
use std::io::Error;
use std::path::Path;
use std::sync::OnceLock;

// ------- Constants --------
/// The constant holding the name of the theme folder.
const PUBLIC_DIRECTORY_NAME: &str = "public";
/// The constant holding the name of the common folder.
const COMMON_DIRECTORY_NAME: &str = "websurfx";
/// The constant holding the name of the config file.
const CONFIG_FILE_NAME: &str = "config.lua";
/// The constant holding the name of the AllowList text file.
const ALLOWLIST_FILE_NAME: &str = "allowlist.txt";
/// The constant holding the name of the BlockList text file.
const BLOCKLIST_FILE_NAME: &str = "blocklist.txt";

/// An enum type which provides different variants to handle paths for various files/folders.
#[derive(Hash, PartialEq, Eq, Debug)]
pub enum FileType {
    /// This variant handles all the paths associated with the config file.
    Config,
    /// This variant handles all the paths associated with the Allowlist text file.
    AllowList,
    /// This variant handles all the paths associated with the BlockList text file.
    BlockList,
    /// This variant handles all the paths associated with the public folder (Theme folder).
    Theme,
}

/// A static variable which stores the different filesystem paths for various file/folder types.
static FILE_PATHS_FOR_DIFF_FILE_TYPES: OnceLock<HashMap<FileType, Vec<String>>> = OnceLock::new();

/// A function which returns an appropriate path for thr provided file type by checking if the path
/// for the given file type exists on that path.
///
/// # Error
///
/// Returns a `<File Name> folder/file not found!!` error if the give file_type folder/file is not
/// present on the path on which it is being tested.
///
/// # Example
///
/// If this function is give the file_type of Theme variant then the theme folder is checked by the
/// following steps:
///
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
        format!("{:?} file/folder not found!!", file_type),
    ))
}
