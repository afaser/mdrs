#[derive(Debug, Clone)]
pub enum Gpu {
    Hardware(String),
    Software(String),
}

#[derive(Debug, Clone, Default)]
pub enum GpuState {
    #[default]
    Unknown,
    Checking,
    Known(Gpu),
}

impl GpuState {
    pub fn begin_check(&mut self) -> bool {
        match self {
            Self::Unknown => {
                *self = Self::Checking;
                true
            }
            Self::Checking | Self::Known(_) => false,
        }
    }

    pub fn apply(&mut self, gpu: Gpu) {
        if let Gpu::Hardware(name) = &gpu {
            log::info!("selected wgpu adapter: {name}");
        }

        *self = Self::Known(gpu);
    }

    pub fn software_name(&self) -> Option<&str> {
        match self {
            Self::Known(Gpu::Software(name)) => Some(name),
            Self::Unknown | Self::Checking | Self::Known(Gpu::Hardware(_)) => None,
        }
    }
}

pub fn detect() -> Gpu {
    use iced::wgpu::{Backends, DeviceType, Instance, InstanceDescriptor};

    let instance = Instance::new(&InstanceDescriptor {
        backends: Backends::VULKAN,
        ..InstanceDescriptor::default()
    });
    let adapters: Vec<_> = instance
        .enumerate_adapters(Backends::VULKAN)
        .iter()
        .map(|adapter| adapter.get_info())
        .collect();

    for info in &adapters {
        log::info!("wgpu adapter: {} ({:?})", info.name, info.device_type);
    }

    let selected = adapters
        .iter()
        .find(|info| info.device_type != DeviceType::Cpu)
        .or_else(|| adapters.first());

    match selected {
        Some(info) if info.device_type != DeviceType::Cpu => Gpu::Hardware(info.name.clone()),
        Some(info) => Gpu::Software(info.name.clone()),
        None => Gpu::Software("no Vulkan adapter".to_owned()),
    }
}
