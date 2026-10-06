//! Reading elements: attributes, ancestors, the element under a point, and what scrolls.

use blitz_dom::{BaseDocument, Node, NodeId};

use crate::hit::element_at;
use crate::scroll::geom::ScrollAxis;
use crate::units::PagePoint;

/// The value of attribute `name` on `node`, if it is an element carrying it.
pub(super) fn attr<'a>(node: &'a Node, name: &str) -> Option<&'a str> {
    node.element_data()?
        .attrs
        .iter()
        .find(|a| *a.name.local == *name)
        .map(|a| a.value.as_str())
}

/// `node` and its ancestors, innermost first.
pub(super) fn ancestors(doc: &BaseDocument, id: NodeId) -> impl Iterator<Item = &Node> {
    std::iter::successors(doc.get_node(id), |node| {
        node.parent.and_then(|p| doc.get_node(p))
    })
}

/// The element under document point `(x, y)` (page coordinates: surface-local plus the
/// viewport's scroll), lifted out of text and anonymous boxes.
pub(super) fn element_under(doc: &BaseDocument, x: f64, y: f64) -> Option<NodeId> {
    element_at(doc, PagePoint { x, y })
}

/// Whether `node`'s computed `overflow-x`/`-y` lets a user scroll it on `axis`.
pub(super) fn scrolls(doc: &BaseDocument, node: &Node, axis: ScrollAxis) -> bool {
    let property = match axis {
        ScrollAxis::X => "overflow-x",
        ScrollAxis::Y => "overflow-y",
    };
    matches!(
        doc.resolved_style_value(node.id, property).as_str(),
        "auto" | "scroll"
    )
}

pub(super) fn is_root(doc: &BaseDocument, id: NodeId) -> bool {
    doc.try_root_element().is_some_and(|root| root.id == id)
}

/// The document's `body`, if it has one.
pub(super) fn body(doc: &BaseDocument) -> Option<&Node> {
    let root = doc.try_root_element()?;
    root.children
        .iter()
        .filter_map(|&id| doc.get_node(id))
        .find(|n| n.element_data().is_some_and(|e| &*e.name.local == "body"))
}
