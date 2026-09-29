//! A 2-D affine map in CSS px.

use crate::units::PagePoint;

/// A 2-D affine map in CSS px, in kurbo's coefficient order: `(x, y)` goes to
/// `(a x + c y + e, b x + d y + f)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine2 {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Affine2 {
    /// The map that moves nothing.
    pub const IDENTITY: Affine2 = Affine2 {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    /// A move by `(x, y)`.
    pub const fn translate(x: f64, y: f64) -> Affine2 {
        Affine2 {
            e: x,
            f: y,
            ..Affine2::IDENTITY
        }
    }

    /// A scale by `(x, y)` about the origin.
    pub const fn scale(x: f64, y: f64) -> Affine2 {
        Affine2 {
            a: x,
            d: y,
            ..Affine2::IDENTITY
        }
    }

    /// A turn by `radians` (clockwise on screen, y down) about the origin.
    pub fn rotate(radians: f64) -> Affine2 {
        let (sin, cos) = radians.sin_cos();
        Affine2 {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Blitz's stored transform (`Node::transform`, kurbo coefficients) back in CSS px: Blitz
    /// keeps it in device px (`S * T * S^-1`: the translation is multiplied by the viewport
    /// `scale`, the linear part is not), so divide the translation back out. A non-positive
    /// scale is taken as 1.
    pub fn from_device(coeffs: [f64; 6], scale: f64) -> Affine2 {
        let scale = if scale > 0.0 { scale } else { 1.0 };
        let [a, b, c, d, e, f] = coeffs;
        Affine2 {
            a,
            b,
            c,
            d,
            e: e / scale,
            f: f / scale,
        }
    }

    pub(super) fn apply(self, point: PagePoint) -> PagePoint {
        PagePoint {
            x: self.a * point.x + self.c * point.y + self.e,
            y: self.b * point.x + self.d * point.y + self.f,
        }
    }
}
