//! The device grid and the box origin the rounding accumulates from.

/// Device pixels per logical pixel, at a fractional scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct DeviceGrid(pub(super) f64);

impl DeviceGrid {
    /// A logical coordinate moved to the nearest device pixel boundary, rounding halves up as
    /// taffy's own rounding does.
    pub(super) fn snap(self, logical: f64) -> f64 {
        (logical * self.0 + 0.5).floor() / self.0
    }

    /// The snapped extent of `length` starting at `start`: the difference of its snapped ends,
    /// so adjacent boxes share an edge exactly.
    pub(super) fn span(self, start: f64, length: f32) -> f32 {
        (self.snap(start + f64::from(length)) - self.snap(start)) as f32
    }

    /// A border width in whole device pixels, never below one when there is a border at all:
    /// the CSS rule for snapping a border width, kept whatever phase the box's edge lands on.
    pub(super) fn line(self, width: f32) -> f32 {
        if width <= 0.0 {
            return 0.0;
        }
        let device = (f64::from(width) * self.0).round().max(1.0);
        (device / self.0) as f32
    }
}

/// A box's top-left corner in document coordinates, before snapping.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct Origin {
    pub(super) x: f64,
    pub(super) y: f64,
}
