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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_size_and_refs_fill_defaults() {
        let s: Size = [3u32, 4u32].into();
        let o1 = TextureOptions::from(s);
        assert_eq!(o1.size, Some(Size::new(3, 4, None)));
        assert_eq!(o1.format, TextureFormat::default());
        assert_eq!(o1.sampler, SamplerOptions::default());

        let s2: Size = [5u32, 6u32, 7u32].into();
        let o2 = TextureOptions::from(&s2);
        assert_eq!(o2.size, Some(s2));
        assert_eq!(o2.format, TextureFormat::default());
        assert_eq!(o2.sampler, SamplerOptions::default());
    }

    #[test]
    fn from_format_and_refs_clear_size() {
        let fmt = TextureFormat::default();
        let o1 = TextureOptions::from(fmt);
        assert_eq!(o1.size, None);
        assert_eq!(o1.format, fmt);
        assert_eq!(o1.sampler, SamplerOptions::default());

        let fmt2 = TextureFormat::default();
        let o2 = TextureOptions::from(&fmt2);
        assert_eq!(o2.size, None);
        assert_eq!(o2.format, fmt2);
        assert_eq!(o2.sampler, SamplerOptions::default());
    }
}
