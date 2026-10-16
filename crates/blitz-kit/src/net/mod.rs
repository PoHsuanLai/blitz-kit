//! A `NetProvider` that serves local resources only: `data:` URIs and `file:` URLs. Blitz loads
//! `@font-face`, `mask-image` and background images only through a net provider; a shell surface
//! never fetches from the network, but the wallpaper is `background-image: url(file:///…)`.

mod roots;
mod source;

#[cfg(test)]
mod tests;

use blitz_traits::net::{Bytes, NetHandler, NetProvider, Request};

pub use roots::FileRoots;
pub use source::{LocalSource, MAX_RESOURCE_BYTES};

/// The local loader: `data:` decoded, and `file:` read from disk when its roots hold the file.
/// Anything else is left unanswered. Meant for documents the consumer wrote or trusts: it
/// refuses a file outside its roots and one over [`MAX_RESOURCE_BYTES`], but a document it
/// serves can still ask for any file inside them.
#[derive(Debug, Clone, Default)]
pub struct LocalNet {
    roots: FileRoots,
}

impl LocalNet {
    /// A loader that reads files from `roots` only.
    pub fn new(roots: FileRoots) -> LocalNet {
        LocalNet { roots }
    }
}

impl NetProvider for LocalNet {
    fn fetch(&self, _doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let resolved = request.url.as_str().to_owned();
        // Both run on the caller's runtime blocking pool (the host holds `rt.enter()` on its
        // main thread), so a large asset never stalls a frame.
        match LocalSource::of(&request.url) {
            LocalSource::Data(raw) => {
                tokio::task::spawn_blocking(move || {
                    let bytes = LocalSource::Data(raw).read().unwrap_or_default();
                    handler.bytes(resolved, Bytes::from(bytes));
                });
            }
            LocalSource::File(path) => {
                let roots = self.roots.clone();
                tokio::task::spawn_blocking(move || {
                    match LocalSource::File(path.clone()).read_within(&roots) {
                        Some(bytes) => handler.bytes(resolved, Bytes::from(bytes)),
                        None => eprintln!("blitz-kit: will not read {}", path.display()),
                    }
                });
            }
            // Nothing else is servable: leave the request unanswered rather than guess.
            LocalSource::Unservable => {}
        }
    }
}
