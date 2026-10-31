//! `ScreenRegion`: a rectangle of pixels on a target,
//! such as a pass viewport, and whether two regions meet.

#[cfg(wasm)]
use crate::FcKind;
#[cfg(wasm)]
use crate::bindings::web::{describe, read_class, read_list, read_number};
#[cfg(wasm)]
use crate::error::WebError;
#[cfg(wasm)]
use wasm_bindgen::JsCast;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use super::error::ScreenRegionError;
#[cfg(wasm)]
use crate::values::read_dimension;

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
    // wasm-bindgen gives each pub field an unchecked setter.
    // JavaScript reads them through the `minX`, `minY`, `maxX` and `maxY` getters.
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
        let (width, height) = size.into();
        Self::build(x, y, width, height)
    }

    /// A region at `x`, `y` that is `width` by `height` large.
    /// A side of 0, or a far edge past `u32::MAX`, is an error.
    fn build(x: u32, y: u32, width: u32, height: u32) -> Result<Self, ScreenRegionError> {
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
        Self::try_from(dimensions.as_slice())
    }

    /// A region from `[w, h]` or `[x, y, w, h]`.
    fn from_dimensions(dimensions: &[u32]) -> Result<Self, ScreenRegionError> {
        match *dimensions {
            [width, height] => Self::build(0, 0, width, height),
            [x, y, width, height] => Self::build(x, y, width, height),
            _ => Err(ScreenRegionError::Invalid(format!(
                "a region takes [w, h] or [x, y, w, h], got {} numbers",
                dimensions.len()
            ))),
        }
    }

    /// The one pixel at `x`, `y`. A pixel at `u32::MAX` has no far edge, so it is an error.
    pub fn from_pixel(x: u32, y: u32) -> Result<Self, ScreenRegionError> {
        Self::build(x, y, 1, 1)
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
/// An origin and a size as `[[x, y], [w, h]]` read as `ScreenRegion::new` reads them, as a Rust
/// `((x, y), (w, h))` does. A value of another type, another FragmentColor class among them, or an
/// item that isn't a number, is a `TypeError`, as Python raises, and a fraction, a negative number
/// or a wrong count is an `InvalidInputError`.
#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for ScreenRegion {
    type Error = WebError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        if let Some((origin, size)) = read_pairs(value) {
            let origin = js_to_pair(&origin, "a region: origin")?;
            let size = js_to_pair(&size, "a region: size")?;
            return Ok(ScreenRegion::new(origin, size)?);
        }

        if let Some(numbers) = read_list(value, "a region")? {
            return Ok(ScreenRegion::from_numbers(&numbers)?);
        }

        let class = read_class(value);
        if value.is_object() && class.as_deref().is_none_or(|class| class == Self::KIND) {
            let field = |names: &[&str]| -> Result<Option<u32>, WebError> {
                for name in names {
                    let item = js_sys::Reflect::get(value, &JsValue::from_str(name))
                        .unwrap_or(JsValue::UNDEFINED);
                    if !item.is_undefined() && !item.is_null() {
                        let number = read_number(&item, format_args!("a region's {name}"))?;
                        return Ok(Some(check_dimension(number)?));
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
                let region = ScreenRegion {
                    min_x,
                    min_y,
                    max_x,
                    max_y,
                };
                return Ok(region.check()?);
            }
            let rect = (
                field(&["x"])?,
                field(&["y"])?,
                field(&["width"])?,
                field(&["height"])?,
            );
            if let (Some(x), Some(y), Some(width), Some(height)) = rect {
                return Ok(ScreenRegion::build(x, y, width, height)?);
            }
        }

        let kind = class.unwrap_or_else(|| describe(value));
        Err(WebError::type_error(&format!(
            "a region is [w, h], [x, y, w, h], [[x, y], [w, h]], {{ x, y, width, height }} or a ScreenRegion, got {kind}"
        )))
    }
}

/// The two items of `[[x, y], [w, h]]`, an array of two arrays, or `None` for any other value.
#[cfg(wasm)]
fn read_pairs(value: &JsValue) -> Option<(JsValue, JsValue)> {
    use js_sys::Array;
    let array = value.dyn_ref::<Array>()?;
    let (origin, size) = (array.get(0), array.get(1));
    let pairs = array.length() == 2 && Array::is_array(&origin) && Array::is_array(&size);
    pairs.then_some((origin, size))
}

/// One region number from JavaScript, checked by [`read_dimension`].
#[cfg(wasm)]
fn check_dimension(number: f64) -> Result<u32, ScreenRegionError> {
    read_dimension(number).ok_or_else(|| {
        ScreenRegionError::Invalid(format!(
            "a region takes whole numbers from 0 up, got {number}"
        ))
    })
}

/// A pair of region numbers from JS, such as an origin or a size, named `label`.
/// A value that isn't a list of numbers is a `TypeError`, and any count but two an `InvalidInputError`.
#[cfg(wasm)]
fn js_to_pair(value: &JsValue, label: &str) -> Result<(u32, u32), WebError> {
    let Some(numbers) = read_list(value, label)? else {
        let kind = describe(value);
        return Err(WebError::type_error(&format!(
            "{label} takes two numbers, got {kind}"
        )));
    };
    let numbers = numbers
        .into_iter()
        .map(check_dimension)
        .collect::<Result<Vec<u32>, ScreenRegionError>>()?;
    match numbers[..] {
        [first, second] => Ok((first, second)),
        _ => Err(ScreenRegionError::Invalid(format!(
            "{label} takes two numbers, got {}",
            numbers.len()
        ))
        .into()),
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
        let origin = js_to_pair(origin, "ScreenRegion: origin")?;
        let size = js_to_pair(size, "ScreenRegion: size")?;
        Ok(ScreenRegion::new(origin, size)?)
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
crate::impl_tryfrom_owned_via_ref!(ScreenRegion, wasm_bindgen::JsValue, WebError);

// The brand lets a reader tell a ScreenRegion from a plain object after a minifier renames the class.
crate::impl_fc_kind!(ScreenRegion, "ScreenRegion");

impl TryFrom<&[u32]> for ScreenRegion {
    type Error = ScreenRegionError;

    fn try_from(dimensions: &[u32]) -> Result<Self, Self::Error> {
        ScreenRegion::from_dimensions(dimensions)
    }
}

impl TryFrom<(u32, u32)> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(size: (u32, u32)) -> Result<Self, Self::Error> {
        ScreenRegion::build(0, 0, size.0, size.1)
    }
}
impl TryFrom<&(u32, u32)> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(size: &(u32, u32)) -> Result<Self, Self::Error> {
        ScreenRegion::build(0, 0, size.0, size.1)
    }
}

impl TryFrom<[u32; 2]> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from([width, height]: [u32; 2]) -> Result<Self, Self::Error> {
        ScreenRegion::build(0, 0, width, height)
    }
}
impl TryFrom<&[u32; 2]> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(&[width, height]: &[u32; 2]) -> Result<Self, Self::Error> {
        ScreenRegion::build(0, 0, width, height)
    }
}

