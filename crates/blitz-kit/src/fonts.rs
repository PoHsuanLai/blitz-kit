//! One `FontContext` for every document: each gets a clone, so the source cache is shared and a
//! face registered once is available to every document built after it.
//!
//! A shell registers the faces its design system bundles at startup, before the first document
//! is built; without them every document falls back to fontconfig's faces.

use std::sync::Arc;

use blitz_dom::FontContext;
use parley::fontique::Blob;

/// The face files a shell's design system bundles, as `&'static` TTF/OTF bytes (typically
/// `include_bytes!` in the design-system crate).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FontFaces(pub Vec<&'static [u8]>);

/// The shared font context.
#[derive(Clone)]
pub struct SharedFonts {
    ctx: FontContext,
}

impl SharedFonts {
    /// System fonts (fontconfig, including CJK fallback) plus Blitz's bullet glyph.
    pub fn system() -> SharedFonts {
        let mut ctx = FontContext::new();
        // Shares the backing store across every clone handed to `for_document`, so a font file
        // loaded for one surface is not re-read from disk for the next.
        ctx.source_cache.make_shared();
        ctx.collection.register_fonts(
            Blob::new(Arc::new(blitz_dom::BULLET_FONT) as Arc<dyn AsRef<[u8]> + Send + Sync>),
            None,
        );
        SharedFonts { ctx }
    }

    /// [`SharedFonts::system`] plus every face in `faces`, registered once.
    pub fn system_with(faces: &FontFaces) -> SharedFonts {
        let mut fonts = SharedFonts::system();
        for bytes in &faces.0 {
            fonts.register(bytes);
        }
        fonts
    }

    /// Register a bundled face for every document, returning the family name(s) it registered
    /// under (empty if `bytes` carried no usable face). A face registered here is available to
    /// every document built from a clone of this context ([`SharedFonts::for_document`]) from
    /// this point on; a document built earlier does not see it.
    pub fn register(&mut self, bytes: &'static [u8]) -> Vec<String> {
        let registered = self.ctx.collection.register_fonts(
            Blob::new(Arc::new(bytes) as Arc<dyn AsRef<[u8]> + Send + Sync>),
            None,
        );
        registered
            .into_iter()
            .filter_map(|(id, _)| self.ctx.collection.family(id))
            .map(|info| info.name().to_owned())
            .collect()
    }

    /// A clone for a new document's `DocumentConfig::font_ctx`.
    pub fn for_document(&self) -> FontContext {
        self.ctx.clone()
    }
}

impl std::fmt::Debug for SharedFonts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedFonts").finish_non_exhaustive()
    }
}
