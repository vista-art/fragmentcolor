//! TextureRegion, the internal transport type for `Texture::write_region(...)`.
//!
//! Describes where in a texture data should be written and (optionally) how
//! the source bytes are laid out. `Texture::write_region` takes anything that converts
//! into a `TextureRegion`, so JS / Python callers pass raw arrays or objects, never the type itself.

/// Where in a texture to write, and how source bytes are laid out.
///
/// `origin` and `size` are 3D, `[x, y, z]` and `[width, height, depth]`,
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

// Size only: [w, h], at origin (0, 0, 0) and depth 1.
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

// 2D rectangle: [x, y, w, h], at z = 0 and depth 1.
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
    type Error = crate::ScreenRegionError;

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
use crate::bindings::InputError;
#[cfg(any(wasm, python, test))]
use crate::bindings::Key;

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

/// Reads a region from a JS object or a Python dict through `read`, for `call`.
///
/// `read` returns the number under a key, or `None` for a missing key.
/// The form `{ x, y, z?, width, height, depth? }` needs its width and height.
/// The form `{ minX, minY, maxX, maxY, minZ?, maxZ? }` needs each max past its min.
/// Both take `bytesPerRow` and `rowsPerImage`, are one layer deep without a depth, and name `call` in each error.
#[cfg(any(wasm, python, test))]
pub(crate) fn read_region<E: From<InputError>>(
    read: impl Fn(Key) -> Result<Option<u32>, E>,
    call: &str,
) -> Result<TextureRegion, E> {
    let triple = |keys: [Key; 3]| -> Result<[Option<u32>; 3], E> {
        Ok([read(keys[0])?, read(keys[1])?, read(keys[2])?])
    };
    let (min, max) = (triple(MIN)?, triple(MAX)?);
    let (origin, extent) = (triple(ORIGIN)?, triple(EXTENT)?);
    let given = |values: &[Option<u32>]| values.iter().any(Option::is_some);
    let bounds = given(&min) || given(&max);
    if bounds && (given(&origin) || given(&extent)) {
        return Err(InputError(format!(
            "{call}: a region takes x, y, width and height, or min and max bounds, not both"
        ))
        .into());
    }
    let mut region = match bounds {
        true => from_bounds(min.map(|value| value.unwrap_or(0)), max, call)?,
        false => from_box(origin.map(|value| value.unwrap_or(0)), extent, call)?,
    };
    region.bytes_per_row = read(("bytesPerRow", "bytes_per_row"))?;
    region.rows_per_image = read(("rowsPerImage", "rows_per_image"))?;
    Ok(region)
}

/// A region from its bounds, one layer deep without a z max.
/// The x and y bounds each need a max past the min, and so does a z max given.
#[cfg(any(wasm, python, test))]
fn from_bounds(
    min: [u32; 3],
    max: [Option<u32>; 3],
    call: &str,
) -> Result<TextureRegion, InputError> {
    let mut size = [1; 3];
    for (axis, name) in AXES.iter().enumerate() {
        size[axis] = match max[axis] {
            None if axis == 2 => 1,
            None => {
                return Err(InputError(format!(
                    "{call}: a region with bounds needs its max {name}"
                )));
            }
            Some(end) if end > min[axis] => end - min[axis],
            Some(end) => {
                return Err(InputError(format!(
                    "{call}: max {name} ({end}) must be greater than min {name} ({})",
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
fn from_box(
    origin: [u32; 3],
    extent: [Option<u32>; 3],
    call: &str,
) -> Result<TextureRegion, InputError> {
    let [Some(width), Some(height), depth] = extent else {
        return Err(InputError(format!(
            "{call}: a region needs its width and height, or min and max bounds"
        )));
    };
    Ok(TextureRegion::new(
        origin,
        [width, height, depth.unwrap_or(1)],
    ))
}

// -------------------------------------------
// JS conversion (#[cfg(wasm)])
// -------------------------------------------

/// Reads a write region from JavaScript. A value of another type, or an item that isn't a
/// number, is a `TypeError`, as Python raises, and a negative or fractional number is an
/// `InvalidInputError`.
#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for TextureRegion {
    type Error = crate::errors::WebError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use crate::bindings::web::{describe, read_class, read_count, read_counts};
        use crate::errors::WebError;
        use js_sys::Reflect;

        const CALL: &str = "Texture.writeRegion";

        // Plain or typed array: [w,h] | [x,y,w,h] | [x,y,z,w,h,d]
        if let Some(numbers) = read_counts(value, CALL)? {
            return match numbers[..] {
                [w, h] => Ok([w, h].into()),
                [x, y, w, h] => Ok([x, y, w, h].into()),
                [x, y, z, w, h, d] => Ok([x, y, z, w, h, d].into()),
                _ => Err(InputError(format!(
                    "{CALL}: a region takes 2, 4 or 6 numbers, got {}",
                    numbers.len()
                ))
                .into()),
            };
        }

        // Object: { x?, y?, z?, width, height, depth?, bytesPerRow?, rowsPerImage? }
        // or       { minX?, minY?, maxX, maxY, minZ?, maxZ?, bytesPerRow?, rowsPerImage? }
        // A `ScreenRegion`, known by its brand, reads by its `minX` to `maxY` getters,
        // and its `width()` and `height()` methods are no keys. A function anywhere else throws,
        // and so does any other FragmentColor class.
        let class = read_class(value);
        let screen = class.as_deref() == Some(<crate::ScreenRegion as crate::FcKind>::KIND);
        if value.is_object() && (class.is_none() || screen) {
            let read = |(camel, snake): Key| -> Result<Option<u32>, WebError> {
                for name in [camel, snake] {
                    let field = Reflect::get(value, &wasm_bindgen::JsValue::from_str(name))
                        .unwrap_or(wasm_bindgen::JsValue::UNDEFINED);
                    let method = screen && field.is_function();
                    if !field.is_undefined() && !field.is_null() && !method {
                        return read_count(&field, &format!("{CALL}: {name}")).map(Some);
                    }
                }
                Ok(None)
            };
            return read_region(read, CALL);
        }

        let kind = class.unwrap_or_else(|| describe(value));
        Err(WebError::type_error(&format!(
            "{CALL}: a region is [w, h], [x, y, w, h], [x, y, z, w, h, d], a ScreenRegion or an object such as {{ x, y, width, height }}, got {kind}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Reads `region` from the pairs of a map, the way a JS object or a Python dict reads.
    fn read(pairs: &[(&str, u32)]) -> Result<TextureRegion, InputError> {
        let map: HashMap<&str, u32> = pairs.iter().copied().collect();
        let read = |(camel, snake): Key| Ok(map.get(camel).or_else(|| map.get(snake)).copied());
        read_region(read, "Texture.write_region")
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
    // forms at once and a z that overflows are refused.
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
            let message = read(pairs).err().map(|error| error.to_string());
            assert!(
                message.is_some_and(|message| message.starts_with("Texture.write_region: ")),
                "{pairs:?} should be refused with an error that names the call"
            );
        }
        let edge = read(&[("maxX", 1), ("maxY", 1), ("minZ", u32::MAX)]).expect("one layer");
        assert_eq!(edge.size[2], 1, "a missing maxZ adds nothing to minZ");
    }
}
