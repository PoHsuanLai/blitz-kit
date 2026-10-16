//! Choosing the adapter: the facts about each wgpu adapter, the preference a shell configures,
//! and the order to try the candidates in. Validating a candidate by a first present needs a
//! Wayland surface, so that half stays with the host.

mod device;
mod facts;
mod pref;
mod rank;

#[cfg(test)]
mod tests;

pub use device::{
    ADAPTER_ENV, DeviceError, DeviceHealth, Health, WatchedDevice, block_on, block_on_timeout,
    ranked, request_device, request_watched_device,
};
pub use facts::{AdapterFacts, DeviceKind, GpuBackend};
pub use pref::{AdapterPref, PciDevice, PciVendor};
pub use rank::rank;
