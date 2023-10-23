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

#[derive(Default, Debug)]
pub struct RenderOptions<'w, W: IsWindow> {
    pub force_software_rendering: Option<bool>,
    pub power_preference: Option<&'static str>,
    pub device_limits: Option<&'static str>,
    pub targets: Option<Vec<&'w W>>,
}
