//! The scroll engine of design/11-BEHAVIOUR-scroll, shared by every Blitz host: the physics
//! (detent latching, the smooth step, momentum, the rubber band, keyboard steps), the document
//! side (finding the scroller under a point, geometry, raw offset writes, the `data-wheel` and
//! `data-overscroll` markers) and a driver that joins them for one document.
//!
//! Pure and table-tested: `config`, `geom`, `smooth`, `keys`, `velocity`, `momentum`, `rubber`,
//! `latch`, `engine`, `animate`, `pad`, `cmd`, `target`. Touching a document: `doc`, `driver`.
//! A host adds only the translation from its own device events (Wayland axis frames, winit's
//! `MouseWheel`), its clock (`time::Elapsed`) and its frame requests.

mod animate;
pub mod cmd;
pub mod config;
pub mod doc;
pub mod driver;
pub mod engine;
pub mod geom;
pub mod keys;
pub mod latch;
mod momentum;
pub mod pad;
pub mod rubber;
pub mod smooth;
pub mod target;
pub mod time;
pub mod tuning;
pub mod velocity;

#[cfg(test)]
mod tests;