impl TryFrom<(u32, u32, u32, u32)> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from((x, y, width, height): (u32, u32, u32, u32)) -> Result<Self, Self::Error> {
        ScreenRegion::build(x, y, width, height)
    }
}
impl TryFrom<&(u32, u32, u32, u32)> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(&(x, y, width, height): &(u32, u32, u32, u32)) -> Result<Self, Self::Error> {
        ScreenRegion::build(x, y, width, height)
    }
}

impl TryFrom<[u32; 4]> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from([x, y, width, height]: [u32; 4]) -> Result<Self, Self::Error> {
        ScreenRegion::build(x, y, width, height)
    }
}
impl TryFrom<&[u32; 4]> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(&[x, y, width, height]: &[u32; 4]) -> Result<Self, Self::Error> {
        ScreenRegion::build(x, y, width, height)
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

impl From<&ScreenRegion> for [f32; 4] {
    fn from(region: &ScreenRegion) -> Self {
        [region.min_x, region.min_y, region.max_x, region.max_y].map(|bound| bound as f32)
    }
}

#[cfg(python)]
mod python_bindings {
    use super::*;
    use crate::bindings::python::{PyUnsigned, read_pair};
    use pyo3::prelude::*;

    #[pymethods]
    impl ScreenRegion {
        // ScreenRegion is a value type used as an argument to pass.setViewport()
        // and similar APIs. Its constructors are internal plumbing; the parity
        // audit tracks the parent methods, not the value type itself.
        #[doc(hidden)]
        #[new]
        pub fn new_py(origin: Vec<PyUnsigned<'_>>, size: Vec<PyUnsigned<'_>>) -> PyResult<Self> {
            let origin = read_pair(&origin, "ScreenRegion: origin")?;
            let size = read_pair(&size, "ScreenRegion: size")?;
            Ok(ScreenRegion::new(origin, size)?)
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
            ScreenRegion::try_from(&[8, 4][..]).ok(),
            Some(build_region(0, 0, 8, 4))
        );
        assert_eq!(
            ScreenRegion::try_from(&[1, 2, 8, 4][..]).ok(),
            Some(build_region(1, 2, 8, 4))
        );
        assert!(ScreenRegion::try_from(&[1, 2, 3][..]).is_err());
        assert!(ScreenRegion::try_from(&[1, 2, 0, 4][..]).is_err());
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
        use crate::ScreenRegionError;
        let overflow = |result: Result<ScreenRegion, ScreenRegionError>| {
            matches!(result, Err(ScreenRegionError::Overflow { .. }))
        };
        let message = ScreenRegion::try_from((u32::MAX - 2, 0, 5, 5))
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
        let edge = ScreenRegion::try_from((u32::MAX - 5, 0, 5, 5)).ok();
        assert_eq!(edge.map(|region| region.max_x), Some(u32::MAX));
    }

    // Story: a region with no width or no height is an error from every
    // constructor and conversion.
    #[test]
    fn refuses_a_side_of_zero() {
        use crate::ScreenRegionError;
        let empty = |result: Result<ScreenRegion, ScreenRegionError>| {
            matches!(result, Err(ScreenRegionError::Empty { .. }))
        };
        assert!(empty(ScreenRegion::new((4, 4), (0, 8))));
        assert!(empty(ScreenRegion::try_from((4, 4, 8, 0))));
        assert!(empty(ScreenRegion::try_from((0, 0))));
        assert!(empty(ScreenRegion::try_from((8, 0))));
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
            ScreenRegion::try_from((64, 32)).ok(),
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

    // Story: a pair of pairs reads as origin, then size,
    // so a viewport of `((8, 0), (2, 4))` covers 2 by 4 pixels from (8, 0).
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
        use crate::ScreenRegionError;
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
