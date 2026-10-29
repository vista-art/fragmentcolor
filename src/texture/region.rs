//! TextureRegion — internal transport type for `Texture::write_region(...)`.
//!
//! Describes where in a texture data should be written and (optionally) how
//! the source bytes are laid out. `Texture::write_region` takes anything that converts
//! into a `TextureRegion`, so JS / Python callers pass raw arrays or objects, never the type itself.

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
/// Each size is at least 1, and a write without a region covers the whole texture.
///
/// Swift and Kotlin see it as `TextureRegion`, the name every other binding uses.
#[cfg(mobile)]
#[derive(Debug, Clone, uniffi::Record)]
#[uniffi(name = "TextureRegion")]
pub struct TextureRegionMobile {
    pub origin_x: u32,
    pub origin_y: u32,
    pub origin_z: u32,
    /// Width of the region to write, at least 1.
    pub size_width: u32,
    /// Height of the region to write, at least 1.
    pub size_height: u32,
    /// Depth of the region to write, at least 1.
    pub size_depth: u32,
    pub bytes_per_row: Option<u32>,
    pub rows_per_image: Option<u32>,
}

#[cfg(mobile)]
impl From<TextureRegionMobile> for TextureRegion {
    fn from(m: TextureRegionMobile) -> Self {
        TextureRegion {
            origin: [m.origin_x, m.origin_y, m.origin_z],
            size: [m.size_width, m.size_height, m.size_depth],
            bytes_per_row: m.bytes_per_row,
            rows_per_image: m.rows_per_image,
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

/// The pixels a `ScreenRegion` covers, at `z = 0` and one layer deep.
/// A region whose max lies before its min, or with a side of 0, is an error.
impl TryFrom<crate::ScreenRegion> for TextureRegion {
    type Error = crate::region::error::ScreenRegionError;

    fn try_from(region: crate::ScreenRegion) -> Result<Self, Self::Error> {
        let region = region.check()?;
        Ok(Self::new(
            [region.min_x, region.min_y, 0],
            [region.width(), region.height(), 1],
        ))
    }
}

// -------------------------------------------
// Object and dict regions (JS and Python)
// -------------------------------------------

#[cfg(any(wasm, python, test))]
use crate::texture::TextureError;

/// A region key in its camelCase and snake_case spellings.
#[cfg(any(wasm, python, test))]
pub(crate) type Key = (&'static str, &'static str);

#[cfg(any(wasm, python, test))]
const MIN: [Key; 3] = [("minX", "min_x"), ("minY", "min_y"), ("minZ", "min_z")];
#[cfg(any(wasm, python, test))]
const MAX: [Key; 3] = [("maxX", "max_x"), ("maxY", "max_y"), ("maxZ", "max_z")];
#[cfg(any(wasm, python, test))]
const ORIGIN: [Key; 3] = [("x", "x"), ("y", "y"), ("z", "z")];
#[cfg(any(wasm, python, test))]
const EXTENT: [Key; 3] = [("width", "width"), ("height", "height"), ("depth", "depth")];
#[cfg(any(wasm, python, test))]
const AXES: [&str; 3] = ["x", "y", "z"];

/// Reads a region from a JS object or a Python dict through `read`.
///
/// `read` returns the number under a key, or `None` for a missing key.
/// The form `{ x, y, z?, width, height, depth? }` needs its width and height.
/// The form `{ minX, minY, maxX, maxY, minZ?, maxZ? }` needs each max past its min,
/// and both take `bytesPerRow` and `rowsPerImage`, with one layer when they give no depth.
#[cfg(any(wasm, python, test))]
pub(crate) fn read_region<E: From<TextureError>>(
    read: impl Fn(Key) -> Result<Option<u32>, E>,
) -> Result<TextureRegion, E> {
    let triple = |keys: [Key; 3]| -> Result<[Option<u32>; 3], E> {
        Ok([read(keys[0])?, read(keys[1])?, read(keys[2])?])
    };
    let (min, max) = (triple(MIN)?, triple(MAX)?);
    let (origin, extent) = (triple(ORIGIN)?, triple(EXTENT)?);
    let given = |values: &[Option<u32>]| values.iter().any(Option::is_some);
    let bounds = given(&min) || given(&max);
    if bounds && (given(&origin) || given(&extent)) {
        return Err(TextureError::InvalidInput(
            "a region takes x, y, width and height, or min and max bounds, not both".into(),
        )
        .into());
    }
    let mut region = match bounds {
        true => from_bounds(min.map(|value| value.unwrap_or(0)), max)?,
        false => from_box(origin.map(|value| value.unwrap_or(0)), extent)?,
    };
    region.bytes_per_row = read(("bytesPerRow", "bytes_per_row"))?;
    region.rows_per_image = read(("rowsPerImage", "rows_per_image"))?;
    Ok(region)
}

/// A region from its bounds, one layer deep without a z max.
/// The x and y bounds each need a max past the min, and so does a z max given.
#[cfg(any(wasm, python, test))]
fn from_bounds(min: [u32; 3], max: [Option<u32>; 3]) -> Result<TextureRegion, TextureError> {
    let mut size = [1; 3];
    for (axis, name) in AXES.iter().enumerate() {
        size[axis] = match max[axis] {
            None if axis == 2 => 1,
            None => {
                return Err(TextureError::InvalidInput(format!(
                    "a region with bounds needs its max {name}"
                )));
            }
            Some(end) if end > min[axis] => end - min[axis],
            Some(end) => {
                return Err(TextureError::InvalidInput(format!(
                    "max {name} ({end}) must be greater than min {name} ({})",
                    min[axis]
                )));
            }
        };
    }
    Ok(TextureRegion::new(min, size))
}

/// A region from its origin and extent.
/// It needs a width and a height, and without a depth it is one layer deep.
#[cfg(any(wasm, python, test))]
fn from_box(origin: [u32; 3], extent: [Option<u32>; 3]) -> Result<TextureRegion, TextureError> {
    let [Some(width), Some(height), depth] = extent else {
        return Err(TextureError::InvalidInput(
            "a region needs its width and height, or min and max bounds".into(),
        ));
    };
    Ok(TextureRegion::new(
        origin,
        [width, height, depth.unwrap_or(1)],
    ))
}

// -------------------------------------------
// JS conversion (#[cfg(wasm)])
// -------------------------------------------

#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for TextureRegion {
    type Error = TextureError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use crate::size::read_dimension;
        use js_sys::{Array, ArrayBuffer, Reflect};

        let dimension = |item: &wasm_bindgen::JsValue| {
            item.as_f64().and_then(read_dimension).ok_or_else(|| {
                TextureError::InvalidInput(format!(
                    "a region takes whole numbers from 0 up, got {item:?}"
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

        // Object: { x?, y?, z?, width, height, depth?, bytesPerRow?, rowsPerImage? }
        // or       { minX?, minY?, maxX, maxY, minZ?, maxZ?, bytesPerRow?, rowsPerImage? }
        // A `ScreenRegion`, known by its brand, reads by its `minX` to `maxY` getters,
        // and its `width()` and `height()` methods are no keys. A function anywhere else throws.
        if value.is_object() {
            let brand = Reflect::get(value, &wasm_bindgen::JsValue::from_str("__fc_kind"))
                .ok()
                .and_then(|brand| brand.as_string());
            let screen = brand.as_deref() == Some(<crate::ScreenRegion as crate::FcKind>::KIND);
            let read = |(camel, snake): Key| -> Result<Option<u32>, TextureError> {
                for name in [camel, snake] {
                    let field = Reflect::get(value, &wasm_bindgen::JsValue::from_str(name))
                        .unwrap_or(wasm_bindgen::JsValue::UNDEFINED);
                    let method = screen && field.is_function();
                    if !field.is_undefined() && !field.is_null() && !method {
                        return dimension(&field).map(Some);
                    }
                }
                Ok(None)
            };
            return read_region(read);
        }

        Err(TextureError::Error(
            "Cannot convert JavaScript value to TextureRegion (expected [w,h], [x,y,w,h], [x,y,z,w,h,d], or {x,y,width,height,...})"
                .into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Reads `region` from the pairs of a map, the way a JS object or a Python dict reads.
    fn read(pairs: &[(&str, u32)]) -> Result<TextureRegion, TextureError> {
        let map: HashMap<&str, u32> = pairs.iter().copied().collect();
        read_region(|(camel, snake): Key| Ok(map.get(camel).or_else(|| map.get(snake)).copied()))
    }

    // Story: both object forms read the region they spell, with one layer when
    // they leave out the depth, and with the stride and rows they carry.
    #[test]
    fn read_region_reads_both_forms() {
        let boxed = read(&[("x", 1), ("y", 2), ("width", 3), ("height", 4)]).expect("box");
        assert_eq!((boxed.origin, boxed.size), ([1, 2, 0], [3, 4, 1]));
        let bounds =
            read(&[("minX", 8), ("maxX", 12), ("min_y", 0), ("max_y", 4)]).expect("bounds");
        assert_eq!((bounds.origin, bounds.size), ([8, 0, 0], [4, 4, 1]));
        let deep = read(&[("maxX", 2), ("maxY", 2), ("minZ", u32::MAX - 1)]).expect("deep");
        assert_eq!((deep.origin[2], deep.size[2]), (u32::MAX - 1, 1));
        let layout = read(&[("width", 4), ("height", 4), ("bytesPerRow", 256)]).expect("stride");
        assert_eq!(layout.bytes_per_row, Some(256));
    }

    // Story: reversed or empty bounds, a missing extent, misspelled keys, both
    // forms at once and a z that overflows are refused, where they turned into a
    // write of a different region or a panic.
    #[test]
    fn read_region_refuses_what_it_cannot_read() {
        let cases: [&[(&str, u32)]; 7] = [
            &[("minX", 10), ("maxX", 2), ("minY", 0), ("maxY", 4)],
            &[
                ("minX", 0),
                ("maxX", 4),
                ("minY", 0),
                ("maxY", 4),
                ("minZ", 2),
                ("maxZ", 2),
            ],
            &[("minX", 0), ("maxX", 4)],
            &[("minY", 0), ("maxY", 4)],
            &[("x", 0), ("y", 0), ("w", 4), ("h", 4)],
            &[("x", 0), ("width", 4), ("height", 4), ("maxX", 4)],
            &[],
        ];
        for pairs in cases {
            assert!(
                matches!(read(pairs), Err(TextureError::InvalidInput(_))),
                "{pairs:?} should be refused"
            );
        }
        let edge = read(&[("maxX", 1), ("maxY", 1), ("minZ", u32::MAX)]).expect("one layer");
        assert_eq!(edge.size[2], 1, "a missing maxZ adds nothing to minZ");
    }
}
