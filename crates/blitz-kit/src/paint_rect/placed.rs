//! The transformed boxes on the way from an element to the root, and the rectangle carried
//! through them.

use super::Affine2;
use crate::units::{Bounds, PagePoint};

/// One transformed box on the way from an element up to the root: its untransformed border-box
/// origin, in the coordinates of the rectangle being carried, and its transform about it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    pub origin: PagePoint,
    pub transform: Affine2,
}

/// Where `bounds` (a layout rectangle) paints, given the transformed boxes from the element
/// itself outward (`chain[0]` is the innermost): the bounding box of its four corners carried
/// through each one in turn. An empty chain returns `bounds` unchanged.
pub fn painted_bounds(bounds: Bounds, chain: &[Placed]) -> Bounds {
    let corners = [
        (bounds.x, bounds.y),
        (bounds.x + bounds.width, bounds.y),
        (bounds.x, bounds.y + bounds.height),
        (bounds.x + bounds.width, bounds.y + bounds.height),
    ]
    .map(|(x, y)| chain.iter().fold(PagePoint { x, y }, carry));
    let left = corners.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let top = corners.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let right = corners
        .iter()
        .map(|p| p.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let bottom = corners
        .iter()
        .map(|p| p.y)
        .fold(f64::NEG_INFINITY, f64::max);
    Bounds {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

/// `point` through one box's transform about its origin: `O + T (p - O)`.
fn carry(point: PagePoint, placed: &Placed) -> PagePoint {
    let local = PagePoint {
        x: point.x - placed.origin.x,
        y: point.y - placed.origin.y,
    };
    let moved = placed.transform.apply(local);
    PagePoint {
        x: moved.x + placed.origin.x,
        y: moved.y + placed.origin.y,
    }
}
