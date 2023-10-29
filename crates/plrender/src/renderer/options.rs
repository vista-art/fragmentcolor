use crate::IsWindow;
use phf::phf_map;

pub static POWER_PREFERENCE: phf::Map<&str, wgpu::PowerPreference> = phf_map! {
    "high-performance" => wgpu::PowerPreference::HighPerformance,
    "high_performance" => wgpu::PowerPreference::HighPerformance,
    "high performance" => wgpu::PowerPreference::HighPerformance,
    "performance" => wgpu::PowerPreference::HighPerformance,
    "default" => wgpu::PowerPreference::HighPerformance,
    "high" => wgpu::PowerPreference::HighPerformance,
    "hi" => wgpu::PowerPreference::HighPerformance,
    "low-power" => wgpu::PowerPreference::LowPower,
    "low_power" => wgpu::PowerPreference::LowPower,
    "low power" => wgpu::PowerPreference::LowPower,
    "low" => wgpu::PowerPreference::LowPower,
    "lo" => wgpu::PowerPreference::LowPower,
    "no-preference" => wgpu::PowerPreference::None,
    "no_preference" => wgpu::PowerPreference::None,
    "no preference" => wgpu::PowerPreference::None,
    "none" => wgpu::PowerPreference::None,
    "" =>wgpu::PowerPreference::None,
};

#[derive(Debug)]
pub struct RenderOptions<W: IsWindow> {
    pub force_software_rendering: Option<bool>,
    pub power_preference: Option<&'static str>,
    pub device_limits: Option<&'static str>,
    pub targets: Option<Vec<W>>,
}

impl<W: IsWindow> Default for RenderOptions<W> {
    fn default() -> Self {
        Self {
            force_software_rendering: Some(false),
            power_preference: Some("default"),
            device_limits: None,
            targets: None,
        }
    }
}
