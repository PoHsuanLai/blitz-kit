//! A scroller's geometry and the raw offset write.

use blitz_dom::{BaseDocument, Node, NodeId};

use super::element::{attr, body, scrolls};
use super::frame::{in_frame, in_frame_mut};
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

/// Which kind of scroller an axis belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Viewport,
    Node,
}

/// An elastic axis (design/11 §11.3.7): opted in, and with content to scroll, or the document
/// viewport's vertical. A container whose content fits never stretches: its content would move
/// with nothing to scroll to.
fn elastic(opted: Overscroll, kind: Kind, axis: ScrollAxis, max: f64) -> Elastic {
    match (opted, kind, axis) {
        (Overscroll::Band, Kind::Viewport, ScrollAxis::Y) => Elastic::Elastic,
        (Overscroll::Band, _, _) if max > 0.0 => Elastic::Elastic,
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
        Scroller::Framed(framed) => in_frame(doc, framed.host, |sub| {
            geom(sub, framed.inside.scroller(), axis)
        })?,
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
                elastic: elastic(opted, Kind::Viewport, axis, max),
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
                elastic: elastic(overscroll(node), Kind::Node, axis, max),
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
        Scroller::Framed(framed) => {
            in_frame_mut(doc, framed.host, |sub| {
                write(sub, framed.inside.scroller(), axis, raw)
            });
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_container_stretches_only_with_content_to_scroll_and_the_viewport_vertically_always() {
        use ScrollAxis::{X, Y};
        let band = Overscroll::Band;
        assert_eq!(
            elastic(band, Kind::Node, Y, 0.0),
            Elastic::Rigid,
            "content fits"
        );
        assert_eq!(elastic(band, Kind::Node, Y, 40.0), Elastic::Elastic);
        assert_eq!(elastic(band, Kind::Node, X, 40.0), Elastic::Elastic);
        assert_eq!(elastic(band, Kind::Viewport, Y, 0.0), Elastic::Elastic);
        assert_eq!(elastic(band, Kind::Viewport, X, 0.0), Elastic::Rigid);
        assert_eq!(
            elastic(Overscroll::Clamp, Kind::Node, Y, 40.0),
            Elastic::Rigid
        );
    }
}
