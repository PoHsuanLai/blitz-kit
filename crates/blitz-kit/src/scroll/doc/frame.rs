//! The documents of `iframe` elements: reaching a frame's document from its host element, and
//! the scrollers inside it as seen from the document that holds the frame.

use blitz_dom::{BaseDocument, Node, NodeId};

use super::chain::{Descend, candidates};
use crate::scroll::geom::{Framed, Inside, ScrollAxis, ViewPoint};
use crate::scroll::latch::Candidate;

/// Read the document of the `iframe` element `host` with `read`, if it has one.
pub(super) fn in_frame<T>(
    doc: &BaseDocument,
    host: u64,
    read: impl FnOnce(&BaseDocument) -> T,
) -> Option<T> {
    let sub = doc.get_node(NodeId::from_u64(host))?.subdoc()?;
    let inner = sub.inner();
    Some(read(&inner))
}

/// Write the document of the `iframe` element `host` with `write`, if it has one.
pub(super) fn in_frame_mut<T>(
    doc: &mut BaseDocument,
    host: u64,
    write: impl FnOnce(&mut BaseDocument) -> T,
) -> Option<T> {
    let sub = doc.get_node_mut(NodeId::from_u64(host))?.subdoc_mut()?;
    let mut inner = sub.inner_mut();
    Some(write(&mut inner))
}

/// The scrollers of the frame `host` is, under page point `(px, py)` of the document holding
/// it, innermost first. Empty when `host` is no frame.
pub(super) fn chain(host: &Node, (px, py): (f64, f64), axis: ScrollAxis) -> Vec<Candidate> {
    let Some(sub) = host.subdoc() else {
        return Vec::new();
    };
    let offset = host.absolute_position(0.0, 0.0);
    let at = ViewPoint {
        x: px - f64::from(offset.x),
        y: py - f64::from(offset.y),
    };
    let inner = sub.inner();
    candidates(&inner, at, axis, Descend::Stay)
        .into_iter()
        .filter_map(|c| {
            let inside = Inside::of(c.scroller)?;
            let scroller = crate::scroll::geom::Scroller::Framed(Framed {
                host: host.id.as_u64(),
                inside,
            });
            Some(Candidate { scroller, ..c })
        })
        .collect()
}
