//! Choosing the adapter: the facts about each wgpu adapter, the preference a shell configures,
//! and the order to try the candidates in. Validating a candidate by a first present needs a
//! Wayland surface, so that half stays with the host.

mod facts;
mod pref;
mod rank;

#[cfg(test)]
mod tests;

pub use facts::{AdapterFacts, DeviceKind, GpuBackend};
pub use pref::AdapterPref;
pub use rank::rank;
