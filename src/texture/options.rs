use crate::{SamplerOptions, Size, TextureFormat};

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

/// Bundles the input source and the options that shape texture creation into
/// one transport. Construct via the `From<T>` impls (which cover the common
/// shapes) or as a struct literal when you need explicit control.
///
/// The library never asks you to construct a `TextureInput` literally for the
/// common cases — `renderer.create_texture(bytes)`,
/// `renderer.create_texture((bytes, [w, h]))`, and
/// `renderer.create_texture((bytes, TextureFormat::Rgba8Unorm))` all work
/// because `Vec<u8>` / `&[u8]` / `Size` / `TextureFormat` etc. all `Into<TextureInput>`.
#[derive(Debug, Clone)]
pub struct TextureInput {
    pub data: crate::TextureData,
    pub options: TextureOptions,
}

// ---------------------------------------------------------------------------
// Bare input → TextureInput with default options.
//
// The blanket `impl<T: Into<TextureInput>> From<T>` would conflict with the
// std `impl<T> From<T> for T` (and with the tuple impls below if any tuple is
// also Into<TextureInput>), so we list the common input types explicitly.
// Each one mirrors an existing `From<T> for TextureInput` impl.
// ---------------------------------------------------------------------------

macro_rules! spec_from_input {
    ($($t:ty),+ $(,)?) => {
        $(
            impl From<$t> for TextureInput {
                fn from(input: $t) -> Self {
                    Self {
                        data: input.into(),
                        options: TextureOptions::default(),
                    }
                }
            }
        )+
    };
}

spec_from_input!(
    &[u8],
    Vec<u8>,
    &Vec<u8>,
    &std::path::Path,
    std::path::PathBuf,
    &std::path::PathBuf,
    &crate::Texture,
    crate::TextureData,
    crate::texture::TextureMipChain,
);

// ---------------------------------------------------------------------------
// (input, options-like) — second tuple element is anything that converts
// into `TextureOptions`. Since `Size: Into<TextureOptions>` and
// `TextureFormat: Into<TextureOptions>` (existing impls below), the same
// bound covers the four old methods at once: `_with_size`, `_with_format`,
// `_with`, and the bare `_prepared` form.
// ---------------------------------------------------------------------------

macro_rules! spec_from_input_with_options {
    ($($t:ty),+ $(,)?) => {
        $(
            impl<O: Into<TextureOptions>> From<($t, O)> for TextureInput {
                fn from((input, options): ($t, O)) -> Self {
                    Self {
                        data: input.into(),
                        options: options.into(),
                    }
                }
            }
        )+
    };
}

spec_from_input_with_options!(
    &[u8],
    Vec<u8>,
    &Vec<u8>,
    &std::path::Path,
    std::path::PathBuf,
    &std::path::PathBuf,
    &crate::Texture,
    crate::TextureData,
    crate::texture::TextureMipChain,
);

// ---------------------------------------------------------------------------
// Storage / size-driven shapes — `(size, format)` allocates an empty storage
// texture (data = TextureData::Empty); `(size, format, bytes)` pre-seeds it.
// These are the same `TextureInput` type the source-driven paths use, so
// `create_texture`, `create_storage_texture`, and `TextureMipChain::prepare`
// all read from one transport. The methods themselves do the validation
// (`create_storage_texture` requires `options.size.is_some()`; `prepare`
// requires `data` to be a sync-friendly variant).
// ---------------------------------------------------------------------------

impl<S: Into<Size>, F: Into<TextureFormat>> From<(S, F)> for TextureInput {
    fn from((size, format): (S, F)) -> Self {
        Self {
            data: crate::TextureData::Empty,
            options: TextureOptions {
                size: Some(size.into()),
                format: format.into(),
                ..Default::default()
            },
        }
    }
}

// Source-first 3-tuple: `(bytes, format, size)` — used by `TextureMipChain::prepare`
// for the raw-pixel path and by `Renderer::create_texture` whenever the caller
// has already-decoded bytes plus a known size + format. The size-first
// counterpart `(size, format, bytes)` lives below for the storage idiom; the
// two orderings produce semantically identical `TextureInput` values, pick
// whichever reads better at the call site.
impl<B: Into<Vec<u8>>, F: Into<TextureFormat>, S: Into<Size>> From<(B, F, S)> for TextureInput {
    fn from((bytes, format, size): (B, F, S)) -> Self {
        Self {
            data: crate::TextureData::Bytes(bytes.into()),
            options: TextureOptions {
                size: Some(size.into()),
                format: format.into(),
                ..Default::default()
            },
        }
    }
}

impl<S: Into<Size>, F: Into<TextureFormat>> From<(S, F, Vec<u8>)> for TextureInput {
    fn from((size, format, data): (S, F, Vec<u8>)) -> Self {
        Self {
            data: crate::TextureData::Bytes(data),
            options: TextureOptions {
                size: Some(size.into()),
                format: format.into(),
                ..Default::default()
            },
        }
    }
}

