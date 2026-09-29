//! A document element's painted rectangle: its layout box through the chain of transforms.

use blitz_dom::{BaseDocument, NodeId};

use super::{Affine2, Placed, painted_bounds};
use crate::units::{Bounds, PagePoint};

/// Where the element `id` paints, in surface-local logical px: its layout box
/// (`get_client_bounding_rect`) carried through its own and its ancestors' CSS transforms;
/// `None` before layout or when absent. Read once per rendered frame, a region follows a
/// transform as it animates.
pub fn painted_rect(doc: &BaseDocument, id: NodeId) -> Option<Bounds> {
    let laid_out = doc.get_client_bounding_rect(id)?;
    Some(painted_bounds(
        Bounds {
            x: laid_out.x,
            y: laid_out.y,
            width: laid_out.width,
            height: laid_out.height,
        },
        &transform_chain(doc, id),
    ))
}

/// The transformed boxes from `id` up its layout parents (the chain Blitz's paint composes),
/// innermost first, each at its untransformed border-box origin in client coordinates: the
/// origin `get_client_bounding_rect` gives, less the box's own scroll offset, which Blitz's
/// `unrounded_absolute_position` subtracts at every level including the box itself.
fn transform_chain(doc: &BaseDocument, id: NodeId) -> Vec<Placed> {
    let scale = doc.viewport().scale_f64();
    std::iter::successors(Some(id), |&at| doc.get_node(at)?.layout_parent.get())
        .filter_map(|at| {
            let node = doc.get_node(at)?;
            // `Node::transform` panics on a text or comment node; only elements carry one.
            if !node.is_element() {
                return None;
            }
            let transform = node.transform().as_deref()?.as_coeffs();
            let rect = doc.get_client_bounding_rect(at)?;
            let scroll = node.scroll_offset();
            Some(Placed {
                origin: PagePoint {
                    x: rect.x + scroll.x,
                    y: rect.y + scroll.y,
                },
                transform: Affine2::from_device(transform, scale),
            })
        })
        .collect()
}
