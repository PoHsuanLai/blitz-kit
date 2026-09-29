//! Portable Blitz repairs and helpers: no Wayland, no Dioxus, no design system. Everything takes
//! and returns CSS px `f64` in Blitz's own units ([`units`]); a consumer converts at its own
//! boundary.

pub mod fonts;
pub mod hit;
pub mod hover;
pub mod net;
pub mod units;
