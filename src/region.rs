#[cfg(wasm)]
use crate::error::WebError;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

pub mod error;
#[cfg(wasm)]
use crate::size::read_dimension;
use error::ScreenRegionError;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(
    python,
    pyo3::pyclass(from_py_object, get_all, eq, hash, frozen, module = "fragmentcolor")
)]
#[cfg_attr(mobile, derive(uniffi::Record))]
/// A 2D pixel-space rectangle used for viewports and scissor rects.
/// For texture upload regions see `TextureRegion` in the texture module.
///
/// `max_x` and `max_y` are exclusive, so `width()` is `max_x - min_x`.
/// A size alone, `[w, h]` or `(w, h)`, covers exactly `w` by `h` pixels from the origin.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct ScreenRegion {
    #[cfg_attr(wasm, wasm_bindgen(skip))]
    pub min_x: u32,
    #[cfg_attr(wasm, wasm_bindgen(skip))]
    pub min_y: u32,
    #[cfg_attr(wasm, wasm_bindgen(skip))]
    pub max_x: u32,
    #[cfg_attr(wasm, wasm_bindgen(skip))]
    pub max_y: u32,
}

impl Default for ScreenRegion {
    fn default() -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: 1,
            max_y: 1,
        }
    }
}

impl ScreenRegion {
    /// A region at `origin` that is `size` large. A width or height of 0 is an error.
    pub fn new(
        origin: impl Into<(u32, u32)>,
        size: impl Into<(u32, u32)>,
    ) -> Result<Self, ScreenRegionError> {
        let (x, y) = origin.into();
        let (w, h) = size.into();
        Self::from_region(x, y, w, h)
    }

    /// A region at `x`, `y` that is `width` by `height` large.
    /// A side of 0, or a far edge past `u32::MAX`, is an error.
    pub fn from_region(x: u32, y: u32, width: u32, height: u32) -> Result<Self, ScreenRegionError> {
        let overflow = ScreenRegionError::Overflow {
            x,
            y,
            width,
            height,
        };
        let (Some(max_x), Some(max_y)) = (x.checked_add(width), y.checked_add(height)) else {
            return Err(overflow);
        };
        Self {
            min_x: x,
            min_y: y,
            max_x,
            max_y,
        }
        .check()
    }

    /// A region of exactly `size` pixels at the origin, the same as [`ScreenRegion::from_size`].
    pub fn from_tuple(size: (u32, u32)) -> Result<Self, ScreenRegionError> {
        Self::from_size(size.0, size.1)
    }

    /// The region, or [`ScreenRegionError::Reversed`] when a max lies before its min,
    /// and [`ScreenRegionError::Empty`] when it has no width or no height.
    /// The fields are public, so a viewport checks the region it receives too.
    pub(crate) fn check(self) -> Result<Self, ScreenRegionError> {
        if self.max_x < self.min_x || self.max_y < self.min_y {
            return Err(ScreenRegionError::Reversed {
                min_x: self.min_x,
                min_y: self.min_y,
                max_x: self.max_x,
                max_y: self.max_y,
            });
        }
        if self.width() == 0 || self.height() == 0 {
            return Err(ScreenRegionError::Empty {
                width: self.width(),
                height: self.height(),
            });
        }
        Ok(self)
    }

    /// Reads JavaScript's `[w, h]` or `[x, y, w, h]`, each number checked by [`read_dimension`].
    #[cfg(wasm)]
    pub(crate) fn from_numbers(numbers: &[f64]) -> Result<Self, ScreenRegionError> {
        let dimensions = numbers
            .iter()
            .map(|&number| check_dimension(number))
            .collect::<Result<Vec<u32>, ScreenRegionError>>()?;
        Self::from_dimensions(&dimensions)
    }

