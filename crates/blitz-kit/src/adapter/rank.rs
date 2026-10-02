//! The order to try adapters in.

use super::{AdapterFacts, AdapterPref, DeviceKind, GpuBackend};

/// The order to try `candidates` in (indices into it) under `pref`: a `Named` match (case-
/// insensitive substring) or a `Device` match (PCI ids) first, then by kind as the preference says, Vulkan before GL, and
/// software rasterizers last. Pure.
pub fn rank(pref: &AdapterPref, candidates: &[AdapterFacts]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..candidates.len()).collect();
    // A stable sort keeps the enumeration order (wgpu's own) among equals.
    order.sort_by_key(|&i| sort_key(pref, &candidates[i]));
    order
}

/// Smaller is tried first: (preferred-adapter miss, software, kind rank, backend rank).
fn sort_key(pref: &AdapterPref, facts: &AdapterFacts) -> (u8, u8, u8, u8) {
    let miss = match pref {
        AdapterPref::Named(wanted) => u8::from(!names_match(&facts.name, wanted)),
        AdapterPref::Device { vendor, device } => {
            u8::from((facts.vendor, facts.device) != (*vendor, *device))
        }
        _ => 0,
    };
    let software = u8::from(facts.kind == DeviceKind::Cpu);
    let backend = match facts.backend {
        GpuBackend::Vulkan => 0,
        GpuBackend::Gl => 1,
    };
    (miss, software, kind_rank(pref, facts.kind), backend)
}

/// The adapter name contains `wanted`, ignoring case. A substring by definition: this is
/// `WGPU_ADAPTER_NAME`'s own contract ("NVIDIA" names "NVIDIA GeForce RTX 5070 Ti").
fn names_match(name: &str, wanted: &str) -> bool {
    name.to_lowercase().contains(&wanted.to_lowercase())
}

fn kind_rank(pref: &AdapterPref, kind: DeviceKind) -> u8 {
    let order: [DeviceKind; 5] = match pref {
        AdapterPref::Integrated => [
            DeviceKind::Integrated,
            DeviceKind::Discrete,
            DeviceKind::Virtual,
            DeviceKind::Other,
            DeviceKind::Cpu,
        ],
        // Auto is wgpu's high-performance order; Named and Device fall back to it after the
        // matches.
        AdapterPref::Auto
        | AdapterPref::Discrete
        | AdapterPref::Named(_)
        | AdapterPref::Device { .. } => [
            DeviceKind::Discrete,
            DeviceKind::Integrated,
            DeviceKind::Virtual,
            DeviceKind::Other,
            DeviceKind::Cpu,
        ],
    };
    order
        .iter()
        .position(|k| *k == kind)
        .map_or(u8::MAX, |p| p as u8)
}
