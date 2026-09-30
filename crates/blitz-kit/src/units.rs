//! The units the kit takes and returns: CSS px `f64` in Blitz's own coordinates, and the
//! fractional scale as integer 120ths. A consumer converts at its own boundary.

/// A laid-out box in page-space logical px, fractional as Blitz reports it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// A point in the same fractional page-space CSS px as [`Bounds`] (what Blitz hit-tests: the
/// client position plus the viewport's scroll).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PagePoint {
    pub x: f64,
    pub y: f64,
}

/// A fractional scale in 120ths (`180` is 1.5), the unit Wayland's `wp_fractional_scale` uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Scale120(pub u32);

impl Scale120 {
    /// The nearest 120th to a device-pixels-per-logical-pixel `factor` (`1.5` is `180`).
    pub fn from_factor(factor: f64) -> Scale120 {
        Scale120((factor * 120.0).round().max(0.0) as u32)
    }

    /// Device pixels per logical pixel.
    pub fn factor(self) -> f64 {
        f64::from(self.0) / 120.0
    }

    /// Whether the scale is a whole number, where Blitz's own logical rounding is already the
    /// device grid.
    pub fn is_whole(self) -> bool {
        self.0.is_multiple_of(120)
    }
}

#[cfg(test)]
mod tests {
    use super::Scale120;

    #[test]
    fn a_factor_rounds_to_the_nearest_120th() {
        for (factor, want) in [(1.0, 120), (1.25, 150), (1.5, 180), (1.75, 210), (0.0, 0)] {
            assert_eq!(Scale120::from_factor(factor), Scale120(want), "{factor}");
        }
    }
}
