//! The scrollers under a point.

use blitz_dom::{BaseDocument, Node};

use super::element::{ancestors, element_under, is_root, scrolls};
use super::frame;
use super::geometry::geom;
use crate::scroll::geom::{Elastic, ScrollAxis, Scroller, ViewPoint};
use crate::scroll::latch::Candidate;

/// Whether the chain goes into the documents of `iframe` elements it passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Descend {
    Into,
    Stay,
}

/// Page coordinates of surface-local `at`.
pub fn page_point(doc: &BaseDocument, at: ViewPoint) -> (f64, f64) {
    let scroll = doc.viewport_scroll();
    (at.x + scroll.x, at.y + scroll.y)
}

/// The scrollers on `axis` from the element under surface-local `at` outward, innermost first,
/// ending with the viewport when it can scroll or stretch on `axis`. Over an `iframe`, the
/// scrollers of its document come first, then those around the `iframe`.
pub fn chain_at(doc: &BaseDocument, at: ViewPoint, axis: ScrollAxis) -> Vec<Candidate> {
    candidates(doc, at, axis, Descend::Into)
}

pub(super) fn candidates(
    doc: &BaseDocument,
    at: ViewPoint,
    axis: ScrollAxis,
    descend: Descend,
) -> Vec<Candidate> {
    let page = page_point(doc, at);
    let mut chain: Vec<Candidate> = element_under(doc, page.0, page.1)
        .into_iter()
        .flat_map(|id| ancestors(doc, id))
        .flat_map(|node| {
            let within = match descend {
                Descend::Into => frame::chain(node, page, axis),
                Descend::Stay => Vec::new(),
            };
            within.into_iter().chain(own(doc, node, axis))
        })
        .collect();
    if let Some(geom) = geom(doc, Scroller::Viewport, axis)
        && (geom.max.0 > 0.0 || geom.elastic == Elastic::Elastic)
    {
        chain.push(Candidate {
            scroller: Scroller::Viewport,
            geom,
        });
    }
    chain
}

/// `node` as a scroller on `axis`, if it is one.
fn own(doc: &BaseDocument, node: &Node, axis: ScrollAxis) -> Option<Candidate> {
    if !node.is_element() || is_root(doc, node.id) || !scrolls(doc, node, axis) {
        return None;
    }
    let scroller = Scroller::Node(node.id.as_u64());
    geom(doc, scroller, axis).map(|geom| Candidate { scroller, geom })
}
