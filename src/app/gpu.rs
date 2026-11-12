#[derive(Debug, Clone, PartialEq)]
pub enum Gpu {
    Hardware(String),
    Software(String),
}

#[derive(Debug, Clone, Default)]
pub(super) enum GpuState {
    #[default]
    Unknown,
    Checking,
    Known(Gpu),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GpuCheck {
    Started,
    Skipped,
}

impl GpuState {
    pub(super) fn begin_check(&mut self) -> GpuCheck {
        match self {
            Self::Unknown => {
                *self = Self::Checking;
                GpuCheck::Started
            }
            Self::Checking | Self::Known(_) => GpuCheck::Skipped,
        }
    }

    pub(super) fn apply(&mut self, gpu: Gpu) {
        *self = Self::Known(gpu);
    }

    pub(super) fn software_name(&self) -> Option<&str> {
        match self {
            Self::Known(Gpu::Software(name)) => Some(name),
            Self::Unknown | Self::Checking | Self::Known(Gpu::Hardware(_)) => None,
        }
    }
}

pub(super) fn detect() -> Gpu {
    use iced::wgpu::{Backends, Instance, InstanceDescriptor};

    let instance = Instance::new(&InstanceDescriptor {
        backends: Backends::VULKAN,
        ..InstanceDescriptor::default()
    });
    let adapters = instance.enumerate_adapters(Backends::VULKAN);

    let selected = select(
        adapters
            .iter()
            .map(|adapter| adapter.get_info())
            .inspect(|info| log::info!("wgpu adapter: {} ({:?})", info.name, info.device_type))
            .map(to_gpu),
    );

    if let Gpu::Hardware(name) = &selected {
        log::info!("selected wgpu adapter: {name}");
    }

    selected
}

pub(super) fn select(adapters: impl Iterator<Item = Gpu>) -> Gpu {
    adapters
        .reduce(preferred)
        .unwrap_or_else(|| Gpu::Software("no Vulkan adapter".to_owned()))
}

fn preferred(current: Gpu, candidate: Gpu) -> Gpu {
    match (current, candidate) {
        (hardware @ Gpu::Hardware(_), _) => hardware,
        (_, hardware @ Gpu::Hardware(_)) => hardware,
        (first, _) => first,
    }
}

fn to_gpu(info: iced::wgpu::AdapterInfo) -> Gpu {
    if is_cpu(info.device_type) {
        return Gpu::Software(info.name);
    }

    Gpu::Hardware(info.name)
}

fn is_cpu(device_type: iced::wgpu::DeviceType) -> bool {
    device_type == iced::wgpu::DeviceType::Cpu
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardware_adapter_is_preferred() {
        let selected = select(
            [
                Gpu::Software("cpu".to_owned()),
                Gpu::Hardware("gpu".to_owned()),
            ]
            .into_iter(),
        );

        assert_eq!(selected, Gpu::Hardware("gpu".to_owned()));
    }

    #[test]
    fn empty_adapter_list_falls_back_to_software() {
        assert_eq!(
            select(std::iter::empty()),
            Gpu::Software("no Vulkan adapter".to_owned())
        );
    }
}
