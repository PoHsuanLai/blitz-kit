//! What Blitz would hit at a point, lifted to the element that carries the listeners.
//!
//! Blitz's own `hit` carries the point back through each transformed ancestor (the inverse of
//! the node's transform at every level of the descent), so the answer already agrees with
//! where a transformed box paints.

use blitz_dom::{BaseDocument, ElementData, NodeId};

use crate::units::PagePoint;

/// `id` if it is an element, else its nearest element ancestor: the node whose listeners a
/// hover change reaches.
pub fn element_of(doc: &BaseDocument, id: NodeId) -> Option<NodeId> {
    let node = doc.get_node(id)?;
    if node.is_element() {
        Some(id)
    } else {
        element_of(doc, node.parent?)
    }
}

/// The element Blitz would hover at `at`: the hit, lifted out of anonymous boxes as
/// `set_hover_to` lifts it, then to its element.
pub fn element_at(doc: &BaseDocument, at: PagePoint) -> Option<NodeId> {
    let hit = doc.hit(at.x as f32, at.y as f32)?;
    element_of(doc, doc.nearest_non_anonymous_ancestor(hit.node_id)?)
}

/// Whether `data` is a document's own content: not a tag that is `display:none` by Blitz's
/// default stylesheet (`style`, `script`, `title`, `link`, `meta`, `base`, `template`).
pub fn is_content_element(data: &ElementData) -> bool {
    !matches!(
        &*data.name.local,
        "style" | "script" | "title" | "link" | "meta" | "base" | "template"
    )
}
