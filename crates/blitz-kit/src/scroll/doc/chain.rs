//! The scrollers under a point.

use blitz_dom::BaseDocument;

use super::element::{ancestors, element_under, is_root, scrolls};
use super::geometry::geom;
use crate::scroll::geom::{Elastic, ScrollAxis, Scroller, ViewPoint};
use crate::scroll::latch::Candidate;

/// Page coordinates of surface-local `at`.
pub fn page_point(doc: &BaseDocument, at: ViewPoint) -> (f64, f64) {
    let scroll = doc.viewport_scroll();
    (at.x + scroll.x, at.y + scroll.y)
}

/// The scrollers on `axis` from the element under surface-local `at` outward, innermost first,
/// ending with the viewport when it can scroll or stretch on `axis`.
pub fn chain_at(doc: &BaseDocument, at: ViewPoint, axis: ScrollAxis) -> Vec<Candidate> {
    let (px, py) = page_point(doc, at);
    let mut chain: Vec<Candidate> = element_under(doc, px, py)
        .into_iter()
        .flat_map(|id| ancestors(doc, id))
        .filter(|node| node.is_element() && !is_root(doc, node.id) && scrolls(doc, node, axis))
        .filter_map(|node| {
            let scroller = Scroller::Node(node.id.as_u64());
            geom(doc, scroller, axis).map(|geom| Candidate { scroller, geom })
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
