//! Portable Blitz repairs and helpers: no Wayland, no Dioxus, no design system. Everything takes
//! and returns CSS px `f64` in Blitz's own units ([`units`]); a consumer converts at its own
//! boundary.
//!
//! # Modules
//!
//! - [`after_resolve`]: readings a host takes from its document after every resolve.
//! - [`scroll`]: the scroll engine (physics, keys, rubber band, a driver per document).
//! - [`hover`]: repairs Blitz's hover when an element arrives under a resting pointer.
//! - [`hit`] and [`paint_rect`]: what a point hits, and an element's painted rectangle.
//! - [`snap`]: pixel snapping at fractional scales.
//! - [`fonts`]: one shared font context for every document.
//! - [`net`] and [`data_url`]: a local-only network provider and `data:` URL decoding.
//! - [`element_id`] and [`units`]: the small typed vocabulary the rest shares.
//! - `adapter` (feature `adapter`): ranking GPU adapters, and the one lock every wgpu opener in a
//!   process holds.
//!
//! Every module but `data_url`, `element_id`, `units` and `adapter` needs the default feature
//! `blitz`.
//!
//! ```
//! use blitz_kit::data_url::decode;
//!
//! assert_eq!(decode("data:text/plain;base64,aGk=").as_deref(), Some(&b"hi"[..]));
//! assert_eq!(decode("https://example.com/a.png"), None);
//! ```

#[cfg(feature = "adapter")]
pub mod adapter;
#[cfg(feature = "blitz")]
pub mod after_resolve;
pub mod data_url;
pub mod element_id;
#[cfg(feature = "blitz")]
pub mod fonts;
#[cfg(feature = "blitz")]
pub mod hit;
#[cfg(feature = "blitz")]
pub mod hover;
#[cfg(feature = "blitz")]
pub mod net;
#[cfg(feature = "blitz")]
pub mod paint_rect;
#[cfg(feature = "blitz")]
pub mod scroll;
#[cfg(feature = "blitz")]
pub mod snap;
pub mod units;