    /// A region from the `[w, h]` or `[x, y, w, h]` a binding read.
    #[cfg(any(wasm, python, test))]
    pub(crate) fn from_dimensions(dimensions: &[u32]) -> Result<Self, ScreenRegionError> {
        match *dimensions {
            [width, height] => Self::from_size(width, height),
            [x, y, width, height] => Self::from_region(x, y, width, height),
            _ => Err(ScreenRegionError::TypeMismatch(format!(
                "a region takes [w, h] or [x, y, w, h], got {} numbers",
                dimensions.len()
            ))),
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [
            self.min_x as f32,
            self.min_y as f32,
            self.max_x as f32,
            self.max_y as f32,
        ]
    }

    /// A region at the origin that is `width` by `height` large. A side of 0 is an error.
    pub fn from_size(width: u32, height: u32) -> Result<Self, ScreenRegionError> {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: width,
            max_y: height,
        }
        .check()
    }

    /// The one pixel at `x`, `y`. A pixel at `u32::MAX` has no far edge, so it is an error.
    pub fn from_pixel(x: u32, y: u32) -> Result<Self, ScreenRegionError> {
        Self::from_region(x, y, 1, 1)
    }

    pub fn clamp(&mut self, width: u32, height: u32) {
        *self = self.clip(width, height);
    }

    /// The part of this region inside a `width` by `height` area.
    pub(crate) fn clip(&self, width: u32, height: u32) -> ScreenRegion {
        Self {
            min_x: self.min_x.min(width),
            min_y: self.min_y.min(height),
            max_x: self.max_x.min(width),
            max_y: self.max_y.min(height),
        }
    }

    pub fn union(&mut self, other: ScreenRegion) {
        self.min_x = self.min_x.min(other.min_x);
        self.min_y = self.min_y.min(other.min_y);
        self.max_x = self.max_x.max(other.max_x);
        self.max_y = self.max_y.max(other.max_y);
    }

    /// Grows the region to hold the pixel at `x`, `y`, and keeps it as it was for a pixel at `u32::MAX`.
    pub fn encompass(&mut self, x: u32, y: u32) -> Result<(), ScreenRegionError> {
        self.union(Self::from_pixel(x, y)?);
        Ok(())
    }

    pub fn intersects(&self, other: ScreenRegion) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    pub fn area(&self) -> u32 {
        self.width() * self.height()
    }

    pub fn antialias_factor(&self) -> f32 {
        2.0 / self.smaller_side() as f32
    }

    pub fn smaller_side(&self) -> u32 {
        self.width().min(self.height())
    }

    pub fn larger_side(&self) -> u32 {
        self.width().max(self.height())
    }

    pub fn is_larger_than(&self, other: ScreenRegion) -> bool {
        self.area() > other.area()
    }

    pub fn is_smaller_than(&self, other: ScreenRegion) -> bool {
        self.area() < other.area()
    }

    pub fn equals(&self, other: ScreenRegion) -> bool {
        self.min_x == other.min_x
            && self.min_y == other.min_y
            && self.max_x == other.max_x
            && self.max_y == other.max_y
    }

    /// The pixels from `min_x` up to `max_x`, and 0 for a region whose max lies before its min.
    pub fn width(&self) -> u32 {
        self.max_x.saturating_sub(self.min_x)
    }

    /// The pixels from `min_y` up to `max_y`, and 0 for a region whose max lies before its min.
    pub fn height(&self) -> u32 {
        self.max_y.saturating_sub(self.min_y)
    }

    pub fn width_f32(&self) -> f32 {
        self.width() as f32
    }

    pub fn height_f32(&self) -> f32 {
        self.height() as f32
    }

    pub fn half_width(&self) -> u32 {
        self.width() / 2
    }

    pub fn half_height(&self) -> u32 {
        self.height() / 2
    }

    pub fn half_width_f32(&self) -> f32 {
        self.width() as f32 / 2.0
    }

    pub fn half_height_f32(&self) -> f32 {
        self.height() as f32 / 2.0
    }

