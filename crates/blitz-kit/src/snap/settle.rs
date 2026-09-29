//! Re-deriving each node's transform and scrollable overflow from its snapped layout.

use blitz_dom::{BaseDocument, Node, NodeId};

use super::grid::DeviceGrid;
use super::place::layout_children;

/// A box in device px: left, top, right, bottom.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Extent {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl Extent {
    fn union(self, other: Extent) -> Extent {
        Extent {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        }
    }

    /// The bounding box of `self` through the affine map `[a, b, c, d, e, f]` (kurbo's
    /// coefficient order).
    fn through(self, [a, b, c, d, e, f]: [f64; 6]) -> Extent {
        let corners = [
            (self.x0, self.y0),
            (self.x1, self.y0),
            (self.x0, self.y1),
            (self.x1, self.y1),
        ]
        .map(|(x, y)| (a * x + c * y + e, b * x + d * y + f));
        let fold = |pick: fn(&(f64, f64)) -> f64, join: fn(f64, f64) -> f64, start: f64| {
            corners.iter().map(pick).fold(start, join)
        };
        Extent {
            x0: fold(|p| p.0, f64::min, f64::INFINITY),
            y0: fold(|p| p.1, f64::min, f64::INFINITY),
            x1: fold(|p| p.0, f64::max, f64::NEG_INFINITY),
            y1: fold(|p| p.1, f64::max, f64::NEG_INFINITY),
        }
    }
}

const NOTHING: Extent = Extent {
    x0: 0.0,
    y0: 0.0,
    x1: 0.0,
    y1: 0.0,
};

/// Re-derive `id`'s transform and scrollable overflow from its snapped layout, as Blitz's
/// `resolve_transforms` did from the logical one; the box's painted bounds in its parent.
pub(super) fn settle(doc: &mut BaseDocument, id: NodeId, grid: DeviceGrid) -> Extent {
    let mut children = layout_children(doc, id);
    let Some(node) = doc.get_node_mut(id) else {
        return NOTHING;
    };
    children.extend(node.before());
    children.extend(node.after());
    let transform = node
        .set_transform(grid.0 as f32)
        .map(|t| whole_translation(t.as_coeffs(), node));
    let layout = *node.final_layout();
    let scale = grid.0;
    let mut overflow = Extent {
        x1: f64::from(layout.size.width) * scale,
        y1: f64::from(layout.size.height) * scale,
        ..NOTHING
    };
    for child in children {
        overflow = overflow.union(settle(doc, child, grid));
    }
    let Some(node) = doc.get_node_mut(id) else {
        return NOTHING;
    };
    let slot = node.scrollable_overflow_mut();
    (slot.x0, slot.y0, slot.x1, slot.y1) = (overflow.x0, overflow.y0, overflow.x1, overflow.y1);
    let [a, b, c, d, e, f] = transform.unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    let (dx, dy) = (
        f64::from(layout.location.x) * scale,
        f64::from(layout.location.y) * scale,
    );
    overflow.through([a, b, c, d, e + dx, f + dy])
}

/// A pure translation (`coeffs` of the node's transform) moved to whole device pixels and
/// written back to the node; any other transform (a scale, a rotation, a motion part-way
/// through) is left as it is. Returns the transform now in force.
fn whole_translation(coeffs: [f64; 6], node: &mut Node) -> [f64; 6] {
    let [a, b, c, d, e, f] = coeffs;
    if (a, b, c, d) != (1.0, 0.0, 0.0, 1.0) {
        return coeffs;
    }
    if let Some(slot) = node.transform_mut().as_deref_mut() {
        *slot = slot.with_translation((e.round(), f.round()).into());
    }
    [a, b, c, d, e.round(), f.round()]
}
