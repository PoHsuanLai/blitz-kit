//! Adapter ranking, offscreen: the pure table, no adapter is opened.

use blitz_kit::adapter::{AdapterFacts, AdapterPref, DeviceKind, GpuBackend, rank};

fn facts(name: &str, kind: DeviceKind, backend: GpuBackend) -> AdapterFacts {
    AdapterFacts {
        name: name.into(),
        kind,
        backend,
        driver: "test".into(),
    }
}

/// This machine's adapters as wgpu enumerates them, plus a software rasterizer.
fn this_machine() -> Vec<AdapterFacts> {
    vec![
        facts(
            "AMD Radeon Graphics (RADV)",
            DeviceKind::Integrated,
            GpuBackend::Vulkan,
        ),
        facts(
            "NVIDIA GeForce RTX 5070 Ti",
            DeviceKind::Discrete,
            GpuBackend::Vulkan,
        ),
        facts("llvmpipe (LLVM 20)", DeviceKind::Cpu, GpuBackend::Vulkan),
        facts(
            "NVIDIA GeForce RTX 5070 Ti",
            DeviceKind::Discrete,
            GpuBackend::Gl,
        ),
        facts("Virtio GPU", DeviceKind::Virtual, GpuBackend::Vulkan),
    ]
}

#[test]
fn adapters_are_tried_in_preference_order() {
    let cases: &[(&str, AdapterPref, &[usize])] = &[
        (
            "auto is high performance first",
            AdapterPref::Auto,
            &[1, 3, 0, 4, 2],
        ),
        ("discrete", AdapterPref::Discrete, &[1, 3, 0, 4, 2]),
        ("integrated", AdapterPref::Integrated, &[0, 1, 3, 4, 2]),
        (
            "named, any case",
            AdapterPref::Named("radv".into()),
            &[0, 1, 3, 4, 2],
        ),
        (
            "named matches both backends, vulkan first",
            AdapterPref::Named("NVIDIA".into()),
            &[1, 3, 0, 4, 2],
        ),
        (
            "named software beats the rule that puts it last",
            AdapterPref::Named("llvmpipe".into()),
            &[2, 1, 3, 0, 4],
        ),
        (
            "named nothing falls back to auto",
            AdapterPref::Named("Intel".into()),
            &[1, 3, 0, 4, 2],
        ),
    ];
    let candidates = this_machine();
    for (name, pref, want) in cases {
        assert_eq!(rank(pref, &candidates), *want, "{name}");
    }
}

#[test]
fn ranking_keeps_enumeration_order_among_equals_and_handles_none() {
    let twins = vec![
        facts("GPU A", DeviceKind::Discrete, GpuBackend::Vulkan),
        facts("GPU B", DeviceKind::Discrete, GpuBackend::Vulkan),
    ];
    assert_eq!(rank(&AdapterPref::Auto, &twins), vec![0, 1]);
    assert_eq!(rank(&AdapterPref::Auto, &[]), Vec::<usize>::new());
}

#[test]
fn the_label_is_name_backend_driver() {
    let nvidia = AdapterFacts {
        name: "NVIDIA GeForce RTX 5070 Ti".into(),
        kind: DeviceKind::Discrete,
        backend: GpuBackend::Vulkan,
        driver: "NVIDIA 615.71".into(),
    };
    assert_eq!(
        nvidia.label(),
        "NVIDIA GeForce RTX 5070 Ti (Vulkan, NVIDIA 615.71)"
    );
}
