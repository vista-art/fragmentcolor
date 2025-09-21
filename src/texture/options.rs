use crate::{SamplerOptions, Size, TextureFormat};

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass)]
#[derive(Debug, Clone, Default)]
pub struct TextureOptions {
    pub size: Option<Size>,
    pub format: TextureFormat,
    pub sampler: SamplerOptions,
}

impl From<crate::Size> for TextureOptions {
    fn from(size: crate::Size) -> Self {
        TextureOptions {
            size: Some(size),
            format: TextureFormat::default(),
            sampler: SamplerOptions::default(),
        }
    }
}

impl From<&crate::Size> for TextureOptions {
    fn from(size: &crate::Size) -> Self {
        TextureOptions {
            size: Some(*size),
            format: TextureFormat::default(),
            sampler: SamplerOptions::default(),
        }
    }
}

impl From<TextureFormat> for TextureOptions {
    fn from(format: TextureFormat) -> Self {
        TextureOptions {
            size: None,
            format,
            sampler: SamplerOptions::default(),
        }
    }
}

impl From<&TextureFormat> for TextureOptions {
    fn from(format: &TextureFormat) -> Self {
        TextureOptions {
            size: None,
            format: *format,
            sampler: SamplerOptions::default(),
        }
    }
}

// @TODO move TextureOptions to its own file and implement more conversions
//      reuse the impl from reference macros (look at UniformData for reference)
