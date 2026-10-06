//! `ScrollSettings`: every tunable number of the scroll engine, one field per settings key of
//! design/22 §3.7 `scroll`, with the documented defaults. A host reads its own settings file and
//! hands the engine a `ScrollSettings`; the kit never reads a file itself.
//!
//! Field names are the keys without their section, and with feature `settings` the serde form is
//! the keys' own (a `[scroll]` table deserializes straight into it). Settled values design/22 has
//! no key for (the keyboard line, the spring, the rebound end rule) are constants next to the code
//! that uses them (`scroll::keys`, `scroll::rubber`).

use std::time::Duration;

#[cfg(feature = "settings")]
use serde::{Deserialize, Serialize};

/// A length in logical pixels, as a settings key spells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "settings", derive(Serialize, Deserialize))]
pub struct ScrollPx(pub u16);

/// A duration in whole milliseconds, as a settings key spells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "settings", derive(Serialize, Deserialize))]
pub struct ScrollMs(pub u16);

/// A ratio in thousandths (`550` is 0.55), as a settings key spells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "settings", derive(Serialize, Deserialize))]
pub struct ScrollFraction(pub u16);

/// A dimensionless physics constant (a drag exponent, a speed in px/s).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "settings", derive(Serialize, Deserialize))]
pub struct ScrollScalar(pub f32);

impl ScrollPx {
    /// As an `f64` for the formulas.
    pub fn get(self) -> f64 {
        f64::from(self.0)
    }
}

impl ScrollMs {
    /// As a `Duration`.
    pub fn duration(self) -> Duration {
        Duration::from_millis(u64::from(self.0))
    }

    /// In seconds, for the formulas.
    pub fn secs(self) -> f64 {
        f64::from(self.0) / 1000.0
    }
}

impl ScrollFraction {
    /// As a ratio.
    pub fn get(self) -> f64 {
        f64::from(self.0) / 1000.0
    }
}

impl ScrollScalar {
    /// As an `f64` for the formulas.
    pub fn get(self) -> f64 {
        f64::from(self.0)
    }
}

/// `scroll.natural`: whether content follows the fingers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "settings", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "settings", serde(rename_all = "snake_case"))]
pub enum NaturalScroll {
    #[default]
    Natural,
    Traditional,
}

/// `scroll.momentum`: whether a flick glides on after the fingers lift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "settings", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "settings", serde(rename_all = "snake_case"))]
pub enum ScrollMomentum {
    #[default]
    On,
    Off,
}

/// `scroll.momentum_model`: the deceleration curve of a glide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "settings", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "settings", serde(rename_all = "snake_case"))]
pub enum ScrollMomentumModel {
    /// `v' = -a v^b` (design/11 §11.5.1).
    #[default]
    MacMouseFix,
    /// `v(t) = v0 r^t_ms`, `r = 0.998`: the longer glide (design/11 R5).
    Ios,
}

/// `scroll.rubber_band`: how far content stretches past an edge under the fingers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "settings", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "settings", serde(rename_all = "snake_case"))]
pub enum ScrollRubberBand {
    /// `s = (1 - 1/(O c/d + 1)) d`.
    #[default]
    Bounded,
    /// `s = O / divisor`.
    Linear,
    /// No stretch: every edge clamps.
    Off,
}

/// `scroll.scrollbars`: when the overlay thumb shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "settings", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "settings", serde(rename_all = "snake_case"))]
pub enum ScrollbarVisibility {
    #[default]
    WhenScrolling,
    /// Never fades, on the legacy 15 px track.
    Always,
}

