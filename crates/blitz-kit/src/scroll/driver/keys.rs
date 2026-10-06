//! Keyboard scrolling: scroll keys on the focused or remembered scroller (design/11 §11.3.10).

use blitz_dom::BaseDocument;

use super::{KeyRepeat, KeyUse, Moves, ScrollDriver};
use crate::scroll::doc::{self, KeyFocus};
use crate::scroll::engine::{ScrollAnimate, ScrollIn};
use crate::scroll::geom::{Latch, Px, ScrollAxis, Scroller, ViewPoint};
use crate::scroll::keys::{self, ScrollKey};
use crate::scroll::tuning::Env;

impl ScrollDriver {
    /// A scroll key went down (`KeyRepeat::First`) or repeated. `Passed` (the document's) when
    /// something on the focus path handles keys or nothing around the focus scrolls.
    pub fn key(
        &mut self,
        doc: &mut BaseDocument,
        key: ScrollKey,
        repeat: KeyRepeat,
        pointer: Option<ViewPoint>,
        env: Env<'_>,
    ) -> KeyUse {
        let axis = key.axis();
        let scroller = match doc::key_focus(doc, axis) {
            KeyFocus::Document => return KeyUse::Passed,
            KeyFocus::Scroller(scroller) => Some(scroller),
            KeyFocus::Free => self.key_target(doc, axis, pointer),
        };
        let Some(scroller) = scroller else {
            return KeyUse::Passed;
        };
        let latch = Latch { scroller, axis };
        let Some(geom) = doc::geom(doc, scroller, axis) else {
            return KeyUse::Scrolled(Moves::none());
        };
        let input = match (key, repeat) {
            (ScrollKey::Line(_, dir), KeyRepeat::First) => ScrollIn::Step {
                latch,
                by: Px(dir.sign() * keys::LINE_PX),
            },
            (ScrollKey::Line(_, dir), KeyRepeat::Repeated) => ScrollIn::Hold {
                latch,
                step: Px(dir.sign() * keys::LINE_PX),
            },
            (ScrollKey::Page(dir), _) => ScrollIn::Step {
                latch,
                by: Px(dir.sign() * keys::page(geom.viewport.0)),
            },
            (ScrollKey::Edge(dir), _) => ScrollIn::Jump {
                latch,
                to: Px(geom.edge(dir)),
                animate: ScrollAnimate::Smooth,
            },
        };
        KeyUse::Scrolled(self.run(doc, input, env).took())
    }

    /// The keyboard's scroller when nothing has focus: the last gesture's scroller if it is
    /// still under the pointer, else the innermost one under the pointer with content to scroll
    /// on `axis`, else the viewport if it has; `None` (the key is the document's) otherwise.
    fn key_target(
        &self,
        doc: &BaseDocument,
        axis: ScrollAxis,
        pointer: Option<ViewPoint>,
    ) -> Option<Scroller> {
        let chain = pointer
            .map(|at| doc::chain_at(doc, at, axis))
            .unwrap_or_default();
        let remembered = self
            .last_gesture
            .filter(|latch| latch.axis == axis)
            .map(|latch| latch.scroller)
            .filter(|scroller| chain.iter().any(|c| c.scroller == *scroller));
        remembered
            .or_else(|| {
                chain
                    .iter()
                    .find(|c| c.geom.max.0 > 0.0)
                    .map(|c| c.scroller)
            })
            .or_else(|| {
                doc::geom(doc, Scroller::Viewport, axis)
                    .filter(|g| g.max.0 > 0.0)
                    .map(|_| Scroller::Viewport)
            })
    }

    /// A held scroll key was released.
    pub fn key_up(&mut self, doc: &mut BaseDocument, env: Env<'_>) -> Moves {
        self.run(doc, ScrollIn::Release, env)
    }
}
