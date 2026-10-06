//! Portable Blitz repairs and helpers: no Wayland, no Dioxus, no design system. Everything takes
//! and returns CSS px `f64` in Blitz's own units ([`units`]); a consumer converts at its own
//! boundary.

#[cfg(feature = "adapter")]
pub mod adapter;
pub mod data_url;
pub mod element_id;
pub mod fonts;
pub mod hit;
pub mod hover;
pub mod net;
pub mod paint_rect;
pub mod scroll;
pub mod snap;
pub mod units;