/// The `scroll` section (design/22 §3.7), plus `scroll.scrollbars` (listed under §3.8).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "settings", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "settings", serde(default))]
pub struct ScrollSettings {
    pub natural: NaturalScroll,
    pub momentum: ScrollMomentum,
    pub momentum_model: ScrollMomentumModel,
    pub momentum_a: ScrollScalar,
    pub momentum_b: ScrollScalar,
    pub momentum_stop_px_s: ScrollScalar,
    pub momentum_start_px_s: ScrollScalar,
    pub momentum_cap_px_s: ScrollScalar,
    pub velocity_window_ms: ScrollMs,
    pub velocity_held_still_ms: ScrollMs,
    pub rubber_band: ScrollRubberBand,
    pub rubber_band_c: ScrollFraction,
    pub rubber_band_linear_divisor: ScrollScalar,
    pub rubber_band_edge_min_px: ScrollPx,
    pub rubber_band_snap_rate: ScrollScalar,
    pub rubber_band_snap_gain: ScrollScalar,
    pub wheel_detent_px: ScrollPx,
    pub wheel_burst_window_ms: ScrollMs,
    pub momentum_boost_threshold_px_s: ScrollScalar,
    pub double_scroll_grace_ms: ScrollMs,
    pub scrollbars: ScrollbarVisibility,
}

impl Default for ScrollSettings {
    fn default() -> ScrollSettings {
        ScrollSettings {
            natural: NaturalScroll::Natural,
            momentum: ScrollMomentum::On,
            momentum_model: ScrollMomentumModel::MacMouseFix,
            momentum_a: ScrollScalar(30.0),
            momentum_b: ScrollScalar(0.7),
            momentum_stop_px_s: ScrollScalar(1.0),
            momentum_start_px_s: ScrollScalar(100.0),
            momentum_cap_px_s: ScrollScalar(12_000.0),
            velocity_window_ms: ScrollMs(80),
            velocity_held_still_ms: ScrollMs(50),
            rubber_band: ScrollRubberBand::Bounded,
            rubber_band_c: ScrollFraction(550),
            rubber_band_linear_divisor: ScrollScalar(20.0),
            rubber_band_edge_min_px: ScrollPx(10),
            rubber_band_snap_rate: ScrollScalar(12.5),
            rubber_band_snap_gain: ScrollScalar(0.31),
            wheel_detent_px: ScrollPx(60),
            wheel_burst_window_ms: ScrollMs(300),
            momentum_boost_threshold_px_s: ScrollScalar(350.0),
            double_scroll_grace_ms: ScrollMs(150),
            scrollbars: ScrollbarVisibility::WhenScrolling,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_are_design_22s() {
        let s = ScrollSettings::default();
        let cases: &[(&str, f64, f64)] = &[
            ("momentum_a", s.momentum_a.get(), 30.0),
            ("momentum_b", s.momentum_b.get(), 0.7),
            ("stop", s.momentum_stop_px_s.get(), 1.0),
            ("start", s.momentum_start_px_s.get(), 100.0),
            ("cap", s.momentum_cap_px_s.get(), 12_000.0),
            ("window", s.velocity_window_ms.secs(), 0.08),
            ("held still", s.velocity_held_still_ms.secs(), 0.05),
            ("c", s.rubber_band_c.get(), 0.55),
            ("divisor", s.rubber_band_linear_divisor.get(), 20.0),
            ("edge min", s.rubber_band_edge_min_px.get(), 10.0),
            ("snap rate", s.rubber_band_snap_rate.get(), 12.5),
            ("snap gain", s.rubber_band_snap_gain.get(), 0.31),
            ("detent", s.wheel_detent_px.get(), 60.0),
            ("burst", s.wheel_burst_window_ms.secs(), 0.3),
            ("boost", s.momentum_boost_threshold_px_s.get(), 350.0),
            ("grace", s.double_scroll_grace_ms.secs(), 0.15),
        ];
        for (name, got, want) in cases {
            assert!((got - want).abs() < 1e-6, "{name}: {got} != {want}");
        }
        assert_eq!(s.natural, NaturalScroll::Natural);
        assert_eq!(s.rubber_band, ScrollRubberBand::Bounded);
        assert_eq!(s.scrollbars, ScrollbarVisibility::WhenScrolling);
    }
}
