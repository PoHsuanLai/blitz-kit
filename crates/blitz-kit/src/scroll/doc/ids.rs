//! Scrollers named by element `id`, and the boxes programmatic scrolls aim at.

use blitz_dom::{BaseDocument, NodeId};

use super::element::{ancestors, is_root, scrolls};
use crate::element_id::ElementId;
use crate::scroll::geom::{Area, ScrollAxis, Scroller};

/// The scroller the element with `id` names, if it exists.
pub fn scroller_by_id(doc: &BaseDocument, id: &ElementId) -> Option<Scroller> {
    let node = doc.get_element_by_id(id.as_str())?;
    Some(match is_root(doc, node) {
        true => Scroller::Viewport,
        false => Scroller::Node(node.as_u64()),
    })
}

/// The nearest ancestor of the element with `id` that scrolls on `axis` (the viewport if none),
/// and the element's own border box in surface-local coordinates.
pub fn container_of(
    doc: &BaseDocument,
    id: &ElementId,
    axis: ScrollAxis,
) -> Option<(Scroller, Area)> {
    let node = doc.get_element_by_id(id.as_str())?;
    let area = area(doc, Scroller::Node(node.as_u64()))?;
    let container = ancestors(doc, node)
        .skip(1)
        .find(|n| n.is_element() && !is_root(doc, n.id) && scrolls(doc, n, axis))
        .map_or(Scroller::Viewport, |n| Scroller::Node(n.id.as_u64()));
    Some((container, area))
}

/// `scroller`'s border box in surface-local coordinates (the viewport: the whole surface).
/// Blitz's client rect of an element subtracts the element's own scroll offset along with its
/// ancestors' (`unrounded_absolute_position`), which moves a scrolled container's box by its
/// own scroll; that is added back here.
pub fn area(doc: &BaseDocument, scroller: Scroller) -> Option<Area> {
    match scroller {
        // A frame's boxes are not aimed at by programmatic scrolls.
        Scroller::Framed(_) => None,
        Scroller::Viewport => {
            let (w, h) = doc.viewport().logical_size();
            Some(Area {
                x: 0.0,
                y: 0.0,
                width: f64::from(w),
                height: f64::from(h),
            })
        }
        Scroller::Node(id) => {
            let id = NodeId::from_u64(id);
            let rect = doc.get_client_bounding_rect(id)?;
            let own = doc.get_node(id)?.scroll_offset();
            Some(Area {
                x: rect.x + own.x,
                y: rect.y + own.y,
                width: rect.width,
                height: rect.height,
            })
        }
    }
}
