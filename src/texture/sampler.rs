#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass)]
#[derive(Debug, Copy, Clone)]
pub struct SamplerOptions {
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub smooth: bool,
    pub compare: Option<CompareFunction>,
}

#[cfg_attr(python, pyo3::pyclass)]
#[derive(Debug, Clone, PartialEq)]
pub struct SamplerInfo {
    pub comparison: bool,
}

impl Default for SamplerOptions {
    fn default() -> Self {
        Self {
            repeat_x: false,
            repeat_y: false,
            smooth: true,
            compare: None,
        }
    }
}

pub fn create_default_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    create_sampler(device, SamplerOptions::default())
}

pub fn create_sampler(device: &wgpu::Device, options: SamplerOptions) -> wgpu::Sampler {
    let label = format!("{:?}", options);
    let address_mode_u = match options.repeat_x {
        true => wgpu::AddressMode::Repeat,
        false => wgpu::AddressMode::ClampToEdge,
    };
    let address_mode_v = match options.repeat_y {
        true => wgpu::AddressMode::Repeat,
        false => wgpu::AddressMode::ClampToEdge,
    };
    let filter = match options.smooth {
        true => wgpu::FilterMode::Linear,
        false => wgpu::FilterMode::Nearest,
    };

    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(&label),
        address_mode_u,
        address_mode_v,
        address_mode_w: address_mode_v,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: filter,
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        compare: options.compare.map(Into::into),
        anisotropy_clamp: 1,
        border_color: None,
    })
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass)]
#[derive(Debug, Copy, Clone)]
pub enum CompareFunction {
    /// Function never passes
    Never = 1,
    /// Function passes if new value less than existing value
    Less = 2,
    /// Function passes if new value is equal to existing value. When using
    /// this compare function, make sure to mark your Vertex Shader's `@builtin(position)`
    /// output as `@invariant` to prevent artifacting.
    Equal = 3,
    /// Function passes if new value is less than or equal to existing value
    LessEqual = 4,
    /// Function passes if new value is greater than existing value
    Greater = 5,
    /// Function passes if new value is not equal to existing value. When using
    /// this compare function, make sure to mark your Vertex Shader's `@builtin(position)`
    /// output as `@invariant` to prevent artifacting.
    NotEqual = 6,
    /// Function passes if new value is greater than or equal to existing value
    GreaterEqual = 7,
    /// Function always passes
    Always = 8,
}

impl From<wgpu::CompareFunction> for CompareFunction {
    fn from(v: wgpu::CompareFunction) -> Self {
        match v {
            wgpu::CompareFunction::Never => CompareFunction::Never,
            wgpu::CompareFunction::Less => CompareFunction::Less,
            wgpu::CompareFunction::Equal => CompareFunction::Equal,
            wgpu::CompareFunction::LessEqual => CompareFunction::LessEqual,
            wgpu::CompareFunction::Greater => CompareFunction::Greater,
            wgpu::CompareFunction::NotEqual => CompareFunction::NotEqual,
            wgpu::CompareFunction::GreaterEqual => CompareFunction::GreaterEqual,
            wgpu::CompareFunction::Always => CompareFunction::Always,
        }
    }
}

impl From<CompareFunction> for wgpu::CompareFunction {
    fn from(v: CompareFunction) -> Self {
        match v {
            CompareFunction::Never => wgpu::CompareFunction::Never,
            CompareFunction::Less => wgpu::CompareFunction::Less,
            CompareFunction::Equal => wgpu::CompareFunction::Equal,
            CompareFunction::LessEqual => wgpu::CompareFunction::LessEqual,
            CompareFunction::Greater => wgpu::CompareFunction::Greater,
            CompareFunction::NotEqual => wgpu::CompareFunction::NotEqual,
            CompareFunction::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
            CompareFunction::Always => wgpu::CompareFunction::Always,
        }
    }
}
