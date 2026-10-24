//! Public mirror of `wgpu::TextureUsages`.
//!
//! Keeps the underlying bit layout identical to wgpu so conversion in either
//! direction is a single `from_bits_retain` / `bits()` call. Stored as a
//! plain `u32` field so the type crosses every FFI cleanly.
//!
//! Use the named constants and `|` to compose:
//!
//! ```rust
//! use fragmentcolor::TextureUsage;
//!
//! let usage = TextureUsage::STORAGE_BINDING
//!     | TextureUsage::TEXTURE_BINDING
//!     | TextureUsage::COPY_SRC
//!     | TextureUsage::COPY_DST;
//! # let _ = usage;
//! ```

use std::ops::{BitOr, BitOrAssign};

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(
    python,
    pyo3::pyclass(from_py_object, get_all, eq, hash, frozen, module = "fragmentcolor")
)]
#[cfg_attr(mobile, derive(uniffi::Record))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextureUsage {
    pub bits: u32,
}

impl TextureUsage {
    /// Texture can be the source of a copy operation (e.g. `copy_texture_to_buffer`).
    pub const COPY_SRC: TextureUsage = TextureUsage { bits: 1 << 0 };
    /// Texture can be the destination of a copy operation (e.g. `copy_buffer_to_texture`).
    pub const COPY_DST: TextureUsage = TextureUsage { bits: 1 << 1 };
    /// Texture can be bound to a shader for sampling (`textureSample*`) or non-filtered fetch
    /// (`textureLoad`).
    pub const TEXTURE_BINDING: TextureUsage = TextureUsage { bits: 1 << 2 };
    /// Texture can be bound as a storage texture (`textureStore`, `textureLoad` from
    /// `texture_storage_*`).
    pub const STORAGE_BINDING: TextureUsage = TextureUsage { bits: 1 << 3 };
    /// Texture can be used as a render-pass attachment (color or depth/stencil).
    pub const RENDER_ATTACHMENT: TextureUsage = TextureUsage { bits: 1 << 4 };

    pub const fn empty() -> Self {
        TextureUsage { bits: 0 }
    }

    pub const fn contains(self, other: TextureUsage) -> bool {
        (self.bits & other.bits) == other.bits
    }

    pub const fn raw_bits(self) -> u32 {
        self.bits
    }
}

// The named flags as plain numbers, like WebGPU's `GPUTextureUsage`, so they
// combine with `|` and pass straight to the `usage` options.
#[cfg(wasm)]
#[wasm_bindgen]
impl TextureUsage {
    #[doc(hidden)]
    #[wasm_bindgen(getter = COPY_SRC)]
    pub fn copy_src_js() -> u32 {
        TextureUsage::COPY_SRC.bits
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = COPY_DST)]
    pub fn copy_dst_js() -> u32 {
        TextureUsage::COPY_DST.bits
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = TEXTURE_BINDING)]
    pub fn texture_binding_js() -> u32 {
        TextureUsage::TEXTURE_BINDING.bits
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = STORAGE_BINDING)]
    pub fn storage_binding_js() -> u32 {
        TextureUsage::STORAGE_BINDING.bits
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = RENDER_ATTACHMENT)]
    pub fn render_attachment_js() -> u32 {
        TextureUsage::RENDER_ATTACHMENT.bits
    }
}

// The named flags as class attributes that combine with `|`, like `enum.Flag`.
#[cfg(python)]
#[pyo3::pymethods]
impl TextureUsage {
    #[classattr]
    #[pyo3(name = "COPY_SRC")]
    fn copy_src_py() -> TextureUsage {
        TextureUsage::COPY_SRC
    }

    #[classattr]
    #[pyo3(name = "COPY_DST")]
    fn copy_dst_py() -> TextureUsage {
        TextureUsage::COPY_DST
    }

    #[classattr]
    #[pyo3(name = "TEXTURE_BINDING")]
    fn texture_binding_py() -> TextureUsage {
        TextureUsage::TEXTURE_BINDING
    }

    #[classattr]
    #[pyo3(name = "STORAGE_BINDING")]
    fn storage_binding_py() -> TextureUsage {
        TextureUsage::STORAGE_BINDING
    }

    #[classattr]
    #[pyo3(name = "RENDER_ATTACHMENT")]
    fn render_attachment_py() -> TextureUsage {
        TextureUsage::RENDER_ATTACHMENT
    }

    fn __or__(&self, other: TextureUsage) -> TextureUsage {
        *self | other
    }

    fn __int__(&self) -> u32 {
        self.bits
    }
}

impl BitOr for TextureUsage {
    type Output = TextureUsage;
    fn bitor(self, rhs: TextureUsage) -> TextureUsage {
        TextureUsage {
            bits: self.bits | rhs.bits,
        }
    }
}

impl BitOrAssign for TextureUsage {
    fn bitor_assign(&mut self, rhs: TextureUsage) {
        self.bits |= rhs.bits;
    }
}

impl From<TextureUsage> for wgpu::TextureUsages {
    fn from(usage: TextureUsage) -> wgpu::TextureUsages {
        wgpu::TextureUsages::from_bits_retain(usage.bits)
    }
}

impl From<wgpu::TextureUsages> for TextureUsage {
    fn from(usage: wgpu::TextureUsages) -> TextureUsage {
        TextureUsage { bits: usage.bits() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composes_with_bit_or_and_contains() {
        let u =
            TextureUsage::STORAGE_BINDING | TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_SRC;
        assert!(u.contains(TextureUsage::STORAGE_BINDING));
        assert!(u.contains(TextureUsage::TEXTURE_BINDING));
        assert!(u.contains(TextureUsage::COPY_SRC));
        assert!(!u.contains(TextureUsage::COPY_DST));
    }

    #[test]
    fn round_trips_through_wgpu_texture_usages() {
        let cases = [
            TextureUsage::COPY_SRC,
            TextureUsage::COPY_DST,
            TextureUsage::TEXTURE_BINDING,
            TextureUsage::STORAGE_BINDING,
            TextureUsage::RENDER_ATTACHMENT,
            TextureUsage::STORAGE_BINDING | TextureUsage::COPY_SRC,
        ];
        for u in cases.iter().copied() {
            let w: wgpu::TextureUsages = u.into();
            let back: TextureUsage = w.into();
            assert_eq!(u, back);
        }
    }
}
