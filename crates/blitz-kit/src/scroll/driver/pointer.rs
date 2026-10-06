//! Pointer-driven scrolling: wheel detents and a touchpad's phased pan, latched from the
//! scrollers under the pointer.

use blitz_dom::BaseDocument;

use super::{Moves, ScrollDriver};
use crate::scroll::doc;
use crate::scroll::engine::{Engine, Kinetic, ScrollIn};
use crate::scroll::geom::{Dir, Px, ScrollAxis, ViewPoint};
use crate::scroll::latch::{self, Latched};
use crate::scroll::pad::{PadEvent, PointerScroll};
use crate::scroll::rubber::AtEdge;
use crate::scroll::tuning::Env;
use crate::scroll::velocity::SampleTime;

impl ScrollDriver {
    /// One pointer-scroll event at surface-local `at`. The engine takes a detent that finds a
    /// scroller and a touchpad gesture's start.
    pub fn pointer(
        &mut self,
        doc: &mut BaseDocument,
        input: PointerScroll,
        at: ViewPoint,
        env: Env<'_>,
    ) -> Moves {
        match input {
            PointerScroll::Detents { axis, steps } => self.detents(doc, axis, steps, at, env),
            pan @ (PointerScroll::Pan { .. } | PointerScroll::Stop) => {
                let (pad, event) = self.pad.feed(pan);
                self.pad = pad;
                match event {
                    Some(event) => self.pad_event(doc, event, at, env),
                    None => Moves::none(),
                }
            }
            PointerScroll::Nothing => Moves::none(),
        }
    }

    /// `steps` wheel detents on `axis` (signed, positive toward the maximum): each adds the
    /// detent distance to the target, which animates by the smooth rule (design/11 §11.3.11).
    /// Detents less than the burst window apart keep their first scroller.
    pub fn detents(
        &mut self,
        doc: &mut BaseDocument,
        axis: ScrollAxis,
        steps: f64,
        at: ViewPoint,
        env: Env<'_>,
    ) -> Moves {
        let px = steps * env.settings.wheel_detent_px.get();
        let Some(dir) = Dir::of(px) else {
            return Moves::none();
        };
        let window = env.settings.wheel_burst_window_ms.duration();
        let latch = self
            .burst
            .current(axis, env.now, window)
            .or_else(|| latch::latch_rigid(&doc::chain_at(doc, at, axis), axis, dir));
        let Some(latch) = latch else {
            return Moves::none();
        };
        self.burst = self.burst.record(latch, env.now);
        self.run(doc, ScrollIn::Step { latch, by: Px(px) }, env)
            .took()
    }

    fn pad_event(
        &mut self,
        doc: &mut BaseDocument,
        event: PadEvent,
        at: ViewPoint,
        env: Env<'_>,
    ) -> Moves {
        let sampled = SampleTime(env.now.0);
        match event {
            PadEvent::Began {
                axis,
                dir,
                delta,
                kinetic,
            } => {
                let began = self.begin(doc, axis, dir, kinetic, at, env).took();
                let changed = ScrollIn::Changed {
                    delta: Px(delta),
                    at: sampled,
                };
                began.then(self.run(doc, changed, env))
            }
            PadEvent::Changed { delta } => {
                let changed = ScrollIn::Changed {
                    delta: Px(delta),
                    at: sampled,
                };
                self.run(doc, changed, env)
            }
            PadEvent::Ended => self.run(doc, ScrollIn::Ended { at: sampled }, env),
        }
    }

    /// Latch a gesture on `axis` toward `dir` from the scrollers under `at`.
    pub fn begin(
        &mut self,
        doc: &mut BaseDocument,
        axis: ScrollAxis,
        dir: Dir,
        kinetic: Kinetic,
        at: ViewPoint,
        env: Env<'_>,
    ) -> Moves {
        let chain = doc::chain_at(doc, at, axis);
        let latched = match kinetic {
            Kinetic::Momentum => latch::latch(&chain, axis, dir),
            Kinetic::Plain => latch::latch_rigid(&chain, axis, dir).map(|latch| Latched {
                latch,
                began: AtEdge::Inside,
            }),
        };
        let Some(latched) = latched else {
            // Nothing can move: the gesture is consumed with no effect.
            self.engine = Engine::Idle;
            return Moves::none();
        };
        self.last_gesture = Some(latched.latch);
        let began = ScrollIn::Began {
            latched,
            dir,
            kinetic,
        };
        self.run(doc, began, env)
    }
}
