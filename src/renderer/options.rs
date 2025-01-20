use phf::phf_map;
use serde::{Deserialize, Serialize};
#[cfg(wasm)]
use wasm_bindgen::prelude::wasm_bindgen;

/// Convenience Lookup Table for converting a static string
/// from the external API into a the wgpu::PowerPreference enum.
pub static POWER_PREFERENCE: phf::Map<&str, wgpu::PowerPreference> = phf_map! {
    // This requests an adapter with high performance, often a discrete GPU.
    // It will result in the best performance, but will consume more power.
    "high-performance" => wgpu::PowerPreference::HighPerformance,
    "default" => wgpu::PowerPreference::HighPerformance,

    // This requests an adapter with low power usage, often an integrated GPU.
    // It will generally result in lower performance, but will consume less power.
    "low-power" => wgpu::PowerPreference::LowPower,
    "low" => wgpu::PowerPreference::LowPower,

    // This requests the first available GPU adapter, regardless of power usage.
    "no-preference" => wgpu::PowerPreference::None,
    "none" => wgpu::PowerPreference::None,
    "" =>wgpu::PowerPreference::None,
};

#[cfg_attr(wasm, wasm_bindgen(getter_with_clone))]
/// Options for configuring the Renderer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererOptions {
    pub force_software_rendering: bool,
    pub power_preference: String,
    pub panic_on_error: bool,
    pub device_limits: String,
}

impl Default for RendererOptions {
    fn default() -> Self {
        Self {
            force_software_rendering: false,
            power_preference: "default".to_string(),
            panic_on_error: false,
            device_limits: "default".to_string(),
        }
    }
}
