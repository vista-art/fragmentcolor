//! TextureRegion — internal transport type for `Texture::write_region(...)`.
//!
//! Describes where in a texture data should be written and (optionally) how
//! the source bytes are laid out. Public methods accept `impl Into<TextureRegion>`
//! so JS / Python callers pass raw arrays or objects, never the type itself.

/// Where in a texture to write, and how source bytes are laid out.
///
/// `origin` and `size` are 3D — `[x, y, z]` and `[width, height, depth]` —
/// so the same struct covers 2D textures (`z = 0`, `depth = 1`), texture arrays,
/// and 3D textures.
///
/// `bytes_per_row` and `rows_per_image` are advanced source-data layout overrides
/// used when the input has row padding (e.g. a buffer copied back from the GPU).
/// Leave them `None` for tightly-packed input.
#[derive(Copy, Clone, Debug)]
pub struct TextureRegion {
    pub origin: [u32; 3],
    pub size: [u32; 3],
    pub bytes_per_row: Option<u32>,
    pub rows_per_image: Option<u32>,
}

/// Mobile-only flat version of [`TextureRegion`] for uniffi FFI.
///
/// Uniffi cannot marshal fixed-size arrays (`[u32; 3]`), so the mobile
/// binding exposes the 3D origin and size as individual `u32` fields.
/// Use `origin_x/y/z = 0` and `size_width/height/depth = 0` (the defaults)
/// to target the whole texture; the write path interprets zeros as
/// "infer the full extent from the texture dimensions".
///
/// Swift and Kotlin see it as `TextureRegion`, the name every other binding uses.
#[cfg(mobile)]
#[derive(Debug, Clone, uniffi::Record)]
#[uniffi(name = "TextureRegion")]
pub struct TextureRegionMobile {
    pub origin_x: u32,
    pub origin_y: u32,
    pub origin_z: u32,
    /// Width of the region to write. `0` → infer (full texture width).
    pub size_width: u32,
    /// Height of the region to write. `0` → infer (full texture height).
    pub size_height: u32,
    /// Depth of the region to write. `0` → infer (full texture depth, usually 1).
    pub size_depth: u32,
    pub bytes_per_row: Option<u32>,
    pub rows_per_image: Option<u32>,
}

#[cfg(mobile)]
impl Default for TextureRegionMobile {
    /// "Whole texture" sentinel — all zeros tells the write path to infer the
    /// full extent. Equivalent to `TextureRegion::default()` on the Rust side.
    fn default() -> Self {
        Self {
            origin_x: 0,
            origin_y: 0,
            origin_z: 0,
            size_width: 0,
            size_height: 0,
            size_depth: 0,
            bytes_per_row: None,
            rows_per_image: None,
        }
    }
}

#[cfg(mobile)]
impl From<TextureRegionMobile> for TextureRegion {
    fn from(m: TextureRegionMobile) -> Self {
        // Zeros are preserved — the write path interprets [0,0,0] size as
        // "infer the full extent from the texture dimensions", matching
        // TextureRegion::default() semantics.
        TextureRegion {
            origin: [m.origin_x, m.origin_y, m.origin_z],
            size: [m.size_width, m.size_height, m.size_depth],
            bytes_per_row: m.bytes_per_row,
            rows_per_image: m.rows_per_image,
        }
    }
}

impl Default for TextureRegion {
    /// "Whole texture" sentinel — `size` of `[0, 0, 0]` is interpreted by the
    /// write path as "infer the full extent from the texture itself".
    fn default() -> Self {
        Self {
            origin: [0, 0, 0],
            size: [0, 0, 0],
            bytes_per_row: None,
            rows_per_image: None,
        }
    }
}

impl TextureRegion {
    /// Construct a region from explicit 3D origin and size.
    pub fn new(origin: [u32; 3], size: [u32; 3]) -> Self {
        Self {
            origin,
            size,
            bytes_per_row: None,
            rows_per_image: None,
        }
    }

    /// Set `bytes_per_row` for source data with row padding.
    /// It must be at least the region width times the bytes per pixel.
    /// When unset, rows are tightly packed unless the data is exactly the size
    /// of rows padded to 256 bytes, in which case they read at the padded stride.
    pub fn with_stride(mut self, bytes_per_row: u32) -> Self {
        self.bytes_per_row = Some(bytes_per_row);
        self
    }

    /// Set `rows_per_image`. Defaults to `size[1]` (the region height) when unset;
    /// override when uploading to layered or 3D textures with non-default packing.
    pub fn with_rows(mut self, rows_per_image: u32) -> Self {
        self.rows_per_image = Some(rows_per_image);
        self
    }
}

// -------------------------------------------
// Trait-based conversions (preferred surface)
// -------------------------------------------

// Size only: [w, h] — origin (0,0,0), depth 1.
impl From<[u32; 2]> for TextureRegion {
    #[inline]
    fn from(s: [u32; 2]) -> Self {
        Self::new([0, 0, 0], [s[0], s[1], 1])
    }
}
impl From<(u32, u32)> for TextureRegion {
    #[inline]
    fn from(s: (u32, u32)) -> Self {
        Self::new([0, 0, 0], [s.0, s.1, 1])
    }
}

