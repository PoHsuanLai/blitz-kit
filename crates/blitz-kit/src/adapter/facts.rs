//! What is true of one adapter.

/// What kind of device an adapter is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceKind {
    Discrete,
    Integrated,
    Virtual,
    /// A software rasterizer (llvmpipe, lavapipe).
    Cpu,
    Other,
}

/// Which wgpu backend an adapter runs on. Vulkan first, GL second (plan "GPU").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpuBackend {
    Vulkan,
    Gl,
}

/// The facts about an adapter the ranking and the stats need.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AdapterFacts {
    pub name: String,
    pub kind: DeviceKind,
    pub backend: GpuBackend,
    pub driver: String,
}

impl AdapterFacts {
    /// The facts of a wgpu adapter.
    pub fn of(adapter: &wgpu::Adapter) -> AdapterFacts {
        let info = adapter.get_info();
        AdapterFacts {
            name: info.name,
            kind: kind_of(info.device_type),
            backend: backend_of(info.backend),
            driver: [info.driver, info.driver_info]
                .iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

    /// `name (backend, driver)`, as `FrameStats::adapter` reports it.
    pub fn label(&self) -> String {
        format!("{} ({:?}, {})", self.name, self.backend, self.driver)
    }
}

fn kind_of(device: wgpu::DeviceType) -> DeviceKind {
    match device {
        wgpu::DeviceType::DiscreteGpu => DeviceKind::Discrete,
        wgpu::DeviceType::IntegratedGpu => DeviceKind::Integrated,
        wgpu::DeviceType::VirtualGpu => DeviceKind::Virtual,
        wgpu::DeviceType::Cpu => DeviceKind::Cpu,
        wgpu::DeviceType::Other => DeviceKind::Other,
    }
}

/// The instance enables only Vulkan and GL (`GpuContext::instance`), and Linux has no other
/// The instance enables only Vulkan and GL, and Linux has no other native backend, so
/// anything that is not GL is Vulkan.
fn backend_of(backend: wgpu::Backend) -> GpuBackend {
    match backend {
        wgpu::Backend::Gl => GpuBackend::Gl,
        _ => GpuBackend::Vulkan,
    }
}
