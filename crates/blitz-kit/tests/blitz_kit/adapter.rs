//! Adapter ranking, offscreen: the pure table, no adapter is opened.

use blitz_kit::adapter::{
    AdapterFacts, AdapterPref, DeviceKind, GpuBackend, PciDevice, PciVendor, rank,
};

fn facts(name: &str, kind: DeviceKind, backend: GpuBackend) -> AdapterFacts {
    AdapterFacts {
        name: name.into(),
        vendor: PciVendor(0),
        device: PciDevice(0),
        kind,
        backend,
        driver: "test".into(),
    }
}

const AMD: PciVendor = PciVendor(0x1002);
const RADEON_IGPU: PciDevice = PciDevice(0x13c0);
const NVIDIA: PciVendor = PciVendor(0x10de);
const RTX_5070_TI: PciDevice = PciDevice(0x2c05);

fn with_ids(mut facts: AdapterFacts, vendor: PciVendor, device: PciDevice) -> AdapterFacts {
    facts.vendor = vendor;
    facts.device = device;
    facts
}

/// This machine's adapters as wgpu enumerates them, plus a software rasterizer.
fn this_machine() -> Vec<AdapterFacts> {
    let amd = |f| with_ids(f, AMD, RADEON_IGPU);
    let nvidia = |f| with_ids(f, NVIDIA, RTX_5070_TI);
    vec![
        amd(facts(
            "AMD Radeon Graphics (RADV)",
            DeviceKind::Integrated,
            GpuBackend::Vulkan,
        )),
        nvidia(facts(
            "NVIDIA GeForce RTX 5070 Ti",
            DeviceKind::Discrete,
            GpuBackend::Vulkan,
        )),
        facts("llvmpipe (LLVM 20)", DeviceKind::Cpu, GpuBackend::Vulkan),
        nvidia(facts(
            "NVIDIA GeForce RTX 5070 Ti",
            DeviceKind::Discrete,
            GpuBackend::Gl,
        )),
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
        (
            "the compositor's device, by PCI ids, ahead of the default order",
            AdapterPref::Device {
                vendor: AMD,
                device: RADEON_IGPU,
            },
            &[0, 1, 3, 4, 2],
        ),
        (
            "the compositor's device matches both backends, vulkan first",
            AdapterPref::Device {
                vendor: NVIDIA,
                device: RTX_5070_TI,
            },
            &[1, 3, 0, 4, 2],
        ),
        (
            "right vendor, other device falls back to auto",
            AdapterPref::Device {
                vendor: AMD,
                device: PciDevice(0x9999),
            },
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
        vendor: NVIDIA,
        device: RTX_5070_TI,
        kind: DeviceKind::Discrete,
        backend: GpuBackend::Vulkan,
        driver: "NVIDIA 615.71".into(),
    };
    assert_eq!(
        nvidia.label(),
        "NVIDIA GeForce RTX 5070 Ti (Vulkan, NVIDIA 615.71)"
    );
}