    pub fn outbound_radius(&self) -> f32 {
        let width = self.half_width_f32();
        let height = self.half_height_f32();
        (width * width + height * height).sqrt()
    }

    pub fn inbound_radius(&self) -> f32 {
        self.half_width_f32().min(self.half_height_f32())
    }

    pub fn aspect(&self) -> f32 {
        if self.height() == 0 {
            return 0.0;
        }
        self.width() as f32 / self.height() as f32
    }

    pub fn pixel_center(&self) -> (u32, u32) {
        (
            self.min_x + self.half_width(),
            self.min_y + self.half_height(),
        )
    }
}

/// Reads `[w, h]` or `[x, y, w, h]` from a plain or typed array, an `{ x, y, width, height }`
/// object such as a `DOMRect`, or the `{ minX, minY, maxX, maxY }` fields of a `ScreenRegion`.
#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for ScreenRegion {
    type Error = ScreenRegionError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::{Array, ArrayBuffer};

        if Array::is_array(value) || ArrayBuffer::is_view(value) {
            let numbers = Array::from(value)
                .iter()
                .map(|item| read_number(&item))
                .collect::<Result<Vec<f64>, ScreenRegionError>>()?;
            return ScreenRegion::from_numbers(&numbers);
        }

        if value.is_object() {
            let field = |names: &[&str]| -> Result<Option<u32>, ScreenRegionError> {
                for name in names {
                    let item = js_sys::Reflect::get(value, &JsValue::from_str(name))
                        .unwrap_or(JsValue::UNDEFINED);
                    if !item.is_undefined() && !item.is_null() {
                        return js_to_dimension(&item).map(Some);
                    }
                }
                Ok(None)
            };
            // Bounds first: on a `ScreenRegion`, `width` and `height` are methods.
            let bounds = (
                field(&["minX", "min_x"])?,
                field(&["minY", "min_y"])?,
                field(&["maxX", "max_x"])?,
                field(&["maxY", "max_y"])?,
            );
            if let (Some(min_x), Some(min_y), Some(max_x), Some(max_y)) = bounds {
                return ScreenRegion {
                    min_x,
                    min_y,
                    max_x,
                    max_y,
                }
                .check();
            }
            let rect = (
                field(&["x"])?,
                field(&["y"])?,
                field(&["width"])?,
                field(&["height"])?,
            );
            if let (Some(x), Some(y), Some(width), Some(height)) = rect {
                return ScreenRegion::from_region(x, y, width, height);
            }
        }

        Err(ScreenRegionError::TypeMismatch(
            "Cannot convert JavaScript value to ScreenRegion (expected [x,y,w,h], [w,h], or {x,y,width,height})".into(),
        ))
    }
}

/// One region number from JavaScript, checked by [`read_dimension`].
#[cfg(wasm)]
fn check_dimension(number: f64) -> Result<u32, ScreenRegionError> {
    read_dimension(number).ok_or_else(|| {
        ScreenRegionError::TypeMismatch(format!(
            "a region takes whole numbers from 0 up, got {number}"
        ))
    })
}

/// One region item from JavaScript, which must be a number.
#[cfg(wasm)]
fn read_number(item: &JsValue) -> Result<f64, ScreenRegionError> {
    item.as_f64().ok_or_else(|| {
        ScreenRegionError::TypeMismatch(format!("a region takes numbers, got {item:?}"))
    })
}

/// One region number from JS, checked by [`read_dimension`].
#[cfg(wasm)]
fn js_to_dimension(item: &JsValue) -> Result<u32, ScreenRegionError> {
    check_dimension(read_number(item)?)
}

/// A pair of region numbers from JS, such as an origin or a size.
#[cfg(wasm)]
fn js_to_pair(value: &JsValue) -> Result<(u32, u32), ScreenRegionError> {
    let numbers = js_sys::Array::from(value)
        .iter()
        .map(|item| js_to_dimension(&item))
        .collect::<Result<Vec<u32>, ScreenRegionError>>()?;
    match numbers[..] {
        [first, second] => Ok((first, second)),
        _ => Err(ScreenRegionError::TypeMismatch(format!(
            "expected two numbers, got {value:?}"
        ))),
    }
}

