//! The engine applied to one Blitz document: a scroller is found under the pointer, the engine
//! steps with the document's own geometry, and its offsets are written raw. The pure physics is
//! `scroll::engine`; what a host adds is turning its device events into [`PointerScroll`],
//! [`ScrollKey`](crate::scroll::keys::ScrollKey) and [`ScrollCmd`](crate::scroll::cmd::ScrollCmd)
//! and calling for a frame while [`ScrollDriver::motion`] says `Animating`.

mod command;
mod keys;
mod moves;
mod pointer;

use blitz_dom::BaseDocument;

use crate::scroll::doc;
use crate::scroll::engine::{self, Engine, Motion, ScrollIn, ScrollOut};
use crate::scroll::geom::Latch;
use crate::scroll::latch::WheelBurst;
use crate::scroll::pad::Pad;
use crate::scroll::tuning::Env;

pub use moves::{Claim, KeyRepeat, KeyUse, Moved, Moves};

/// One document's scroll state: the engine, the touchpad gesture and wheel burst being
/// tracked, and the scroller the last gesture latched (the keyboard's target while under the
/// pointer).
#[derive(Debug, Clone)]
pub struct ScrollDriver {
    engine: Engine,
    pad: Pad,
    burst: WheelBurst,
    last_gesture: Option<Latch>,
}

impl Default for ScrollDriver {
    fn default() -> ScrollDriver {
        ScrollDriver {
            engine: Engine::Idle,
            pad: Pad::Idle,
            burst: WheelBurst::default(),
            last_gesture: None,
        }
    }
}

impl ScrollDriver {
    /// The engine, for a host that reads its phase.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// The scroller the engine is on.
    pub fn latch(&self) -> Option<Latch> {
        engine::latch_of(&self.engine)
    }

    /// Whether the engine needs a frame per callback.
    pub fn motion(&self) -> Motion {
        engine::motion(&self.engine)
    }

    /// Step the engine with `input` and write what it says.
    pub fn run(&mut self, doc: &mut BaseDocument, input: ScrollIn, env: Env<'_>) -> Moves {
        let target = engine::target(&self.engine, &input);
        let geom = target.and_then(|latch| doc::geom(doc, latch.scroller, latch.axis));
        let current = std::mem::take(&mut self.engine);
        let (next, outs) = engine::step(current, input, env.now, geom, env.physics);
        self.engine = next;
        let written = outs
            .into_iter()
            .filter_map(|out| match out {
                ScrollOut::SetOffset { latch, raw } => {
                    doc::write(doc, latch.scroller, latch.axis, raw.0);
                    Some(latch)
                }
                ScrollOut::Notify { .. } => None,
            })
            .collect();
        Moves::wrote(written)
    }

    /// A frame is rendered at `env.now`: advance the animation. Called before the resolve, so
    /// the layout (and Blitz's hoisting of positioned descendants) sees this frame's offsets.
    pub fn frame(&mut self, doc: &mut BaseDocument, env: Env<'_>) -> Moves {
        self.run(doc, ScrollIn::Frame, env)
    }
}
