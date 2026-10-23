//! This module provides the functionality to handle theme folder present on different paths and
//! provide one appropriate path on which it is present and can be used.

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};
use tokio::{fs::try_exists, io::Error};

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
static FILE_PATHS_FOR_DIFF_FILE_TYPES: OnceLock<Vec<(FileType, Vec<PathBuf>)>> = OnceLock::new();

#[cfg(feature = "embedded")]
/// Initializes resource paths with a host application's data directory.
pub(crate) fn set_data_root(path: impl AsRef<Path>) -> Result<(), Error> {
    let home = std::env::var("HOME").unwrap_or_default();
    let path = path.as_ref();
    let root = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    FILE_PATHS_FOR_DIFF_FILE_TYPES
        .set(file_paths(&home, Some(&root)))
        .map_err(|_| {
            Error::new(
                tokio::io::ErrorKind::AlreadyExists,
                "data paths already initialized",
            )
        })
}

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
pub async fn file_path(file_type: FileType) -> Result<PathBuf, Error> {
    let home = std::env::var("HOME").unwrap_or_default();

    let paths = FILE_PATHS_FOR_DIFF_FILE_TYPES
        .get_or_init(|| file_paths(&home, None))
        .iter()
        .find_map(|(candidate, paths)| (candidate == &file_type).then_some(paths))
        .unwrap();

    for path in paths {
        if try_exists(path).await? {
            return Ok(path.to_path_buf());
        }
    }

    // if no of the configs above exist, return error
    Err(Error::new(
        tokio::io::ErrorKind::NotFound,
        format!("{:?} file/folder not found!!", file_type),
    ))
}

/// Builds resource lookup paths, preferring the optional host-provided root.
fn file_paths(home: &str, data_root: Option<&Path>) -> Vec<(FileType, Vec<PathBuf>)> {
    let mut config_paths = Vec::with_capacity(4);
    let mut theme_paths = Vec::with_capacity(3);
    let mut allowlist_paths = Vec::with_capacity(4);
    let mut blocklist_paths = Vec::with_capacity(4);

    if let Some(root) = data_root {
        config_paths.push(root.join(COMMON_DIRECTORY_NAME).join(CONFIG_FILE_NAME));
        theme_paths.push(root.join(PUBLIC_DIRECTORY_NAME));
        allowlist_paths.push(root.join(COMMON_DIRECTORY_NAME).join(ALLOWLIST_FILE_NAME));
        blocklist_paths.push(root.join(COMMON_DIRECTORY_NAME).join(BLOCKLIST_FILE_NAME));
    }

    config_paths.extend([
        PathBuf::from(format!(
            "{home}/.config/{COMMON_DIRECTORY_NAME}/{CONFIG_FILE_NAME}"
        )),
        PathBuf::from(format!(
            "/etc/xdg/{COMMON_DIRECTORY_NAME}/{CONFIG_FILE_NAME}"
        )),
        PathBuf::from(format!("./{COMMON_DIRECTORY_NAME}/{CONFIG_FILE_NAME}")),
    ]);
    theme_paths.extend([
        PathBuf::from(format!("/opt/websurfx/{PUBLIC_DIRECTORY_NAME}")),
        PathBuf::from(format!("./{PUBLIC_DIRECTORY_NAME}")),
    ]);
    allowlist_paths.extend([
        PathBuf::from(format!(
            "{home}/.config/{COMMON_DIRECTORY_NAME}/{ALLOWLIST_FILE_NAME}"
        )),
        PathBuf::from(format!(
            "/etc/xdg/{COMMON_DIRECTORY_NAME}/{ALLOWLIST_FILE_NAME}"
        )),
        PathBuf::from(format!("./{COMMON_DIRECTORY_NAME}/{ALLOWLIST_FILE_NAME}")),
    ]);
    blocklist_paths.extend([
        PathBuf::from(format!(
            "{home}/.config/{COMMON_DIRECTORY_NAME}/{BLOCKLIST_FILE_NAME}"
        )),
        PathBuf::from(format!(
            "/etc/xdg/{COMMON_DIRECTORY_NAME}/{BLOCKLIST_FILE_NAME}"
        )),
        PathBuf::from(format!("./{COMMON_DIRECTORY_NAME}/{BLOCKLIST_FILE_NAME}")),
    ]);

    vec![
        (FileType::Config, config_paths),
        (FileType::Theme, theme_paths),
        (FileType::AllowList, allowlist_paths),
        (FileType::BlockList, blocklist_paths),
    ]
}
