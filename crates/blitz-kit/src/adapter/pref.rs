//! Which GPU a shell prefers.

use serde::{Deserialize, Serialize};

/// A PCI vendor id (`0x10de` is NVIDIA); `0` when the adapter has none (software rasterizers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PciVendor(pub u32);

/// A PCI device id, meaningful within one vendor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PciDevice(pub u32);

/// Which adapter the GPU backend prefers. A preference, not a guarantee: every candidate is
/// validated by a first present on a probe surface over a throwaway connection, because a
/// cross-GPU adapter can pass `is_surface_supported` and still have its first buffer rejected
/// with a fatal protocol error (RADV on this machine, `zwp_linux_dmabuf` error 7).
/// A candidate that fails is dropped and the next one is tried.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum AdapterPref {
    /// wgpu's default order for the surface (high performance first).
    #[default]
    Auto,
    /// The first adapter whose name contains this text, case-insensitively.
    Named(String),
    /// The adapter with this PCI vendor and device id: the GPU the compositor renders on, so
    /// the shell's buffers need no cross-GPU copy. Falls back to `Auto`'s order when no
    /// adapter has the ids.
    Device {
        vendor: PciVendor,
        device: PciDevice,
    },
    /// Discrete GPUs first.
    Discrete,
    /// Integrated GPUs first.
    Integrated,
}

impl AdapterPref {
    /// The preference in force once the environment is read: a non-empty `WGPU_ADAPTER_NAME`
    /// wins over whatever was configured.
    pub fn with_env(self, env_value: Option<&str>) -> AdapterPref {
        match env_value.map(str::trim) {
            Some(name) if !name.is_empty() => AdapterPref::Named(name.to_owned()),
            _ => self,
        }
    }
}
