//! A scroller's geometry and the raw offset write.

use blitz_dom::{BaseDocument, Node, NodeId};

use super::element::{attr, body, scrolls};
use crate::scroll::geom::{Elastic, Geom, Px, ScrollAxis, Scroller};

/// What a node's `data-overscroll` asks for at the end of its range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Overscroll {
    Band,
    Clamp,
}

impl Overscroll {
    /// This request, or `other` when this one asks for nothing.
    fn or(self, other: Overscroll) -> Overscroll {
        match self {
            Overscroll::Band => Overscroll::Band,
            Overscroll::Clamp => other,
        }
    }
}

/// What `node` opted into (`data-overscroll="band"` is the rubber band).
fn overscroll(node: &Node) -> Overscroll {
    match attr(node, "data-overscroll") {
        Some("band") => Overscroll::Band,
        _ => Overscroll::Clamp,
    }
}

/// An elastic axis (design/11 §11.3.7): opted in, and vertical or with content to scroll.
fn elastic(opted: Overscroll, axis: ScrollAxis, max: f64) -> Elastic {
    match (opted, axis) {
        (Overscroll::Band, ScrollAxis::Y) => Elastic::Elastic,
        (Overscroll::Band, ScrollAxis::X) if max > 0.0 => Elastic::Elastic,
        _ => Elastic::Rigid,
    }
}

/// `scroller`'s geometry on `axis`; `None` for a node that is gone.
pub fn geom(doc: &BaseDocument, scroller: Scroller, axis: ScrollAxis) -> Option<Geom> {
    let pick = |x: f64, y: f64| match axis {
        ScrollAxis::X => x,
        ScrollAxis::Y => y,
    };
    match scroller {
        Scroller::Viewport => {
            let root = doc.try_root_element()?;
            let layout = root.final_layout();
            let (vw, vh) = doc.viewport().logical_size();
            let content_w = f64::from(layout.size.width.max(layout.scrollable_overflow_rect.right));
            let content_h = f64::from(
                layout
                    .size
                    .height
                    .max(layout.scrollable_overflow_rect.bottom),
            );
            let max = pick(content_w - f64::from(vw), content_h - f64::from(vh)).max(0.0);
            let scroll = doc.viewport_scroll();
            let opted = body(doc).map_or(Overscroll::Clamp, overscroll);
            let opted = overscroll(root).or(opted);
            Some(Geom {
                offset: Px(pick(scroll.x, scroll.y)),
                max: Px(max),
                viewport: Px(pick(f64::from(vw), f64::from(vh))),
                elastic: elastic(opted, axis, max),
            })
        }
        Scroller::Node(id) => {
            let node = doc.get_node(NodeId::from_u64(id))?;
            let layout = node.final_layout();
            let max = match scrolls(doc, node, axis) {
                true => pick(
                    f64::from(layout.scroll_width()),
                    f64::from(layout.scroll_height()),
                ),
                false => 0.0,
            }
            .max(0.0);
            let offset = node.scroll_offset();
            Some(Geom {
                offset: Px(pick(offset.x, offset.y)),
                max: Px(max),
                viewport: Px(pick(
                    f64::from(layout.size.width),
                    f64::from(layout.size.height),
                )),
                elastic: elastic(overscroll(node), axis, max),
            })
        }
    }
}

/// Write `raw` as `scroller`'s offset on `axis`, leaving the other axis alone.
pub fn write(doc: &mut BaseDocument, scroller: Scroller, axis: ScrollAxis, raw: f64) {
    let set = |point: &mut blitz_dom::Point<f64>| match axis {
        ScrollAxis::X => point.x = raw,
        ScrollAxis::Y => point.y = raw,
    };
    match scroller {
        Scroller::Viewport => {
            let mut scroll = doc.viewport_scroll();
            set(&mut scroll);
            doc.set_viewport_scroll(scroll);
        }
        Scroller::Node(id) => {
            if let Some(node) = doc.get_node_mut(NodeId::from_u64(id)) {
                set(node.scroll_offset_mut());
            }
        }
    }
}
