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

/// The smallest preferred scale [`Scale120::from_wire`] accepts, in 120ths: a quarter of a
/// logical pixel per pixel.
const MIN_WIRE_120THS: u32 = 30;

/// The largest preferred scale [`Scale120::from_wire`] accepts, in 120ths: ten pixels per
/// logical pixel.
const MAX_WIRE_120THS: u32 = 1200;

/// The largest whole `wl_surface.preferred_buffer_scale` [`Scale120::from_integer_factor`]
/// accepts.
const MAX_INTEGER_FACTOR: i32 = 10;

/// A fractional scale in 120ths (`180` is 1.5), the unit Wayland's `wp_fractional_scale` uses.
/// The one type for it below the design system: shell-host's surface scale and the kit's pixel
/// snap are this value, so a host passes it straight through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Scale120(pub u32);

impl Scale120 {
    /// 1.0x, the scale a surface has until told otherwise.
    pub const ONE: Scale120 = Scale120(120);

    /// The scale of a `wp_fractional_scale_v1.preferred_scale` event's raw value, kept between a
    /// quarter and ten times: a scale outside that is a compositor bug, and would size a buffer
    /// beyond any device.
    pub fn from_wire(value: u32) -> Scale120 {
        Scale120(value.clamp(MIN_WIRE_120THS, MAX_WIRE_120THS))
    }

    /// The scale of a whole buffer-scale factor (`wl_output.scale`, `wl_surface`'s
    /// `scale_factor_changed`), at least 1 and at most 10.
    pub fn from_integer_factor(factor: i32) -> Scale120 {
        let whole = u32::try_from(factor.clamp(1, MAX_INTEGER_FACTOR)).unwrap_or(1);
        Scale120(whole * Scale120::ONE.0)
    }

    /// The nearest 120th to a device-pixels-per-logical-pixel `factor` (`1.5` is `180`).
    pub fn from_factor(factor: f64) -> Scale120 {
        Scale120((factor * 120.0).round().max(0.0) as u32)
    }

    /// Device pixels per logical pixel, for the one place arithmetic needs a float (sizing a
    /// buffer). Never stored.
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
