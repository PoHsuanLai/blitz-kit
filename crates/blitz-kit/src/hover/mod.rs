//! Hover changes a resolve makes on its own.
//!
//! Blitz ends every `resolve` with `refresh_hover()`, which re-hit-tests the last pointer
//! position against the fresh layout and moves the hovered node without dispatching anything
//! (its own TODO: no enter/leave synthesis). The event driver diffs a move against that hover
//! state, so an element that slides in under a resting pointer never gets its
//! `pointerenter`/`mouseenter`, and the one it covered never gets its leave, until the pointer
//! leaves the chain and comes back.
//!
//! The repair goes around the resolve ([`repair`]): when the hovered element changed and the
//! document has seen a real move, put the old hover back without dispatch
//! ([`HoverSync::Restore`] at a point that hits it, or [`HoverSync::Clear`] when none does),
//! then feed the last move again, so the document's own driver dispatches exactly the
//! leave/enter diff. [`decide`] chooses which, and [`remember`] keeps the move.

mod decide;
mod last_move;
mod repair;

pub use decide::{HoverSync, Shift, decide, probe_points};
pub use last_move::{LastMove, remember};
pub use repair::{Repaired, repair};

#[cfg(test)]
mod tests;
