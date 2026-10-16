//! What a request asks the local loader for.

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use blitz_traits::net::Url;

use super::roots::FileRoots;

/// The most bytes one local resource may have: a wallpaper or a font fits, a device or a
/// runaway file does not, so a request cannot exhaust memory.
pub const MAX_RESOURCE_BYTES: u64 = 32 * 1024 * 1024;

/// What a request asks [`super::LocalNet`] for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalSource {
    /// A `data:` URI, decoded in full.
    Data(String),
    /// An absolute path named by a `file:` URL (empty host or `localhost`).
    File(PathBuf),
    /// A scheme or a path this provider does not serve.
    Unservable,
}

impl LocalSource {
    /// The bytes this source names, read on the calling thread: a `data:` URI decoded, a regular
    /// file read from disk. `None` when the payload is malformed or over [`MAX_RESOURCE_BYTES`],
    /// the file unreadable, not a regular file (a symlink, a device, a pipe) or unservable. The
    /// file may be anywhere; [`LocalSource::read_within`] limits it to a consumer's roots.
    pub fn read(&self) -> Option<Vec<u8>> {
        match self {
            LocalSource::Data(raw) => read_data(raw),
            LocalSource::File(path) => read_regular(path, MAX_RESOURCE_BYTES),
            LocalSource::Unservable => None,
        }
    }

    /// [`LocalSource::read`], reading a file only when `roots` hold it.
    pub fn read_within(&self, roots: &FileRoots) -> Option<Vec<u8>> {
        match self {
            LocalSource::File(path) => read_regular(&roots.admit(path)?, MAX_RESOURCE_BYTES),
            LocalSource::Data(_) | LocalSource::Unservable => self.read(),
        }
    }

    /// Classify a request URL.
    pub fn of(url: &Url) -> LocalSource {
        match url.scheme() {
            "data" => LocalSource::Data(url.as_str().to_owned()),
            "file" => match url.to_file_path() {
                Ok(path) if path.is_absolute() => LocalSource::File(path),
                Ok(_) | Err(()) => LocalSource::Unservable,
            },
            _ => LocalSource::Unservable,
        }
    }
}

fn read_data(raw: &str) -> Option<Vec<u8>> {
    if u64::try_from(raw.len()).is_ok_and(|len| len > MAX_RESOURCE_BYTES) {
        return None;
    }
    crate::data_url::decode(raw)
}

/// The contents of the regular file at `path`, or `None` when it is anything else, unreadable or
/// longer than `cap`. The file is checked before it is opened (a pipe would block the open) and
/// again once open, and read through `take`, so a file that grows or lies about its size still
/// stops at `cap`.
pub(super) fn read_regular(path: &Path, cap: u64) -> Option<Vec<u8>> {
    if !fs::symlink_metadata(path).ok()?.is_file() {
        return None;
    }
    let file = File::open(path).ok()?;
    if !file.metadata().ok()?.is_file() {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(cap.saturating_add(1))
        .read_to_end(&mut bytes)
        .ok()?;
    u64::try_from(bytes.len())
        .is_ok_and(|len| len <= cap)
        .then_some(bytes)
}
