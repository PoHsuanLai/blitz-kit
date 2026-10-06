//! The engine's settings, the physics derived from them, and the time a call runs at.

use crate::scroll::config::ScrollSettings;
use crate::scroll::engine::Physics;
use crate::scroll::time::Elapsed;

/// The settings and the physics derived from them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tuning {
    pub settings: ScrollSettings,
    pub physics: Physics,
}

impl Tuning {
    /// The tuning `settings` give.
    pub fn new(settings: ScrollSettings) -> Tuning {
        Tuning {
            physics: Physics::from_settings(&settings),
            settings,
        }
    }

    /// This tuning at `now`.
    pub fn at(&self, now: Elapsed) -> Env<'_> {
        Env {
            settings: &self.settings,
            physics: &self.physics,
            now,
        }
    }
}

impl Default for Tuning {
    fn default() -> Tuning {
        Tuning::new(ScrollSettings::default())
    }
}

/// The time and the tuning a call works with.
#[derive(Debug, Clone, Copy)]
pub struct Env<'a> {
    pub settings: &'a ScrollSettings,
    pub physics: &'a Physics,
    pub now: Elapsed,
}
