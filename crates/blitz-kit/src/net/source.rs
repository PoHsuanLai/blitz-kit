//! What a request asks the local loader for.

use std::path::PathBuf;

use blitz_traits::net::Url;

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
    /// The bytes this source names, read on the calling thread: a `data:` URI decoded, a file
    /// read from disk. `None` when the payload is malformed, the file unreadable or the source
    /// unservable.
    pub fn read(&self) -> Option<Vec<u8>> {
        match self {
            LocalSource::Data(raw) => crate::data_url::decode(raw),
            LocalSource::File(path) => std::fs::read(path).ok(),
            LocalSource::Unservable => None,
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
