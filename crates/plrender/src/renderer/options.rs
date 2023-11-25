use crate::renderer::limits::{DEFAULT_LIMITS, DOWNLEVEL_DEFAULTS, DOWNLEVEL_WEBGL2};
use phf::phf_map;
use serde::{Deserialize, Serialize};

/// The default render pass (2D).
const DEFAULT_RENDER_PASS: &str = "flat";

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

/// Convenience Lookup Table for converting a static string
/// from the external API into a the wgpu::Limits struct.
pub static DEVICE_LIMITS: phf::Map<&str, wgpu::Limits> = phf_map! {
    // Limits::downlevel_defaults(). This is a set of limits that is guaranteed
    // to work on almost all backends, including "downlevel" backends such as
    // OpenGL and D3D11, other than WebGL. For most applications we recommend
    // using these limits, assuming they are high enough for your application,
    // and you do not intent to support WebGL.
    "downlevel_defaults" => DOWNLEVEL_DEFAULTS,
    "opengl" => DOWNLEVEL_DEFAULTS,
    "d3d11" => DOWNLEVEL_DEFAULTS,

    // Limits::downlevel_webgl2_defaults() This is a set of limits that is lower
    // even than the [downlevel_defaults()], configured to be low enough to support
    // running in the browser using WebGL2.
    "downlevel_webgl2_defaults" => DOWNLEVEL_WEBGL2,
    "downlevel_webgl2" => DOWNLEVEL_WEBGL2,
    "webgl2" => DOWNLEVEL_WEBGL2,
    "webgl" => DOWNLEVEL_WEBGL2,

    // Limits::default(). This is the set of limits that is guaranteed to work on
    // all modern backends and is guaranteed to be supported by WebGPU. Applications
    // needing more modern features can use this as a reasonable set of limits if
    // they are targeting only desktop and modern mobile devices.
    "default" => DEFAULT_LIMITS,
};

/// Options for configuring the Renderer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererOptions {
    pub force_software_rendering: bool,
    pub power_preference: String,
    pub panic_on_error: bool,
    pub device_limits: String,
    pub render_pass: String, // supports only ("flat" or "solid") for now, not chainable yet
}

impl Default for RendererOptions {
    fn default() -> Self {
        Self {
            force_software_rendering: false,
            power_preference: "default".to_string(),
            panic_on_error: false,
            device_limits: "default".to_string(),
            render_pass: DEFAULT_RENDER_PASS.to_string(),
        }
    }
}
