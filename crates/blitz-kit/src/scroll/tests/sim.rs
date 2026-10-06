//! A scroller, the engine and a hand-advanced clock: synthetic streams at 120 Hz through
//! `scroll::engine::step`, with the scroller's geometry updated from the engine's own
//! `SetOffset`s, as a host does.

use std::time::Duration;

use crate::scroll::config::ScrollSettings;
use crate::scroll::engine::{
    self, Engine, Flight, Kinetic, Motion, NotifyPhase, Physics, ScrollIn, ScrollOut,
};
use crate::scroll::geom::{Dir, Elastic, Geom, Latch, Px, ScrollAxis, Scroller};
use crate::scroll::latch::Latched;
use crate::scroll::rubber::AtEdge;
use crate::scroll::time::Elapsed;
use crate::scroll::velocity::SampleTime;

pub(super) const FRAME: f64 = 1.0 / 120.0;

pub(super) const PAGE: Latch = Latch {
    scroller: Scroller::Viewport,
    axis: ScrollAxis::Y,
};

/// One scroller, the engine, and a clock.
pub(super) struct Sim {
    pub(super) engine: Engine,
    pub(super) geom: Geom,
    /// Seconds since the start.
    pub(super) t: f64,
    pub(super) physics: Physics,
    pub(super) notes: Vec<(f64, NotifyPhase)>,
}

impl Sim {
    pub(super) fn new(offset: f64, max: f64, elastic: Elastic) -> Sim {
        Sim::with(offset, max, elastic, ScrollSettings::default())
    }

    pub(super) fn with(offset: f64, max: f64, elastic: Elastic, scroll: ScrollSettings) -> Sim {
        Sim {
            engine: Engine::Idle,
            geom: Geom {
                offset: Px(offset),
                max: Px(max),
                viewport: Px(1000.0),
                elastic,
            },
            t: 0.0,
            physics: Physics::from_settings(&scroll),
            notes: Vec::new(),
        }
    }

    pub(super) fn now(&self) -> Elapsed {
        Elapsed(Duration::from_secs_f64(self.t))
    }

    pub(super) fn at(&self) -> SampleTime {
        SampleTime(self.now().0)
    }

    pub(super) fn offset(&self) -> f64 {
        self.geom.offset.0
    }

    pub(super) fn feed(&mut self, input: ScrollIn) {
        let engine = std::mem::take(&mut self.engine);
        let (next, outs) = engine::step(engine, input, self.now(), Some(self.geom), &self.physics);
        self.engine = next;
        for out in outs {
            match out {
                ScrollOut::SetOffset { raw, .. } => self.geom.offset = raw,
                ScrollOut::Notify { phase, .. } => self.notes.push((self.t, phase)),
            }
        }
    }

    pub(super) fn began(&mut self, dir: Dir, began: AtEdge) {
        self.feed(ScrollIn::Began {
            latched: Latched { latch: PAGE, began },
            dir,
            kinetic: Kinetic::Momentum,
        });
    }

    /// A steady stroke of `speed` px/s (offset space) for `ms`, one Changed per frame.
    pub(super) fn stroke(&mut self, speed: f64, ms: u32) {
        let frames = (f64::from(ms) / 1000.0 / FRAME).round() as u32;
        for _ in 0..frames {
            self.t += FRAME;
            self.feed(ScrollIn::Changed {
                delta: Px(speed * FRAME),
                at: self.at(),
            });
        }
    }

    pub(super) fn pull(&mut self, px: f64) {
        self.t += FRAME;
        self.feed(ScrollIn::Changed {
            delta: Px(px),
            at: self.at(),
        });
    }

    pub(super) fn ended(&mut self) {
        self.feed(ScrollIn::Ended { at: self.at() });
    }

    /// Frames until the engine stops or `max_s` passes; the offset after each.
    pub(super) fn run(&mut self, max_s: f64) -> Vec<(f64, f64)> {
        let start = self.t;
        let mut trace = Vec::new();
        while engine::motion(&self.engine) == Motion::Animating && self.t - start < max_s {
            self.t += FRAME;
            self.feed(ScrollIn::Frame);
            trace.push((self.t - start, self.offset()));
        }
        trace
    }

    pub(super) fn when(&self, phase: NotifyPhase) -> Option<f64> {
        self.notes
            .iter()
            .rev()
            .find(|(_, p)| *p == phase)
            .map(|(t, _)| *t)
    }

    pub(super) fn flight(&self) -> Flight {
        match self.engine {
            Engine::Momentum(f) => f,
            ref other => panic!("not gliding: {other:?}"),
        }
    }
}
