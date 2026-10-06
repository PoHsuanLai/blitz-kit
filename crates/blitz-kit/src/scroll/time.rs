//! The time a scroll step runs at: a point on any one fixed timeline (the host's start, a
//! window's first event). The engine only ever subtracts two of them, so its callers choose the
//! origin and pass the same one every time; the pure modules never read a clock.

use std::time::Duration;

/// A point on the caller's timeline, as the time since its origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Elapsed(pub Duration);
