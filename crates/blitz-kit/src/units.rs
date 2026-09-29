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
