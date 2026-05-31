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
#[cfg_attr(python, pyo3::pyclass(from_py_object))]
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
