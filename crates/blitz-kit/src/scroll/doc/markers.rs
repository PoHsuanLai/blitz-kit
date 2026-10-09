//! The markers an element sets to take input itself: `data-wheel="capture"` (design/11 §11.3.1)
//! and the key handlers on the focus path (§11.3.10).

use blitz_dom::{BaseDocument, Node};

use super::chain::{chain_at, page_point};
use super::element::{ancestors, attr, body, element_under, is_root, scrolls};
use super::geometry::geom;
use crate::scroll::geom::{Elastic, ScrollAxis, Scroller, ViewPoint};

/// Whether a wheel over a point goes to the engine or, raw, to a capturing element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelRoute {
    Engine,
    /// An element on the chain carries `data-wheel="capture"`.
    Capture,
    /// Nothing under the pointer can scroll, on either axis: the wheel is the document's.
    Nothing,
}

/// Whether any scroller under `at` has room to move, or stretch, on either axis.
fn scrollable(doc: &BaseDocument, at: ViewPoint) -> bool {
    [ScrollAxis::X, ScrollAxis::Y].into_iter().any(|axis| {
        chain_at(doc, at, axis)
            .iter()
            .any(|c| c.geom.max.0 > 0.0 || c.geom.elastic == Elastic::Elastic)
    })
}

/// What the keyboard focus means for scroll keys (design/11 §11.3.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyFocus {
    /// Nothing focused (or the root or body, where Blitz parks focus): keys scroll the default
    /// target (the scroller under the pointer).
    Free,
    /// The focus sits in this scroller and nothing on its path handles keys: keys scroll it.
    Scroller(Scroller),
    /// Keys are the document's: an element on the focus path has a Dioxus key listener or
    /// `data-keys="capture"`, the focus is an editable or a form control, or nothing around the
    /// focus scrolls on the key's axis.
    Document,
}

/// Where a wheel at surface-local `at` goes.
pub fn wheel_route(doc: &BaseDocument, at: ViewPoint) -> WheelRoute {
    let (px, py) = page_point(doc, at);
    let captured = element_under(doc, px, py)
        .into_iter()
        .flat_map(|id| ancestors(doc, id))
        .any(|node| attr(node, "data-wheel") == Some("capture"));
    match (captured, scrollable(doc, at)) {
        (true, _) => WheelRoute::Capture,
        (false, true) => WheelRoute::Engine,
        (false, false) => WheelRoute::Nothing,
    }
}

/// The Dioxus key listeners: dioxus-native-dom records a listener as an attribute named for
/// the event (`keydown`) whose value is this placeholder (`MutationWriter::
/// create_event_listener`); Blitz cannot report a handler's `prevent_default`, so a listener on
/// the focus path is taken as handling keys.
const KEY_LISTENERS: [&str; 3] = ["keydown", "keyup", "keypress"];
const LISTENER_PLACEHOLDER: &str = "<rust func>";

/// Whether `node` carries a Dioxus key listener.
fn listens_for_keys(node: &Node) -> bool {
    KEY_LISTENERS
        .iter()
        .any(|name| attr(node, name) == Some(LISTENER_PLACEHOLDER))
}

/// What the focused element means for a scroll key on `axis`.
pub fn key_focus(doc: &BaseDocument, axis: ScrollAxis) -> KeyFocus {
    let Some(focus) = doc.get_focussed_node_id() else {
        return KeyFocus::Free;
    };
    // Blitz focuses the root element when nothing else is: that is "nothing focused" (the
    // body counts the same), unless a key listener sits there.
    let unfocused = is_root(doc, focus) || body(doc).is_some_and(|b| b.id == focus);
    if unfocused {
        return match ancestors(doc, focus).any(listens_for_keys) {
            true => KeyFocus::Document,
            false => KeyFocus::Free,
        };
    }
    let handled = ancestors(doc, focus).enumerate().any(|(depth, node)| {
        listens_for_keys(node)
            || attr(node, "data-keys") == Some("capture")
            || (depth == 0 && owns_keys(node))
    });
    if handled {
        return KeyFocus::Document;
    }
    let scroller = ancestors(doc, focus)
        .find(|n| n.is_element() && !is_root(doc, n.id) && scrolls(doc, n, axis))
        .map(|n| Scroller::Node(n.id.as_u64()))
        .or_else(|| {
            geom(doc, Scroller::Viewport, axis)
                .filter(|g| g.max.0 > 0.0)
                .map(|_| Scroller::Viewport)
        });
    scroller.map_or(KeyFocus::Document, KeyFocus::Scroller)
}

/// Whether a focused `node` takes arrow and space keys itself: text inputs, form controls,
/// buttons (Space presses them) and `contenteditable`.
fn owns_keys(node: &Node) -> bool {
    let Some(element) = node.element_data() else {
        return false;
    };
    element.text_input_data().is_some()
        || matches!(
            &*element.name.local,
            "input" | "textarea" | "select" | "button"
        )
        || attr(node, "contenteditable").is_some_and(|v| v != "false")
}
