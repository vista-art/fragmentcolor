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

    /// The region, or [`ScreenRegionError::Empty`] when it has no width or no height.
    /// The fields are public, so a viewport checks the region it receives too.
    pub(crate) fn check(self) -> Result<Self, ScreenRegionError> {
        if self.width() == 0 || self.height() == 0 {
            return Err(ScreenRegionError::Empty {
                width: self.width(),
                height: self.height(),
            });
        }
        Ok(self)
    }

    /// The region between two corners, both inside it, so it spans one pixel more than their distance.
    /// A corner at `u32::MAX` has no pixel past it, so it is an error.
    pub fn from_tuples(a: (u32, u32), b: (u32, u32)) -> Result<Self, ScreenRegionError> {
        let (min_x, min_y) = (a.0.min(b.0), a.1.min(b.1));
        let (far_x, far_y) = (a.0.max(b.0), a.1.max(b.1));
        match (
            (far_x - min_x).checked_add(1),
            (far_y - min_y).checked_add(1),
        ) {
            (Some(width), Some(height)) => Self::from_region(min_x, min_y, width, height),
            (width, height) => Err(ScreenRegionError::Overflow {
                x: min_x,
                y: min_y,
                width: width.unwrap_or(u32::MAX),
                height: height.unwrap_or(u32::MAX),
            }),
        }
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

    /// The part of this region inside a `width` by `height` area, min corner first.
    pub(crate) fn clip(&self, width: u32, height: u32) -> ScreenRegion {
        Self {
            min_x: self.min_x.min(self.max_x).min(width),
            min_y: self.min_y.min(self.max_y).min(height),
            max_x: self.min_x.max(self.max_x).min(width),
            max_y: self.min_y.max(self.max_y).min(height),
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

    pub fn width(&self) -> u32 {
        u32::abs_diff(self.max_x, self.min_x)
    }

    pub fn height(&self) -> u32 {
        u32::abs_diff(self.max_y, self.min_y)
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

    /// Clamps this ScreenRegion to a theoretical overlap of another ScreenRegion,
    /// referring to "overlapping pixels" (such as a copy destination vs copy source),
    /// in such a way that only pixels that are valid for both Regions are valid.
    ///
    /// The other ScreenRegion is also clamped to reflect the same overlap.
    ///
    /// The overlap of two regions starts at `self_point` on `self`, and `other_point` on `other`,
    /// and is at most `size` big.
    ///
    /// The overlap does not actually need to happen on the same coordinate plane,
    /// for example -1,-1 on this may be 100,100 on other, with an overlap region of 5x5.
    /// As long as both textures can fit that, that's considered an overlap.
    /// However, since -1,-1 is outside of the valid area on the first region,
    /// the overlap actually happens at 0,0 and 101,101 for a size of 4x4.
    pub fn clamp_with_intersection(
        &mut self,
        self_point: (i32, i32),
        other_point: (i32, i32),
        size: (i32, i32),
        other: &mut ScreenRegion,
    ) {
        // Translate both regions to same coordinate system.

        let r1 = (
            self.min_x as i32,
            self.min_y as i32,
            self.max_x as i32,
            self.max_y as i32,
        );
        let r2 = (
            other.min_x as i32,
            other.min_y as i32,
            other.max_x as i32,
            other.max_y as i32,
        );

        let r1_trans = translate_region(r1, (-self_point.0, -self_point.1));
        let r2_trans = translate_region(r2, (-other_point.0, -other_point.1));

        // Intersection.

        let inters = intersection_same_coordinate_system(
            intersection_same_coordinate_system(r1_trans, r2_trans),
            (0, 0, size.0, size.1),
        );

        // Translate the intersection back.

        let r1_result = translate_region(inters, self_point);
        let r2_result = translate_region(inters, other_point);

        // Ensure empty results yield (0, 0, 0, 0).

        let is_empty = inters.0 == inters.2 || inters.1 == inters.3;

        let r1_result = if is_empty { (0, 0, 0, 0) } else { r1_result };
        let r2_result = if is_empty { (0, 0, 0, 0) } else { r2_result };

        // Mutate.

        self.min_x = r1_result.0 as u32;
        self.min_y = r1_result.1 as u32;
        self.max_x = r1_result.2 as u32;
        self.max_y = r1_result.3 as u32;

        other.min_x = r2_result.0 as u32;
        other.min_y = r2_result.1 as u32;
        other.max_x = r2_result.2 as u32;
        other.max_y = r2_result.3 as u32;
    }
}

#[inline]
fn intersection_same_coordinate_system(
    (r1_min_x, r1_min_y, r1_max_x, r1_max_y): (i32, i32, i32, i32),
    (r2_min_x, r2_min_y, r2_max_x, r2_max_y): (i32, i32, i32, i32),
) -> (i32, i32, i32, i32) {
    // To guard against 'min' being larger than 'max'.
    let r1_min_x = r1_min_x.min(r1_max_x);
    let r1_min_y = r1_min_y.min(r1_max_y);
    let r2_min_x = r2_min_x.min(r2_max_x);
    let r2_min_y = r2_min_y.min(r2_max_y);

    // First part of intersection.
    let r3_min_x = r1_min_x.max(r2_min_x);
    let r3_min_y = r1_min_y.max(r2_min_y);
    let r3_max_x = r1_max_x.min(r2_max_x);
    let r3_max_y = r1_max_y.min(r2_max_y);

    // In case of no overlap.
    let r3_min_x = r3_min_x.min(r3_max_x);
    let r3_min_y = r3_min_y.min(r3_max_y);

    (r3_min_x, r3_min_y, r3_max_x, r3_max_y)
}

#[inline]
fn translate_region(
    (r_min_x, r_min_y, r_max_x, r_max_y): (i32, i32, i32, i32),
    (trans_x, trans_y): (i32, i32),
) -> (i32, i32, i32, i32) {
    (
        r_min_x + trans_x,
        r_min_y + trans_y,
        r_max_x + trans_x,
        r_max_y + trans_y,
    )
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
                    min_x: min_x.min(max_x),
                    min_y: min_y.min(max_y),
                    max_x: max_x.max(min_x),
                    max_y: max_y.max(min_y),
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
    pub fn new_js(origin: &JsValue, size: &JsValue) -> Result<ScreenRegion, JsError> {
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

impl TryFrom<((u32, u32), (u32, u32))> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(p: ((u32, u32), (u32, u32))) -> Result<Self, Self::Error> {
        ScreenRegion::from_tuples(p.0, p.1)
    }
}
impl TryFrom<&((u32, u32), (u32, u32))> for ScreenRegion {
    type Error = ScreenRegionError;

    #[inline]
    fn try_from(p: &((u32, u32), (u32, u32))) -> Result<Self, Self::Error> {
        ScreenRegion::from_tuples(p.0, p.1)
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

    #[test]
    fn clamp_with_intersection() {
        fn test(
            mut a: ScreenRegion,
            mut b: ScreenRegion,
            a_point: (i32, i32),
            b_point: (i32, i32),
            size: (i32, i32),
            expected_a: ScreenRegion,
            expected_b: ScreenRegion,
        ) {
            a.clamp_with_intersection(a_point, b_point, size, &mut b);

            assert_eq!(expected_a, a, "a (self) region should match");
            assert_eq!(expected_b, b, "b (other) region should match");
        }

        test(
            build_region(0, 0, 10, 10),
            build_region(0, 0, 10, 10),
            (0, 0),
            (0, 0),
            (5, 5),
            build_region(0, 0, 5, 5),
            build_region(0, 0, 5, 5),
        );

        test(
            build_region(0, 0, 10, 10),
            build_region(0, 0, 150, 150),
            (-1, -1),
            (100, 100),
            (5, 5),
            build_region(0, 0, 4, 4),
            build_region(101, 101, 4, 4),
        );

        test(
            build_region(0, 0, 10, 10),
            build_region(0, 0, 150, 150),
            (-1, -1),
            (100, 100),
            (15, 15),
            build_region(0, 0, 10, 10),
            build_region(101, 101, 10, 10),
        );

        test(
            build_region(10, 10, 20, 20),
            build_region(0, 0, 150, 150),
            (15, 5),
            (0, 0),
            (15, 15),
            build_region(15, 10, 15, 10),
            build_region(0, 5, 15, 10),
        );

        test(
            build_region(0, 0, 800, 600),
            build_region(0, 0, 200, 40),
            (400, 440),
            (40, 0),
            (40, 40),
            build_region(400, 440, 40, 40),
            build_region(40, 0, 40, 40),
        );

        test(
            build_region(0, 0, 240, 180),
            build_region(0, 0, 238, 164),
            (-1, 0),
            (0, 0),
            (240, 180),
            build_region(0, 0, 237, 164),
            build_region(1, 0, 237, 164),
        );

        test(
            build_region(0, 0, 10, 10),
            build_region(0, 0, 10, 10),
            (15, 0),
            (0, 15),
            (100, 100),
            build_region(0, 0, 0, 0),
            build_region(0, 0, 0, 0),
        );
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
        assert!(overflow(ScreenRegion::from_tuples((0, 0), (u32::MAX, 3))));
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
        assert!(empty(build_region(3, 3, 0, 2).check()));
        assert_eq!(
            build_region(3, 3, 1, 1).check().ok(),
            Some(build_region(3, 3, 1, 1))
        );
    }

    // Story: a size alone covers exactly that many pixels, while two corners
    // both sit inside the region.
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
        let corners = ScreenRegion::from_tuples((0, 0), (31, 31));
        assert_eq!(corners.map(|region| region.width()).ok(), Some(32));
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

    // Story: Clip keeps the part of a region inside an area and orders its corners.
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
        assert_eq!(flipped.clip(8, 8), build_region(1, 2, 4, 4));
    }
}