// ScreenRegion is a value type. The parity audit tracks the methods that take
// it, so its constructor and accessors have no doc pages.
#[cfg(wasm)]
#[wasm_bindgen]
impl ScreenRegion {
    #[doc(hidden)]
    #[wasm_bindgen(constructor)]
    pub fn new_js(origin: &JsValue, size: &JsValue) -> Result<ScreenRegion, WebError> {
        Ok(ScreenRegion::new(js_to_pair(origin)?, js_to_pair(size)?)?)
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = minX)]
    pub fn min_x_js(&self) -> u32 {
        self.min_x
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = minY)]
    pub fn min_y_js(&self) -> u32 {
        self.min_y
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = maxX)]
    pub fn max_x_js(&self) -> u32 {
        self.max_x
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = maxY)]
    pub fn max_y_js(&self) -> u32 {
        self.max_y
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = width)]
    pub fn width_js(&self) -> u32 {
        self.width()
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = height)]
    pub fn height_js(&self) -> u32 {
        self.height()
    }
}

#[cfg(wasm)]
crate::impl_tryfrom_owned_via_ref!(ScreenRegion, wasm_bindgen::JsValue, ScreenRegionError);

// The brand lets a reader tell a ScreenRegion from a plain object after a minifier renames the class.
crate::impl_fc_kind!(ScreenRegion, "ScreenRegion");

// -------------------------------------------
// Trait-based conversions (preferred surface)
// -------------------------------------------

impl TryFrom<(u32, u32)> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(size: (u32, u32)) -> Result<Self, Self::Error> {
        ScreenRegion::from_size(size.0, size.1)
    }
}
impl TryFrom<&(u32, u32)> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(size: &(u32, u32)) -> Result<Self, Self::Error> {
        ScreenRegion::from_size(size.0, size.1)
    }
}

impl TryFrom<[u32; 2]> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(a: [u32; 2]) -> Result<Self, Self::Error> {
        ScreenRegion::from_size(a[0], a[1])
    }
}
impl TryFrom<&[u32; 2]> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(a: &[u32; 2]) -> Result<Self, Self::Error> {
        ScreenRegion::from_size(a[0], a[1])
    }
}

impl TryFrom<(u32, u32, u32, u32)> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(t: (u32, u32, u32, u32)) -> Result<Self, Self::Error> {
        ScreenRegion::from_region(t.0, t.1, t.2, t.3)
    }
}
impl TryFrom<&(u32, u32, u32, u32)> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(t: &(u32, u32, u32, u32)) -> Result<Self, Self::Error> {
        ScreenRegion::from_region(t.0, t.1, t.2, t.3)
    }
}

impl TryFrom<[u32; 4]> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(a: [u32; 4]) -> Result<Self, Self::Error> {
        ScreenRegion::from_region(a[0], a[1], a[2], a[3])
    }
}
impl TryFrom<&[u32; 4]> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(a: &[u32; 4]) -> Result<Self, Self::Error> {
        ScreenRegion::from_region(a[0], a[1], a[2], a[3])
    }
}

// An origin and a size, as `ScreenRegion::new` and Swift's `[(x, y), (width, height)]` read them.
impl TryFrom<((u32, u32), (u32, u32))> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from((origin, size): ((u32, u32), (u32, u32))) -> Result<Self, Self::Error> {
        ScreenRegion::new(origin, size)
    }
}
impl TryFrom<&((u32, u32), (u32, u32))> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(&(origin, size): &((u32, u32), (u32, u32))) -> Result<Self, Self::Error> {
        ScreenRegion::new(origin, size)
    }
}

