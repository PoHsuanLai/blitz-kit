//! The directories a consumer lets `file:` requests read from.

use std::path::{Path, PathBuf};

/// The directories a document may read local files from. A consumer names them (a wallpaper
/// cache, the icon directories); a path anywhere else, or one that only reaches a root through a
/// symlink or `..`, is refused. The default holds none, so a consumer that names nothing serves
/// no files.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileRoots(Vec<PathBuf>);

impl FileRoots {
    /// These roots. A relative path names no fixed place and is dropped.
    pub fn new(roots: impl IntoIterator<Item = PathBuf>) -> FileRoots {
        FileRoots(
            roots
                .into_iter()
                .filter(|root| root.is_absolute())
                .collect(),
        )
    }

    /// The real location of `path` (symlinks and `..` resolved) when a root holds it, `None`
    /// when no root does or the path does not exist. Whole path components are compared, so
    /// `/data/pics-evil` is not inside `/data/pics`.
    pub(super) fn admit(&self, path: &Path) -> Option<PathBuf> {
        let real = path.canonicalize().ok()?;
        self.0
            .iter()
            .filter_map(|root| root.canonicalize().ok())
            .any(|root| real.starts_with(root))
            .then_some(real)
    }
}
