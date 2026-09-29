//! Snapping each box's final layout from its unrounded one.

use blitz_dom::{BaseDocument, NodeId};

use super::grid::{DeviceGrid, Origin};

/// Snap `id`'s final layout from its unrounded one, then its layout children's.
pub(super) fn place(doc: &mut BaseDocument, id: NodeId, parent: Origin, grid: DeviceGrid) {
    let Some(node) = doc.get_node(id) else {
        return;
    };
    let exact = *node.unrounded_layout();
    let children = layout_children(doc, id);
    let at = Origin {
        x: parent.x + f64::from(exact.location.x),
        y: parent.y + f64::from(exact.location.y),
    };
    let (right, bottom) = (
        at.x + f64::from(exact.size.width),
        at.y + f64::from(exact.size.height),
    );
    let mut snapped = exact;
    snapped.location.x = (grid.snap(at.x) - grid.snap(parent.x)) as f32;
    snapped.location.y = (grid.snap(at.y) - grid.snap(parent.y)) as f32;
    snapped.size.width = grid.span(at.x, exact.size.width);
    snapped.size.height = grid.span(at.y, exact.size.height);
    snapped.border.left = grid.line(exact.border.left);
    snapped.border.right = grid.line(exact.border.right);
    snapped.border.top = grid.line(exact.border.top);
    snapped.border.bottom = grid.line(exact.border.bottom);
    snapped.padding.left = grid.span(at.x, exact.padding.left);
    snapped.padding.right = grid.span(right - f64::from(exact.padding.right), exact.padding.right);
    snapped.padding.top = grid.span(at.y, exact.padding.top);
    snapped.padding.bottom = grid.span(
        bottom - f64::from(exact.padding.bottom),
        exact.padding.bottom,
    );
    snapped.scrollbar_size.width = grid.line(exact.scrollbar_size.width);
    snapped.scrollbar_size.height = grid.line(exact.scrollbar_size.height);
    let overflow = exact.scrollable_overflow_rect;
    snapped.scrollable_overflow_rect.left = grid.span(at.x, overflow.left);
    snapped.scrollable_overflow_rect.right = grid.span(at.x, overflow.right);
    snapped.scrollable_overflow_rect.top = grid.span(at.y, overflow.top);
    snapped.scrollable_overflow_rect.bottom = grid.span(at.y, overflow.bottom);
    if let Some(node) = doc.get_node_mut(id) {
        *node.final_layout_mut() = snapped;
    }
    for child in children {
        place(doc, child, at, grid);
    }
}

/// `id`'s layout children: the boxes taffy laid out under it, in-flow and out-of-flow (Blitz
/// makes every box the containing block of its own out-of-flow children).
pub(super) fn layout_children(doc: &BaseDocument, id: NodeId) -> Vec<NodeId> {
    doc.get_node(id)
        .and_then(|node| node.layout_children.borrow().as_ref().map(|c| c.to_vec()))
        .unwrap_or_default()
}
