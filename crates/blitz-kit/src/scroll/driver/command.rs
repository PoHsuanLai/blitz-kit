//! Programmatic scrolls (design/11 §11.3.11).

use blitz_dom::BaseDocument;

use super::{Moves, ScrollDriver};
use crate::element_id::ElementId;
use crate::scroll::cmd::{self, ScrollCmd};
use crate::scroll::doc;
use crate::scroll::engine::{ScrollAnimate, ScrollIn};
use crate::scroll::geom::{Latch, Px, ScrollAxis};
use crate::scroll::tuning::Env;

impl ScrollDriver {
    /// Run a programmatic scroll. A command is always the engine's own, even when its element
    /// is gone and nothing moves.
    pub fn command(&mut self, doc: &mut BaseDocument, command: &ScrollCmd, env: Env<'_>) -> Moves {
        let input = match command {
            ScrollCmd::To {
                element,
                axis,
                offset,
                animate,
            } => doc::scroller_by_id(doc, element).map(|scroller| ScrollIn::Jump {
                latch: Latch {
                    scroller,
                    axis: *axis,
                },
                to: Px(*offset),
                animate: *animate,
            }),
            ScrollCmd::By {
                element,
                axis,
                delta,
                animate,
            } => doc::scroller_by_id(doc, element).and_then(|scroller| {
                let latch = Latch {
                    scroller,
                    axis: *axis,
                };
                let geom = doc::geom(doc, scroller, *axis)?;
                Some(match animate {
                    ScrollAnimate::Smooth => ScrollIn::Step {
                        latch,
                        by: Px(*delta),
                    },
                    ScrollAnimate::Instant => ScrollIn::Jump {
                        latch,
                        to: Px(geom.offset.0 + delta),
                        animate: ScrollAnimate::Instant,
                    },
                })
            }),
            ScrollCmd::IntoView { element, animate } => into_view(doc, element, *animate),
        };
        match input {
            Some(input) => self.run(doc, input, env).took(),
            None => Moves::none().took(),
        }
    }
}

/// The jump that brings the element with `id` into its vertical scroller's view.
fn into_view(doc: &BaseDocument, id: &ElementId, animate: ScrollAnimate) -> Option<ScrollIn> {
    let (scroller, child) = doc::container_of(doc, id, ScrollAxis::Y)?;
    let view = doc::area(doc, scroller)?;
    let geom = doc::geom(doc, scroller, ScrollAxis::Y)?;
    let to = cmd::into_view(
        (view.y, view.y + view.height),
        (child.y, child.y + child.height),
        geom.offset.0,
        geom.max.0,
        cmd::INTO_VIEW_MARGIN,
    )?;
    Some(ScrollIn::Jump {
        latch: Latch {
            scroller,
            axis: ScrollAxis::Y,
        },
        to: Px(to),
        animate,
    })
}