// 2D rectangle: [x, y, w, h] — z=0, depth=1.
impl From<[u32; 4]> for TextureRegion {
    #[inline]
    fn from(a: [u32; 4]) -> Self {
        Self::new([a[0], a[1], 0], [a[2], a[3], 1])
    }
}
impl From<(u32, u32, u32, u32)> for TextureRegion {
    #[inline]
    fn from(t: (u32, u32, u32, u32)) -> Self {
        Self::new([t.0, t.1, 0], [t.2, t.3, 1])
    }
}

// 3D box: [x, y, z, w, h, d].
impl From<[u32; 6]> for TextureRegion {
    #[inline]
    fn from(a: [u32; 6]) -> Self {
        Self::new([a[0], a[1], a[2]], [a[3], a[4], a[5]])
    }
}
impl From<(u32, u32, u32, u32, u32, u32)> for TextureRegion {
    #[inline]
    fn from(t: (u32, u32, u32, u32, u32, u32)) -> Self {
        Self::new([t.0, t.1, t.2], [t.3, t.4, t.5])
    }
}

// Bridge from ScreenRegion (2D pixel rect → 2D texture region, z=0, depth=1).
impl From<crate::ScreenRegion> for TextureRegion {
    #[inline]
    fn from(r: crate::ScreenRegion) -> Self {
        Self::new([r.min_x, r.min_y, 0], [r.width(), r.height(), 1])
    }
}

// -------------------------------------------
// JS conversion (#[cfg(wasm)])
// -------------------------------------------

#[cfg(wasm)]
use crate::texture::TextureError;

#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for TextureRegion {
    type Error = TextureError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::{Array, ArrayBuffer, Reflect};

        // undefined / null → whole-texture default
        if value.is_undefined() || value.is_null() {
            return Ok(Self::default());
        }

        let dimension = |item: &wasm_bindgen::JsValue| {
            item.as_f64()
                .and_then(crate::size::read_dimension)
                .ok_or_else(|| {
                    TextureError::InvalidInput(format!(
                        "a region takes numbers from 0 up, got {item:?}"
                    ))
                })
        };

        // Plain or typed array: [w,h] | [x,y,w,h] | [x,y,z,w,h,d]
        if Array::is_array(value) || ArrayBuffer::is_view(value) {
            let numbers = Array::from(value)
                .iter()
                .map(|item| dimension(&item))
                .collect::<Result<Vec<u32>, TextureError>>()?;
            return match numbers[..] {
                [w, h] => Ok([w, h].into()),
                [x, y, w, h] => Ok([x, y, w, h].into()),
                [x, y, z, w, h, d] => Ok([x, y, z, w, h, d].into()),
                _ => Err(TextureError::InvalidInput(format!(
                    "TextureRegion array must have length 2, 4, or 6, got {}",
                    numbers.len()
                ))),
            };
        }

        // Object: { x?, y?, z?, width?, height?, depth?, bytesPerRow?, rowsPerImage? }
        // or       { minX, minY, maxX, maxY, minZ?, maxZ?, bytesPerRow?, rowsPerImage? }
        if value.is_object() {
            let read = |names: &[&str]| -> Result<Option<u32>, TextureError> {
                for name in names {
                    let field = Reflect::get(value, &wasm_bindgen::JsValue::from_str(name))
                        .unwrap_or(wasm_bindgen::JsValue::UNDEFINED);
                    if !field.is_undefined() && !field.is_null() {
                        return dimension(&field).map(Some);
                    }
                }
                Ok(None)
            };

            // min/max variant
            let min_x = read(&["minX", "min_x"])?;
            let max_x = read(&["maxX", "max_x"])?;
            if min_x.is_some() || max_x.is_some() {
                let min_x = min_x.unwrap_or(0);
                let min_y = read(&["minY", "min_y"])?.unwrap_or(0);
                let max_x = max_x.unwrap_or(min_x);
                let max_y = read(&["maxY", "max_y"])?.unwrap_or(min_y);
                let min_z = read(&["minZ", "min_z"])?.unwrap_or(0);
                let max_z = read(&["maxZ", "max_z"])?.unwrap_or(min_z + 1);
                let mut tr = Self::new(
                    [min_x, min_y, min_z],
                    [
                        max_x.saturating_sub(min_x),
                        max_y.saturating_sub(min_y),
                        max_z.saturating_sub(min_z).max(1),
                    ],
                );
                tr.bytes_per_row = read(&["bytesPerRow", "bytes_per_row"])?;
                tr.rows_per_image = read(&["rowsPerImage", "rows_per_image"])?;
                return Ok(tr);
            }

            // x/y/z + width/height/depth variant
            let x = read(&["x"])?.unwrap_or(0);
            let y = read(&["y"])?.unwrap_or(0);
            let z = read(&["z"])?.unwrap_or(0);
            let w = read(&["width"])?.unwrap_or(0);
            let h = read(&["height"])?.unwrap_or(0);
            let d = read(&["depth"])?.unwrap_or(1);
            let mut tr = Self::new([x, y, z], [w, h, d]);
            tr.bytes_per_row = read(&["bytesPerRow", "bytes_per_row"])?;
            tr.rows_per_image = read(&["rowsPerImage", "rows_per_image"])?;
            return Ok(tr);
        }

        Err(TextureError::Error(
            "Cannot convert JavaScript value to TextureRegion (expected [w,h], [x,y,w,h], [x,y,z,w,h,d], or {x,y,width,height,...})"
                .into(),
        ))
    }
}
