//! Which repair a hover shift needs: the pure decision.

use super::LastMove;
use crate::units::{Bounds, PagePoint};

/// The hovered element before and after one resolve (an element: a hovered text node counts
/// as its element, which is what carries the listeners).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shift<N> {
    pub before: Option<N>,
    pub after: Option<N>,
}

/// What to do after a resolve, before the move is fed again.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HoverSync {
    /// Nothing moved, or no move to feed again.
    Unchanged,
    /// Hover this point (no dispatch): it hits the element hovered before.
    Restore(PagePoint),
    /// Hover nothing (no dispatch): nothing was hovered before, or the old element cannot be
    /// hit any more (removed from layout, or covered at every probe).
    Clear,
}

/// Where the old element is probed: its centre, then its corners 1 px inside, so an element
/// covered in the middle (a banner over part of a list) is still found at an edge.
pub fn probe_points(bounds: Bounds) -> [PagePoint; 5] {
    let inset_x = 1.0_f64.min(bounds.width / 2.0);
    let inset_y = 1.0_f64.min(bounds.height / 2.0);
    let (left, top) = (bounds.x + inset_x, bounds.y + inset_y);
    let right = bounds.x + bounds.width - inset_x;
    let bottom = bounds.y + bounds.height - inset_y;
    [
        PagePoint {
            x: bounds.x + bounds.width / 2.0,
            y: bounds.y + bounds.height / 2.0,
        },
        PagePoint { x: left, y: top },
        PagePoint { x: right, y: top },
        PagePoint { x: left, y: bottom },
        PagePoint {
            x: right,
            y: bottom,
        },
    ]
}

/// The repair for `shift`: `bounds` gives the old element's current box, `hit` the element at
/// a point. Both are only asked when the hover really moved under a known pointer.
pub fn decide<N: Copy + PartialEq>(
    shift: Shift<N>,
    last: &LastMove,
    bounds: impl FnOnce(N) -> Option<Bounds>,
    mut hit: impl FnMut(PagePoint) -> Option<N>,
) -> HoverSync {
    if matches!(last, LastMove::Unknown) || shift.before == shift.after {
        return HoverSync::Unchanged;
    }
    let Some(before) = shift.before else {
        return HoverSync::Clear;
    };
    bounds(before)
        .into_iter()
        .flat_map(probe_points)
        .find(|&point| hit(point) == Some(before))
        .map_or(HoverSync::Clear, HoverSync::Restore)
}
