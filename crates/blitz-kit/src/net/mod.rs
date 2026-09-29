//! A `NetProvider` that serves local resources only: `data:` URIs and `file:` URLs. Blitz loads
//! `@font-face`, `mask-image` and background images only through a net provider; a shell surface
//! never fetches from the network, but the wallpaper is `background-image: url(file:///…)`.

mod data_url;
mod source;

#[cfg(test)]
mod tests;

use std::path::Path;

use blitz_traits::net::{Bytes, NetHandler, NetProvider, Request};

pub use source::LocalSource;

/// The local loader: `data:` decoded, `file:` read from disk. Anything else is left unanswered.
#[derive(Debug, Clone, Copy, Default)]
pub struct LocalNet;

impl NetProvider for LocalNet {
    fn fetch(&self, _doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let resolved = request.url.as_str().to_owned();
        // Both run on the caller's runtime blocking pool (the host holds `rt.enter()` on its
        // main thread), so a large asset never stalls a frame.
        match LocalSource::of(&request.url) {
            LocalSource::Data(raw) => {
                tokio::task::spawn_blocking(move || {
                    let bytes = data_url::decode(&raw).unwrap_or_default();
                    handler.bytes(resolved, Bytes::from(bytes));
                });
            }
            LocalSource::File(path) => {
                tokio::task::spawn_blocking(move || match read_file(&path) {
                    Some(bytes) => handler.bytes(resolved, Bytes::from(bytes)),
                    None => eprintln!("blitz-kit: cannot read {}", path.display()),
                });
            }
            // Nothing else is servable: leave the request unanswered rather than guess.
            LocalSource::Unservable => {}
        }
    }
}

/// The bytes of an absolute path; `None` when it cannot be read.
fn read_file(path: &Path) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}