impl<S: Into<Size>, F: Into<TextureFormat>> From<(S, F, &[u8])> for TextureInput {
    fn from((size, format, data): (S, F, &[u8])) -> Self {
        Self {
            data: crate::TextureData::Bytes(data.to_vec()),
            options: TextureOptions {
                size: Some(size.into()),
                format: format.into(),
                ..Default::default()
            },
        }
    }
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass)]
#[cfg_attr(mobile, derive(uniffi::Record))]
#[derive(Debug, Clone)]
pub struct TextureOptions {
    pub size: Option<Size>,
    pub format: TextureFormat,
    pub sampler: SamplerOptions,
    /// Generate a full mipmap chain at upload for source images. Default: true.
    /// Combined with the default linear sampler, this enables trilinear filtering
    /// so textured surfaces stay smooth at any zoom or rotation. Set to false
    /// to skip the CPU work for textures that won't be sampled at distance.
    pub mipmaps: bool,
    /// Optional `wgpu::TextureUsages` override, stored as the underlying
    /// `u32` bit mask so it crosses every FFI cleanly. `None` lets the
    /// renderer pick per-method defaults:
    /// - `create_texture` → `TEXTURE_BINDING | COPY_DST`
    /// - `create_storage_texture` → `STORAGE_BINDING | TEXTURE_BINDING | COPY_SRC | COPY_DST`
    /// Set with `.with_usage(wgpu::TextureUsages::STORAGE_BINDING | ...)` for
    /// readability on the Rust side.
    pub usage: Option<u32>,
}

impl TextureOptions {
    /// Builder-style helper that stores the typed `wgpu::TextureUsages` as
    /// raw bits — keeps Rust call sites readable while preserving the
    /// FFI-friendly `u32` field type.
    pub fn with_usage(mut self, usage: wgpu::TextureUsages) -> Self {
        self.usage = Some(usage.bits());
        self
    }
}

impl Default for TextureOptions {
    fn default() -> Self {
        Self {
            size: None,
            format: TextureFormat::default(),
            sampler: SamplerOptions::default(),
            mipmaps: true,
            usage: None,
        }
    }
}

impl From<crate::Size> for TextureOptions {
    fn from(size: crate::Size) -> Self {
        TextureOptions {
            size: Some(size),
            ..Default::default()
        }
    }
}

impl From<&crate::Size> for TextureOptions {
    fn from(size: &crate::Size) -> Self {
        TextureOptions {
            size: Some(*size),
            ..Default::default()
        }
    }
}

impl From<TextureFormat> for TextureOptions {
    fn from(format: TextureFormat) -> Self {
        TextureOptions {
            format,
            ..Default::default()
        }
    }
}

impl From<&TextureFormat> for TextureOptions {
    fn from(format: &TextureFormat) -> Self {
        TextureOptions {
            format: *format,
            ..Default::default()
        }
    }
}

// Size-shorthands. Rust's `Into` does not chain (`[u32; 2] → Size → TextureOptions`
// requires two hops the compiler won't take on its own), so we provide direct
// impls for every shape that already converts into `Size`. Keeps
// `renderer.create_texture((bytes, [w, h]))` and `(bytes, (w, h))` working
// through the `(input, impl Into<TextureOptions>)` From impls on `TextureInput`.
impl From<[u32; 2]> for TextureOptions {
    fn from(arr: [u32; 2]) -> Self {
        TextureOptions::from(crate::Size::from(arr))
    }
}

impl From<[u32; 3]> for TextureOptions {
    fn from(arr: [u32; 3]) -> Self {
        TextureOptions::from(crate::Size::from(arr))
    }
}

impl From<(u32, u32)> for TextureOptions {
    fn from(t: (u32, u32)) -> Self {
        TextureOptions::from(crate::Size::from(t))
    }
}

impl From<(u32, u32, u32)> for TextureOptions {
    fn from(t: (u32, u32, u32)) -> Self {
        TextureOptions::from(crate::Size::from(t))
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
        let d = SamplerOptions::default();
        assert_eq!(o1.sampler.repeat_x, d.repeat_x);
        assert_eq!(o1.sampler.repeat_y, d.repeat_y);
        assert_eq!(o1.sampler.smooth, d.smooth);
        assert!(o1.sampler.compare.is_none());

        let s2: Size = [5u32, 6u32, 7u32].into();
        let o2 = TextureOptions::from(&s2);
        assert_eq!(o2.size, Some(s2));
        assert_eq!(o2.format, TextureFormat::default());
        let d = SamplerOptions::default();
        assert_eq!(o2.sampler.repeat_x, d.repeat_x);
        assert_eq!(o2.sampler.repeat_y, d.repeat_y);
        assert_eq!(o2.sampler.smooth, d.smooth);
        assert!(o2.sampler.compare.is_none());
    }

    #[test]
    fn from_format_and_refs_clear_size() {
        let fmt = TextureFormat::default();
        let o1 = TextureOptions::from(fmt);
        assert_eq!(o1.size, None);
        assert_eq!(o1.format, fmt);
        let d = SamplerOptions::default();
        assert_eq!(o1.sampler.repeat_x, d.repeat_x);
        assert_eq!(o1.sampler.repeat_y, d.repeat_y);
        assert_eq!(o1.sampler.smooth, d.smooth);
        assert!(o1.sampler.compare.is_none());

        let fmt2 = TextureFormat::default();
        let o2 = TextureOptions::from(&fmt2);
        assert_eq!(o2.size, None);
        assert_eq!(o2.format, fmt2);
        let d = SamplerOptions::default();
        assert_eq!(o2.sampler.repeat_x, d.repeat_x);
        assert_eq!(o2.sampler.repeat_y, d.repeat_y);
        assert_eq!(o2.sampler.smooth, d.smooth);
        assert!(o2.sampler.compare.is_none());
    }
}
