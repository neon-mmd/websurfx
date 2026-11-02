//! This module serves the public folder's static assets out of memory.
//!
//! The framework's own file service opens the file and streams it back for every request,
//! with a hop through the blocking pool for the open, one for the metadata and one per
//! chunk. For the small files that make up the theme that dominates the request by two
//! orders of magnitude: serving `/static/index.js` measured 96 requests a second against
//! 66,000 for a dynamically rendered page. The public folder is a handful of megabytes that
//! does not change while the server runs, so each file is read once and then served from
//! memory, sharing one allocation across every request for it.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, RwLock};

use tokio::io::AsyncReadExt;
use xitca_web::bytes::Bytes;

/// The media type sent for a file extension.
fn content_type(extension: &str) -> &'static str {
    match extension {
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" | "map" => "application/json",
        "webmanifest" => "application/manifest+json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "txt" => "text/plain; charset=utf-8",
        "xml" => "application/xml",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

/// One file from the public folder, held in memory.
#[derive(Debug)]
pub struct Asset {
    /// The file's bytes. Cloning the [`Arc`] around this shares them across requests.
    body: Bytes,
    /// The media type derived from the file's extension.
    content_type: &'static str,
}

impl Asset {
    /// The bytes of the file.
    pub fn body(&self) -> &Bytes {
        &self.body
    }

    /// The media type of the file.
    pub fn content_type(&self) -> &'static str {
        self.content_type
    }
}

/// The public folder's files, read on first use and kept for the life of the process.
#[derive(Debug)]
pub struct Assets {
    /// The folder every request path is resolved under.
    root: PathBuf,
    /// The files read so far, keyed by request path.
    files: RwLock<HashMap<String, Arc<Asset>>>,
}

impl Assets {
    /// Creates a store that resolves paths under `root`.
    ///
    /// # Arguments
    ///
    /// * `root` - It is the folder the static directories live in.
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            files: RwLock::new(HashMap::new()),
        }
    }

    /// Resolves a request path to a file inside the root, or [`None`] when it escapes.
    ///
    /// Only plain name components are allowed, so `..`, an absolute path or a prefix all
    /// resolve to [`None`] rather than reaching outside the folder.
    ///
    /// # Arguments
    ///
    /// * `path` - It is the request path with its route prefix already removed.
    fn resolve(&self, path: &str) -> Option<PathBuf> {
        // An empty path has no components at all, which would satisfy the check below
        // vacuously and resolve to the root directory rather than to a file.
        if path.is_empty() {
            return None;
        }

        let relative = Path::new(path);

        if relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        {
            Some(self.root.join(relative))
        } else {
            None
        }
    }

    /// Returns the asset for `path`, reading it from disk the first time it is asked for.
    ///
    /// # Arguments
    ///
    /// * `path` - It is the request path with its route prefix already removed.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but could not be read.
    pub async fn get(&self, path: &str) -> Result<Option<Arc<Asset>>, std::io::Error> {
        if let Some(asset) = self
            .files
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .get(path)
        {
            return Ok(Some(Arc::clone(asset)));
        }

        // A path that escapes the root is a miss, not an error, and is never cached.
        let Some(file) = self.resolve(path) else {
            return Ok(None);
        };

        let mut handle = match tokio::fs::File::open(&file).await {
            Ok(handle) => handle,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };

        let mut bytes = Vec::new();
        handle.read_to_end(&mut bytes).await?;

        let extension = file
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();

        let asset = Arc::new(Asset {
            body: Bytes::from(bytes),
            content_type: content_type(extension),
        });

        self.files
            .write()
            .unwrap_or_else(|error| error.into_inner())
            .insert(path.to_owned(), Arc::clone(&asset));

        Ok(Some(asset))
    }
}

/// One of the directories mounted under the public folder.
#[derive(Debug, Clone, Copy)]
pub enum Directory {
    /// The theme's css and js, mounted at `/static`.
    Static,
    /// The site's icons, mounted at `/favicon`.
    Favicon,
    /// The page illustrations, mounted at `/images`.
    Images,
}

/// The three directories mounted under the public folder, each with its own cache.
///
/// The directories are disjoint, so three stores never hold the same file twice, and a request
/// looks a file up under a single-directory key with nothing to build.
#[derive(Debug)]
pub struct PublicFolder {
    /// The store backing `/static`.
    static_files: Assets,
    /// The store backing `/favicon`.
    favicon: Assets,
    /// The store backing `/images`.
    images: Assets,
}

