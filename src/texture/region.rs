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
#[cfg(mobile)]
#[derive(Debug, Clone, uniffi::Record)]
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

    /// Set `bytes_per_row` (must be a multiple of 256 when calling
    /// `Texture::write_region`). Use this when the source data has row padding.
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
        use js_sys::{Array, Reflect, Uint32Array};
        use wasm_bindgen::JsCast;

        // undefined / null → whole-texture default
        if value.is_undefined() || value.is_null() {
            return Ok(Self::default());
        }

        // Typed arrays (Uint32Array): support length 2, 4, 6
        if let Some(arr) = value.dyn_ref::<Uint32Array>() {
            let len = arr.length();
            return match len {
                2 => {
                    let mut buf = [0u32; 2];
                    arr.copy_to(&mut buf);
                    Ok(buf.into())
                }
                4 => {
                    let mut buf = [0u32; 4];
                    arr.copy_to(&mut buf);
                    Ok(buf.into())
                }
                6 => {
                    let mut buf = [0u32; 6];
                    arr.copy_to(&mut buf);
                    Ok(buf.into())
                }
                _ => Err(TextureError::Error(
                    "TextureRegion typed array must have length 2, 4, or 6".into(),
                )),
            };
        }

        // Plain Array: [w,h] | [x,y,w,h] | [x,y,z,w,h,d]
        if let Some(arr) = value.dyn_ref::<Array>() {
            let len = arr.length();
            let num = |i: u32| arr.get(i).as_f64().unwrap_or(0.0).max(0.0) as u32;
            return match len {
                2 => Ok([num(0), num(1)].into()),
                4 => Ok([num(0), num(1), num(2), num(3)].into()),
                6 => Ok([num(0), num(1), num(2), num(3), num(4), num(5)].into()),
                _ => Err(TextureError::Error(
                    "TextureRegion array must have length 2, 4, or 6".into(),
                )),
            };
        }

        // Object: { x?, y?, z?, width?, height?, depth?, bytesPerRow?, rowsPerImage? }
        // or       { minX, minY, maxX, maxY, minZ?, maxZ?, bytesPerRow?, rowsPerImage? }
        if value.is_object() {
            let read = |names: &[&str]| -> Option<u32> {
                for name in names {
                    if let Ok(field) = Reflect::get(value, &wasm_bindgen::JsValue::from_str(name))
                        && let Some(n) = field.as_f64()
                    {
                        return Some(n.max(0.0) as u32);
                    }
                }
                None
            };

            // min/max variant
            let has_minmax =
                read(&["minX", "min_x"]).is_some() || read(&["maxX", "max_x"]).is_some();
            if has_minmax {
                let min_x = read(&["minX", "min_x"]).unwrap_or(0);
                let min_y = read(&["minY", "min_y"]).unwrap_or(0);
                let max_x = read(&["maxX", "max_x"]).unwrap_or(min_x);
                let max_y = read(&["maxY", "max_y"]).unwrap_or(min_y);
                let min_z = read(&["minZ", "min_z"]).unwrap_or(0);
                let max_z = read(&["maxZ", "max_z"]).unwrap_or(min_z + 1);
                let mut tr = Self::new(
                    [min_x, min_y, min_z],
                    [
                        max_x.saturating_sub(min_x),
                        max_y.saturating_sub(min_y),
                        max_z.saturating_sub(min_z).max(1),
                    ],
                );
                tr.bytes_per_row = read(&["bytesPerRow", "bytes_per_row"]);
                tr.rows_per_image = read(&["rowsPerImage", "rows_per_image"]);
                return Ok(tr);
            }

            // x/y/z + width/height/depth variant
            let x = read(&["x"]).unwrap_or(0);
            let y = read(&["y"]).unwrap_or(0);
            let z = read(&["z"]).unwrap_or(0);
            let w = read(&["width"]).unwrap_or(0);
            let h = read(&["height"]).unwrap_or(0);
            let d = read(&["depth"]).unwrap_or(1);
            let mut tr = Self::new([x, y, z], [w, h, d]);
            tr.bytes_per_row = read(&["bytesPerRow", "bytes_per_row"]);
            tr.rows_per_image = read(&["rowsPerImage", "rows_per_image"]);
            return Ok(tr);
        }

        Err(TextureError::Error(
            "Cannot convert JavaScript value to TextureRegion (expected [w,h], [x,y,w,h], [x,y,z,w,h,d], or {x,y,width,height,...})"
                .into(),
        ))
    }
}

