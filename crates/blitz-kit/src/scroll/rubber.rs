//! The rubber band (design/11 §11.3.7, §11.5.3): how far content stretches past an edge for a
//! given overscroll, and how it springs back.
//!
//! `O` is the overscroll the fingers asked for (px past the edge), `s` the stretch shown, `d`
//! the scroller's viewport extent on the axis. Stretches are magnitudes (>= 0), measured
//! outward from the edge.

use crate::scroll::config::{ScrollRubberBand, ScrollSettings};

/// A rebound ends once `|s|` is below this and ...
const END_STRETCH: f64 = 1.0;
/// ... at least this long has passed (design/11 §11.5.3, R11).
const END_AFTER_S: f64 = 0.024;

/// The stretch model and its constants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    model: ScrollRubberBand,
    c: f64,
    divisor: f64,
    /// Overscroll that does not stretch when the gesture began at the edge.
    edge_min: f64,
}

/// The spring-back after release.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapBack {
    rate: f64,
    gain: f64,
}

/// Whether a gesture began with its scroller already at the edge it pushes against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtEdge {
    AtEdge,
    Inside,
}

/// Whether a rebound has settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Settled {
    Settled,
    Moving,
}

impl Band {
    /// The model `settings` choose.
    pub fn from_settings(settings: &ScrollSettings) -> Band {
        Band {
            model: settings.rubber_band,
            c: settings.rubber_band_c.get(),
            divisor: settings.rubber_band_linear_divisor.get(),
            edge_min: settings.rubber_band_edge_min_px.get(),
        }
    }

    /// Whether any stretch is allowed at all (`scroll.rubber_band != Off`).
    pub fn enabled(&self) -> bool {
        self.model != ScrollRubberBand::Off
    }

    /// The stretch for overscroll `over` on a viewport of `d` px.
    pub fn stretch(&self, over: f64, d: f64, began: AtEdge) -> f64 {
        let over = match began {
            AtEdge::AtEdge => over - self.edge_min,
            AtEdge::Inside => over,
        };
        if over <= 0.0 {
            return 0.0;
        }
        match self.model {
            ScrollRubberBand::Bounded if d > 0.0 => (1.0 - 1.0 / (over * self.c / d + 1.0)) * d,
            ScrollRubberBand::Bounded | ScrollRubberBand::Off => 0.0,
            ScrollRubberBand::Linear => over / self.divisor,
        }
    }

    /// The overscroll that shows stretch `s` (the inverse of [`Band::stretch`]), for a gesture
    /// that starts on a stretched scroller.
    pub fn overscroll(&self, s: f64, d: f64, began: AtEdge) -> f64 {
        if s <= 0.0 {
            return 0.0;
        }
        let over = match self.model {
            ScrollRubberBand::Bounded if s < d => s * d / (self.c * (d - s)),
            ScrollRubberBand::Bounded => f64::MAX / 4.0,
            ScrollRubberBand::Linear => s * self.divisor,
            ScrollRubberBand::Off => 0.0,
        };
        match began {
            AtEdge::AtEdge => over + self.edge_min,
            AtEdge::Inside => over,
        }
    }
}

impl SnapBack {
    /// The constants `settings` carry.
    pub fn from_settings(settings: &ScrollSettings) -> SnapBack {
        SnapBack {
            rate: settings.rubber_band_snap_rate.get(),
            gain: settings.rubber_band_snap_gain.get(),
        }
    }

    /// The stretch `t` s into a rebound from stretch `s0` moving outward at `v0` px/s; never
    /// below 0 (the rebound stops at the edge, it does not cross it).
    pub fn stretch(&self, s0: f64, v0: f64, t: f64) -> f64 {
        ((s0 + self.gain * v0 * t) * (-self.rate * t).exp()).max(0.0)
    }

    /// Whether a rebound showing `s` at `t` s is over.
    pub fn settled(&self, s: f64, t: f64) -> Settled {
        if s.abs() < END_STRETCH && t >= END_AFTER_S {
            Settled::Settled
        } else {
            Settled::Moving
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn band(model: ScrollRubberBand) -> Band {
        Band::from_settings(&ScrollSettings {
            rubber_band: model,
            ..ScrollSettings::default()
        })
    }

    #[test]
    fn overscroll_inverts_stretch() {
        for model in [ScrollRubberBand::Bounded, ScrollRubberBand::Linear] {
            for began in [AtEdge::Inside, AtEdge::AtEdge] {
                let b = band(model);
                let s = b.stretch(420.0, 1000.0, began);
                let over = b.overscroll(s, 1000.0, began);
                assert!((over - 420.0).abs() < 1e-6, "{model:?} {began:?}: {over}");
            }
        }
    }

    #[test]
    fn snap_back_settles_at_368_ms_from_100_px() {
        let snap = SnapBack::from_settings(&ScrollSettings::default());
        // From 100 px at rest: below 1 px at ln(100)/12.5 = 368 ms.
        let t_small = 100f64.ln() / 12.5;
        assert!((t_small - 0.368).abs() < 0.001);
        assert_eq!(
            snap.settled(snap.stretch(100.0, 0.0, 0.369), 0.369),
            Settled::Settled
        );
        assert_eq!(
            snap.settled(snap.stretch(100.0, 0.0, 0.36), 0.36),
            Settled::Moving
        );
        // A fresh impact is not settled although it shows under 1 px.
        assert_eq!(snap.settled(0.0, 0.0), Settled::Moving);
    }
}
