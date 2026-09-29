//! Where a laid-out box paints: its layout rectangle carried through its own CSS transform and
//! every transformed ancestor's. The pure part.
//!
//! Blitz's `get_client_bounding_rect` is the layout box: `transform` never moves it (layout
//! ignores transforms by design), so a region or popup anchor built from it stays where the box
//! was laid out while the box paints somewhere else (a swiped or leaving notification banner
//! left a grey blur rectangle behind). The paint (`blitz-paint`'s `render_element`) composes
//! `parent * translate(location) * node.transform()` down the tree; the same map, written in
//! page coordinates, is the composition of each transformed box's transform *about its own
//! untransformed origin*, innermost first:
//!
//! ```text
//! painted(p) = F_root(... F_parent(F_node(p)))    F_a(p) = O_a + T_a (p - O_a)
//! ```
//!
//! where `O_a` is box `a`'s untransformed border-box origin and `T_a` its transform (with
//! `transform-origin`, `translate`, `rotate` and `scale` already folded in by Blitz). The
//! rectangle's four corners go through that map, and the result is their bounding box:
//! exact for translation and scale, the axis-aligned bounding box of the turned quad for a
//! rotation or skew (a region is a union of axis-aligned rectangles, so that is the most a
//! region can say). Not handled: 3-D transforms (Blitz drops a non-2-D matrix itself, so the
//! box paints untransformed and the rect agrees), clipping by an ancestor's `overflow`, and
//! opacity (a faded box keeps its region until it is removed).

mod affine;
mod chain;
mod placed;

#[cfg(test)]
mod tests;

pub use affine::Affine2;
pub use chain::painted_rect;
pub use placed::{Placed, painted_bounds};