// -------------------------------------------
// Python conversion
// -------------------------------------------

#[cfg(python)]
pub(crate) fn py_to_texture_region<'py>(
    any: &pyo3::Bound<'py, pyo3::PyAny>,
) -> pyo3::PyResult<TextureRegion> {
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyList, PySequence, PyTuple};

    fn read_dict_u32<'py>(
        dict: &pyo3::Bound<'py, PyDict>,
        names: &[&str],
    ) -> pyo3::PyResult<Option<u32>> {
        for name in names {
            if let Some(value) = dict.get_item(name)? {
                if value.is_none() {
                    return Ok(None);
                }
                return Ok(Some(value.extract::<u32>()?));
            }
        }
        Ok(None)
    }

    // `None` → whole-texture default
    if any.is_none() {
        return Ok(TextureRegion::default());
    }

    // Sequence (list / tuple) of length 2, 4, or 6
    if let Ok(seq) = any.downcast::<PySequence>() {
        let len = seq.len()?;
        let read = |i: usize| -> pyo3::PyResult<u32> { seq.get_item(i)?.extract::<u32>() };
        return match len {
            2 => Ok([read(0)?, read(1)?].into()),
            4 => Ok([read(0)?, read(1)?, read(2)?, read(3)?].into()),
            6 => Ok([read(0)?, read(1)?, read(2)?, read(3)?, read(4)?, read(5)?].into()),
            _ => Err(crate::error::PyFragmentColorError::new_err(
                "TextureRegion sequence must have length 2, 4, or 6",
            )),
        };
    }
    // Explicit tuple/list paths in case downcast<PySequence> doesn't catch them on some types.
    if any.downcast::<PyTuple>().is_ok() || any.downcast::<PyList>().is_ok() {
        // delegated above
    }

    // Dict variant
    if let Ok(dict) = any.downcast::<PyDict>() {
        // min/max variant
        let has_minmax = read_dict_u32(dict, &["min_x", "minX"])?.is_some()
            || read_dict_u32(dict, &["max_x", "maxX"])?.is_some();
        if has_minmax {
            let min_x = read_dict_u32(dict, &["min_x", "minX"])?.unwrap_or(0);
            let min_y = read_dict_u32(dict, &["min_y", "minY"])?.unwrap_or(0);
            let max_x = read_dict_u32(dict, &["max_x", "maxX"])?.unwrap_or(min_x);
            let max_y = read_dict_u32(dict, &["max_y", "maxY"])?.unwrap_or(min_y);
            let min_z = read_dict_u32(dict, &["min_z", "minZ"])?.unwrap_or(0);
            let max_z = read_dict_u32(dict, &["max_z", "maxZ"])?.unwrap_or(min_z + 1);
            let mut tr = TextureRegion::new(
                [min_x, min_y, min_z],
                [
                    max_x.saturating_sub(min_x),
                    max_y.saturating_sub(min_y),
                    max_z.saturating_sub(min_z).max(1),
                ],
            );
            tr.bytes_per_row = read_dict_u32(dict, &["bytes_per_row", "bytesPerRow"])?;
            tr.rows_per_image = read_dict_u32(dict, &["rows_per_image", "rowsPerImage"])?;
            return Ok(tr);
        }

        // x/y/z + width/height/depth variant
        let x = read_dict_u32(dict, &["x"])?.unwrap_or(0);
        let y = read_dict_u32(dict, &["y"])?.unwrap_or(0);
        let z = read_dict_u32(dict, &["z"])?.unwrap_or(0);
        let w = read_dict_u32(dict, &["width"])?.unwrap_or(0);
        let h = read_dict_u32(dict, &["height"])?.unwrap_or(0);
        let d = read_dict_u32(dict, &["depth"])?.unwrap_or(1);
        let mut tr = TextureRegion::new([x, y, z], [w, h, d]);
        tr.bytes_per_row = read_dict_u32(dict, &["bytes_per_row", "bytesPerRow"])?;
        tr.rows_per_image = read_dict_u32(dict, &["rows_per_image", "rowsPerImage"])?;
        return Ok(tr);
    }

    Err(crate::error::PyFragmentColorError::new_err(
        "Expected a TextureRegion-shaped value (sequence of length 2/4/6 or dict)",
    ))
}