impl PublicFolder {
    /// Creates a store for each directory under `root`.
    ///
    /// # Arguments
    ///
    /// * `root` - It is the folder the static directories live in.
    pub fn new(root: &Path) -> Self {
        Self {
            static_files: Assets::new(root.join("static")),
            favicon: Assets::new(root.join("favicon")),
            images: Assets::new(root.join("images")),
        }
    }

    /// The route a directory is mounted at.
    ///
    /// # Arguments
    ///
    /// * `directory` - It is the directory being served.
    pub fn prefix(directory: Directory) -> &'static str {
        match directory {
            Directory::Static => "/static",
            Directory::Favicon => "/favicon",
            Directory::Images => "/images",
        }
    }

    /// Returns the asset at `path` inside `directory`, reading it on first use.
    ///
    /// # Arguments
    ///
    /// * `directory` - It is the directory being served.
    /// * `path` - It is the request path with its route prefix already removed.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but could not be read.
    pub async fn get(
        &self,
        directory: Directory,
        path: &str,
    ) -> Result<Option<Arc<Asset>>, std::io::Error> {
        match directory {
            Directory::Static => self.static_files.get(path).await,
            Directory::Favicon => self.favicon.get(path).await,
            Directory::Images => self.images.get(path).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A store rooted at an empty temporary directory.
    fn empty_store() -> Assets {
        Assets::new(PathBuf::from("/nonexistent-websurfx-root"))
    }

    #[test]
    fn maps_known_extensions_to_media_types() {
        assert_eq!(content_type("css"), "text/css; charset=utf-8");
        assert_eq!(content_type("js"), "text/javascript; charset=utf-8");
        assert_eq!(content_type("png"), "image/png");
        assert_eq!(content_type("webmanifest"), "application/manifest+json");
        assert_eq!(content_type("bin"), "application/octet-stream");
    }

    #[test]
    fn resolves_plain_paths_inside_the_root() {
        let store = empty_store();

        assert_eq!(
            store.resolve("index.js"),
            Some(PathBuf::from("/nonexistent-websurfx-root/index.js"))
        );
        assert_eq!(
            store.resolve("themes/simple.css"),
            Some(PathBuf::from(
                "/nonexistent-websurfx-root/themes/simple.css"
            ))
        );
    }

    #[test]
    fn refuses_paths_that_leave_the_root() {
        let store = empty_store();

        for path in ["../secret", "themes/../../secret", "/etc/passwd", ""] {
            assert_eq!(store.resolve(path), None, "{path} should not resolve");
        }
    }

    #[tokio::test]
    async fn a_missing_file_is_a_miss_rather_than_an_error() {
        let store = empty_store();

        assert!(store.get("nope.js").await.is_ok());
        assert!(store.get("../escape.js").await.unwrap().is_none());
    }

    #[test]
    fn maps_each_directory_to_its_route_and_its_own_root() {
        let folder = PublicFolder::new(Path::new("/srv"));

        assert_eq!(PublicFolder::prefix(Directory::Static), "/static");
        assert_eq!(PublicFolder::prefix(Directory::Favicon), "/favicon");
        assert_eq!(PublicFolder::prefix(Directory::Images), "/images");

        assert_eq!(
            folder.static_files.resolve("index.js"),
            Some(PathBuf::from("/srv/static/index.js"))
        );
        assert_eq!(
            folder.images.resolve("robot-404.svg"),
            Some(PathBuf::from("/srv/images/robot-404.svg"))
        );
    }

    #[tokio::test]
    async fn serves_a_real_file_and_reuses_it() {
        let store = Assets::new(std::env::temp_dir().join("websurfx-assets-test"));

        let file = store.root.join("sample.css");
        tokio::fs::create_dir_all(&store.root).await.ok();
        tokio::fs::write(&file, b"body { color: red }").await.ok();

        let first = store.get("sample.css").await.unwrap().unwrap();
        let second = store.get("sample.css").await.unwrap().unwrap();

        assert_eq!(first.body().as_ref(), b"body { color: red }");
        assert_eq!(first.content_type(), "text/css; charset=utf-8");
        // The second lookup has to come from the cache, sharing the first read's allocation.
        assert!(Arc::ptr_eq(&first, &second));

        tokio::fs::remove_dir_all(&store.root).await.ok();
    }
}
