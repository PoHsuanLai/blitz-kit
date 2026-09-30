//! Which GPU a shell prefers.

use serde::{Deserialize, Serialize};

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
