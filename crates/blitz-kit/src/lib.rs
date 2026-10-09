//! Portable Blitz repairs and helpers: no Wayland, no Dioxus, no design system. Everything takes
//! and returns CSS px `f64` in Blitz's own units ([`units`]); a consumer converts at its own
//! boundary.
//!
//! # Modules
//!
//! - [`scroll`]: the scroll engine (physics, keys, rubber band, a driver per document).
//! - [`hover`]: repairs Blitz's hover when an element arrives under a resting pointer.
//! - [`hit`] and [`paint_rect`]: what a point hits, and an element's painted rectangle.
//! - [`snap`]: pixel snapping at fractional scales.
//! - [`fonts`]: one shared font context for every document.
//! - [`net`] and [`data_url`]: a local-only network provider and `data:` URL decoding.
//! - [`element_id`] and [`units`]: the small typed vocabulary the rest shares.
//! - `adapter` (feature `adapter`): ranking GPU adapters.
//!
//! ```
//! use blitz_kit::data_url::decode;
//!
//! assert_eq!(decode("data:text/plain;base64,aGk=").as_deref(), Some(&b"hi"[..]));
//! assert_eq!(decode("https://example.com/a.png"), None);
//! ```

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