// Outbound conversions for convenience
impl From<&ScreenRegion> for [f32; 4] {
    #[inline]
    fn from(r: &ScreenRegion) -> Self {
        r.to_array()
    }
}

#[cfg(python)]
mod python_bindings {
    use super::*;
    use crate::platforms::python::PyUnsigned;
    use pyo3::prelude::*;

    #[pymethods]
    impl ScreenRegion {
        // ScreenRegion is a value type used as an argument to pass.setViewport()
        // and similar APIs. Its constructors are internal plumbing; the parity
        // audit tracks the parent methods, not the value type itself.
        #[doc(hidden)]
        #[new]
        pub fn new_py(
            origin: (PyUnsigned<'_>, PyUnsigned<'_>),
            size: (PyUnsigned<'_>, PyUnsigned<'_>),
        ) -> PyResult<Self> {
            let origin = (
                origin.0.read("ScreenRegion: x")?,
                origin.1.read("ScreenRegion: y")?,
            );
            let size = (
                size.0.read("ScreenRegion: width")?,
                size.1.read("ScreenRegion: height")?,
            );
            Ok(ScreenRegion::new(origin, size)?)
        }

        #[doc(hidden)]
        #[staticmethod]
        #[pyo3(name = "from_region")]
        pub fn from_region_py(
            x: PyUnsigned<'_>,
            y: PyUnsigned<'_>,
            width: PyUnsigned<'_>,
            height: PyUnsigned<'_>,
        ) -> PyResult<Self> {
            Ok(ScreenRegion::from_region(
                x.read("ScreenRegion.from_region: x")?,
                y.read("ScreenRegion.from_region: y")?,
                width.read("ScreenRegion.from_region: width")?,
                height.read("ScreenRegion.from_region: height")?,
            )?)
        }

        #[doc(hidden)]
        #[staticmethod]
        #[pyo3(name = "from_tuple")]
        pub fn from_tuple_py(size: (PyUnsigned<'_>, PyUnsigned<'_>)) -> PyResult<Self> {
            Ok(ScreenRegion::from_tuple((
                size.0.read("ScreenRegion.from_tuple: width")?,
                size.1.read("ScreenRegion.from_tuple: height")?,
            ))?)
        }

        #[doc(hidden)]
        #[pyo3(name = "width")]
        pub fn width_py(&self) -> u32 {
            self.width()
        }

