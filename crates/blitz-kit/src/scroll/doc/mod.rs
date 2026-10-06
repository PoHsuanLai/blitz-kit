//! The scroll engine's view of a Blitz document: which scrollers are under a point, their
//! geometry, the raw offset writes, and the markers (`data-wheel`, `data-overscroll`,
//! `data-keys`). The only place the engine touches Blitz (design/11 §11.3.1, §11.7).
//!
//! Offsets are written raw: `set_viewport_scroll` for the viewport and `scroll_offset_mut` for
//! an element. Neither clamps, redraws, dispatches `scroll` or shows Blitz's scrollbars, and
//! nothing in `BaseDocument::resolve` clamps them back. Blitz's own scroll paths
//! (`scroll_chain_by` from a forwarded wheel, `scroll_to`, a fling) clamp and would undo a
//! stretch, so a host never lets Blitz scroll a wheel except over a `data-wheel="capture"`
//! element.

mod chain;
mod element;
mod geometry;
mod ids;
mod markers;

pub use chain::{chain_at, page_point};
pub use geometry::{geom, write};
pub use ids::{area, container_of, scroller_by_id};
pub use markers::{KeyFocus, WheelRoute, key_focus, wheel_route};