        #[doc(hidden)]
        #[pyo3(name = "height")]
        pub fn height_py(&self) -> u32 {
            self.height()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ScreenRegion;

    /// Builds a region field by field, so a test can hold one with no area.
    fn build_region(x: u32, y: u32, width: u32, height: u32) -> ScreenRegion {
        ScreenRegion {
            min_x: x,
            min_y: y,
            max_x: x + width,
            max_y: y + height,
        }
    }

    // Story: a binding's region is a size or an origin and a size, and any
    // other count of numbers fails.
    #[test]
    fn reads_two_or_four_dimensions() {
        assert_eq!(
            ScreenRegion::from_dimensions(&[8, 4]).ok(),
            Some(build_region(0, 0, 8, 4))
        );
        assert_eq!(
            ScreenRegion::from_dimensions(&[1, 2, 8, 4]).ok(),
            Some(build_region(1, 2, 8, 4))
        );
        assert!(ScreenRegion::from_dimensions(&[1, 2, 3]).is_err());
        assert!(ScreenRegion::from_dimensions(&[1, 2, 0, 4]).is_err());
    }

    // Story: a region tuple reads as unsigned, so a negative number fails to
    // compile and a number past i32::MAX keeps its value.
    #[test]
    fn tuples_read_as_unsigned() {
        let region = ScreenRegion::try_from((3_000_000_000, 0, 10, 10)).ok();
        assert_eq!(region.map(|region| region.min_x), Some(3_000_000_000));
        assert_eq!(region.map(|region| region.width()), Some(10));
    }

    // Story: a region whose far edge would pass u32::MAX is an error that names
    // the size asked for, from every constructor that adds to a coordinate.
    #[test]
    fn refuses_a_far_edge_past_u32() {
        use super::error::ScreenRegionError;
        let overflow = |result: Result<ScreenRegion, ScreenRegionError>| {
            matches!(result, Err(ScreenRegionError::Overflow { .. }))
        };
        let message = ScreenRegion::from_region(u32::MAX - 2, 0, 5, 5)
            .err()
            .map(|error| error.to_string());
        assert_eq!(
            message.as_deref(),
            Some("a region at 4294967293, 0 sized 5 by 5 reaches past 4294967295")
        );
        assert!(overflow(ScreenRegion::new((u32::MAX, 0), (5, 5))));
        assert!(overflow(ScreenRegion::try_from((0, u32::MAX, 4, 4))));
        let message = ScreenRegion::try_from(((1, 0), (u32::MAX, 3)))
            .err()
            .map(|error| error.to_string());
        assert_eq!(
            message.as_deref(),
            Some("a region at 1, 0 sized 4294967295 by 3 reaches past 4294967295")
        );
        assert!(overflow(ScreenRegion::from_pixel(u32::MAX, 0)));
        let mut region = build_region(0, 0, 2, 2);
        assert!(region.encompass(u32::MAX, 0).is_err());
        assert_eq!(region, build_region(0, 0, 2, 2));
        let edge = ScreenRegion::from_region(u32::MAX - 5, 0, 5, 5).ok();
        assert_eq!(edge.map(|region| region.max_x), Some(u32::MAX));
    }

    // Story: a region with no width or no height is an error from every
    // constructor and conversion.
    #[test]
    fn refuses_a_side_of_zero() {
        use super::error::ScreenRegionError;
        let empty = |result: Result<ScreenRegion, ScreenRegionError>| {
            matches!(result, Err(ScreenRegionError::Empty { .. }))
        };
        assert!(empty(ScreenRegion::new((4, 4), (0, 8))));
        assert!(empty(ScreenRegion::from_region(4, 4, 8, 0)));
        assert!(empty(ScreenRegion::from_size(0, 0)));
        assert!(empty(ScreenRegion::from_tuple((8, 0))));
        assert!(empty(ScreenRegion::try_from((0, 8))));
        assert!(empty(ScreenRegion::try_from([8, 0])));
        assert!(empty(ScreenRegion::try_from((1, 2, 0, 4))));
        assert!(empty(ScreenRegion::try_from([1, 2, 4, 0])));
        assert!(empty(ScreenRegion::try_from(((4, 4), (0, 8)))));
        assert!(empty(build_region(3, 3, 0, 2).check()));
        assert_eq!(
            build_region(3, 3, 1, 1).check().ok(),
            Some(build_region(3, 3, 1, 1))
        );
    }

    // Story: a size alone covers exactly that many pixels, and a pair of pairs
    // is an origin and a size, as `ScreenRegion::new` reads them.
    #[test]
    fn size_forms_cover_exact_pixels() {
        assert_eq!(
            ScreenRegion::try_from([32, 32]).ok(),
            Some(build_region(0, 0, 32, 32))
        );
        assert_eq!(
            ScreenRegion::try_from((64, 32)).ok(),
            Some(build_region(0, 0, 64, 32))
        );
        assert_eq!(
            ScreenRegion::from_tuple((64, 32)).ok(),
            Some(build_region(0, 0, 64, 32))
        );
        assert_eq!(
            ScreenRegion::try_from([2, 4, 8, 6]).ok(),
            Some(build_region(2, 4, 8, 6))
        );
        assert_eq!(
            ScreenRegion::try_from(((0, 0), (32, 32))).ok(),
            Some(build_region(0, 0, 32, 32))
        );
    }

    // Story: a viewport from a pair of pairs draws at that origin with that size,
    // where the pairs read as two corners in any order and `((8, 0), (2, 4))` swapped them.
    #[test]
    fn viewport_pairs_read_origin_and_size() {
        let pass = crate::Pass::new("p");

        let set = pass.set_viewport(((8, 0), (2, 4)));

        assert!(set.is_ok(), "{set:?}");
        assert_eq!(*pass.object.viewport.read(), Some(build_region(8, 0, 2, 4)));
    }

    // Story: Construct regions using helpers and verify geometry utilities.
    #[test]
    fn computes_area_aspect_and_centers() {
        // Arrange
        let r = build_region(2, 4, 8, 6); // spans x:[2,10), y:[4,10)

        // Act / Assert
        assert_eq!(r.width(), 8);
        assert_eq!(r.height(), 6);
        assert_eq!(r.area(), 48);
        assert!((r.aspect() - (8.0 / 6.0)).abs() < 1e-6);
        assert_eq!(r.pixel_center(), (2 + 4, 4 + 3));
    }

    // Story: Union grows the region while intersect check reports overlap.
    #[test]
    fn unions_and_intersects() {
        // Arrange
        let mut a = build_region(0, 0, 4, 4);
        let b = build_region(2, 2, 4, 4);

        // Act
        let overlaps = a.intersects(b);
        a.union(b);

        // Assert
        assert!(overlaps);
        assert_eq!(a, build_region(0, 0, 6, 6));
    }

    // Story: Clamp, encompass and equals work at the boundaries; radii helpers return intuitive values.
    #[test]
    fn clamp_encompass_equals_and_radii() {
        let mut r = build_region(1, 2, 3, 4);
        r.clamp(2, 3);
        assert_eq!(r, build_region(1, 2, 1, 1)); // max clamped down

        assert!(r.encompass(5, 6).is_ok());
        assert_eq!(r.max_x, 6);
        assert_eq!(r.max_y, 7);

        let r2 = build_region(1, 2, 5, 5);
        assert!(r.equals(r2));

        let square = build_region(0, 0, 4, 4);
        assert!((square.inbound_radius() - 2.0).abs() < 1e-6);
        assert!((square.outbound_radius() - (2.0f32.hypot(2.0))).abs() < 1e-6);
    }

    // Story: Clip keeps the part of a region inside an area, and leaves reversed bounds as they are.
    #[test]
    fn clips_to_an_area() {
        let inside = build_region(2, 2, 4, 4);
        assert_eq!(inside.clip(8, 8), inside);

        let past = build_region(6, 4, 10, 10);
        assert_eq!(past.clip(8, 8), build_region(6, 4, 2, 4));

        let outside = build_region(10, 10, 4, 4);
        assert_eq!(outside.clip(8, 8).area(), 0);

        let flipped = ScreenRegion {
            min_x: 5,
            min_y: 6,
            max_x: 1,
            max_y: 2,
        };
        assert_eq!(flipped.clip(8, 8), flipped);
        assert_eq!(flipped.area(), 0);
    }

    // Story: a region whose max lies before its min is an error, never swapped back,
    // whether it comes field by field or as a viewport.
    #[test]
    fn refuses_reversed_bounds() {
        use super::error::ScreenRegionError;
        let reversed = ScreenRegion {
            min_x: 8,
            min_y: 0,
            max_x: 2,
            max_y: 4,
        };

        let checked = reversed.check();

        let Err(error) = checked else {
            panic!("expected an error, got {checked:?}");
        };
        assert!(
            matches!(error, ScreenRegionError::Reversed { .. }),
            "{error:?}"
        );
        assert_eq!(
            error.to_string(),
            "a region needs each max past its min, got x from 8 to 2 and y from 0 to 4"
        );
        assert_eq!((reversed.width(), reversed.height()), (0, 4));
        assert!(crate::Pass::new("p").set_viewport(reversed).is_err());
    }
}
