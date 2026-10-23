#[cfg(wasm)]
use wasm_bindgen::prelude::*;

pub mod error;
#[cfg(wasm)]
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
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
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
    pub fn new(origin: impl Into<(u32, u32)>, size: impl Into<(u32, u32)>) -> Self {
        let (x, y) = origin.into();
        let (w, h) = size.into();
        Self::from_region(x, y, w, h)
    }

    pub fn from_region_i32(x: i32, y: i32, width: i32, height: i32) -> Self {
        let a = (x, y);
        let b = (x.saturating_add(width), y.saturating_add(height));
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        Self {
            min_x: min.0.max(0) as u32,
            min_y: min.1.max(0) as u32,
            max_x: max.0.max(0) as u32,
            max_y: max.1.max(0) as u32,
        }
    }

    pub fn from_region(x: u32, y: u32, width: u32, height: u32) -> Self {
        let a: (u32, u32) = (x, y);
        let b = (x.saturating_add(width), y.saturating_add(height));
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        Self {
            min_x: min.0,
            min_y: min.1,
            max_x: max.0,
            max_y: max.1,
        }
    }

    pub fn from_tuples_i32(a: (i32, i32), b: (i32, i32)) -> Self {
        Self::from_tuples(
            (a.0.max(0) as u32, a.1.max(0) as u32),
            (b.0.max(0) as u32, b.1.max(0) as u32),
        )
    }

    /// A region of exactly `size` pixels at the origin, the same as [`ScreenRegion::from_size`].
    pub fn from_tuple(size: (u32, u32)) -> Self {
        Self::from_size(size.0, size.1)
    }

    /// The region between two corners, both inside it, so it spans one pixel more than their distance.
    pub fn from_tuples(a: (u32, u32), b: (u32, u32)) -> Self {
        // Figure out what our two ranges are
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        // Increase max by one pixel as we've calculated the *encompassed* max
        let max = (max.0.saturating_add(1), max.1.saturating_add(1));

        Self {
            min_x: min.0,
            min_y: min.1,
            max_x: max.0,
            max_y: max.1,
        }
    }

    pub fn from_arrays_i32(a: [i32; 2], b: [i32; 2]) -> Self {
        Self::from_tuples_i32((a[0], a[1]), (b[0], b[1]))
    }

    pub fn to_array(&self) -> [f32; 4] {
        [
            self.min_x as f32,
            self.min_y as f32,
            self.max_x as f32,
            self.max_y as f32,
        ]
    }

    pub fn from_size(width: u32, height: u32) -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: width,
            max_y: height,
        }
    }

    pub fn from_size_f32(width: f32, height: f32) -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: width as u32,
            max_y: height as u32,
        }
    }

    pub fn from_pixel(x: u32, y: u32) -> Self {
        Self {
            min_x: x,
            min_y: y,
            max_x: x + 1,
            max_y: y + 1,
        }
    }

    pub fn clamp(&mut self, width: u32, height: u32) {
        self.min_x = self.min_x.min(width);
        self.min_y = self.min_y.min(height);
        self.max_x = self.max_x.min(width);
        self.max_y = self.max_y.min(height);
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

    pub fn encompass(&mut self, x: u32, y: u32) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x + 1);
        self.max_y = self.max_y.max(y + 1);
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

    pub fn from_inbound_radius(radius: f32) -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: (radius * 2.0) as u32,
            max_y: (radius * 2.0) as u32,
        }
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
                .map(|item| js_to_dimension(&item))
                .collect::<Result<Vec<u32>, ScreenRegionError>>()?;
            return match numbers[..] {
                [width, height] => Ok(ScreenRegion::from_size(width, height)),
                [x, y, width, height] => Ok(ScreenRegion::from_region(x, y, width, height)),
                _ => Err(ScreenRegionError::TypeMismatch(format!(
                    "a region takes [w, h] or [x, y, w, h], got {} numbers",
                    numbers.len()
                ))),
            };
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
                return Ok(ScreenRegion {
                    min_x: min_x.min(max_x),
                    min_y: min_y.min(max_y),
                    max_x: max_x.max(min_x),
                    max_y: max_y.max(min_y),
                });
            }
            let rect = (
                field(&["x"])?,
                field(&["y"])?,
                field(&["width"])?,
                field(&["height"])?,
            );
            if let (Some(x), Some(y), Some(width), Some(height)) = rect {
                return Ok(ScreenRegion::from_region(x, y, width, height));
            }
        }

        Err(ScreenRegionError::TypeMismatch(
            "Cannot convert JavaScript value to ScreenRegion (expected [x,y,w,h], [w,h], or {x,y,width,height})".into(),
        ))
    }
}

/// One region number from JS, checked by [`crate::size::read_dimension`].
#[cfg(wasm)]
fn js_to_dimension(item: &JsValue) -> Result<u32, ScreenRegionError> {
    item.as_f64()
        .and_then(crate::size::read_dimension)
        .ok_or_else(|| {
            ScreenRegionError::TypeMismatch(format!(
                "a region takes numbers from 0 up, got {item:?}"
            ))
        })
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
        Ok(ScreenRegion::new(js_to_pair(origin)?, js_to_pair(size)?))
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

crate::impl_from_into_with_refs!(
    ScreenRegion,
    wgpu::Extent3d,
    |r: ScreenRegion| wgpu::Extent3d {
        width: r.width(),
        height: r.height(),
        depth_or_array_layers: 1
    },
    |e: wgpu::Extent3d| ScreenRegion::from_size(e.width, e.height)
);

// -------------------------------------------
// Trait-based conversions (preferred surface)
// -------------------------------------------

impl From<(u32, u32)> for ScreenRegion {
    #[inline]
    fn from(size: (u32, u32)) -> Self {
        ScreenRegion::from_size(size.0, size.1)
    }
}
impl From<&(u32, u32)> for ScreenRegion {
    #[inline]
    fn from(size: &(u32, u32)) -> Self {
        ScreenRegion::from_size(size.0, size.1)
    }
}

impl From<[u32; 2]> for ScreenRegion {
    #[inline]
    fn from(a: [u32; 2]) -> Self {
        ScreenRegion::from_size(a[0], a[1])
    }
}
impl From<&[u32; 2]> for ScreenRegion {
    #[inline]
    fn from(a: &[u32; 2]) -> Self {
        ScreenRegion::from_size(a[0], a[1])
    }
}

impl From<(u32, u32, u32, u32)> for ScreenRegion {
    #[inline]
    fn from(t: (u32, u32, u32, u32)) -> Self {
        ScreenRegion::from_region(t.0, t.1, t.2, t.3)
    }
}
impl From<&(u32, u32, u32, u32)> for ScreenRegion {
    #[inline]
    fn from(t: &(u32, u32, u32, u32)) -> Self {
        ScreenRegion::from_region(t.0, t.1, t.2, t.3)
    }
}

impl From<[u32; 4]> for ScreenRegion {
    #[inline]
    fn from(a: [u32; 4]) -> Self {
        ScreenRegion::from_region(a[0], a[1], a[2], a[3])
    }
}
impl From<&[u32; 4]> for ScreenRegion {
    #[inline]
    fn from(a: &[u32; 4]) -> Self {
        ScreenRegion::from_region(a[0], a[1], a[2], a[3])
    }
}

impl From<((u32, u32), (u32, u32))> for ScreenRegion {
    #[inline]
    fn from(p: ((u32, u32), (u32, u32))) -> Self {
        ScreenRegion::from_tuples(p.0, p.1)
    }
}
impl From<&((u32, u32), (u32, u32))> for ScreenRegion {
    #[inline]
    fn from(p: &((u32, u32), (u32, u32))) -> Self {
        ScreenRegion::from_tuples(p.0, p.1)
    }
}

impl From<(i32, i32, i32, i32)> for ScreenRegion {
    #[inline]
    fn from(t: (i32, i32, i32, i32)) -> Self {
        ScreenRegion::from_region_i32(t.0, t.1, t.2, t.3)
    }
}
impl From<&(i32, i32, i32, i32)> for ScreenRegion {
    #[inline]
    fn from(t: &(i32, i32, i32, i32)) -> Self {
        ScreenRegion::from_region_i32(t.0, t.1, t.2, t.3)
    }
}

impl From<([i32; 2], [i32; 2])> for ScreenRegion {
    #[inline]
    fn from(p: ([i32; 2], [i32; 2])) -> Self {
        ScreenRegion::from_arrays_i32(p.0, p.1)
    }
}
impl From<&([i32; 2], [i32; 2])> for ScreenRegion {
    #[inline]
    fn from(p: &([i32; 2], [i32; 2])) -> Self {
        ScreenRegion::from_arrays_i32(p.0, p.1)
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
    use pyo3::prelude::*;

    #[pymethods]
    impl ScreenRegion {
        // ScreenRegion is a value type used as an argument to pass.setViewport()
        // and similar APIs. Its constructors are internal plumbing; the parity
        // audit tracks the parent methods, not the value type itself.
        #[doc(hidden)]
        #[new]
        pub fn new_py(origin: (u32, u32), size: (u32, u32)) -> Self {
            ScreenRegion::new(origin, size)
        }

        #[doc(hidden)]
        #[staticmethod]
        #[pyo3(name = "from_region")]
        pub fn from_region_py(x: u32, y: u32, width: u32, height: u32) -> Self {
            ScreenRegion::from_region(x, y, width, height)
        }

        #[doc(hidden)]
        #[staticmethod]
        #[pyo3(name = "from_tuple")]
        pub fn from_tuple_py(size: (u32, u32)) -> Self {
            ScreenRegion::from_tuple(size)
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
            ScreenRegion::from_size(10, 10),
            ScreenRegion::from_size(10, 10),
            (0, 0),
            (0, 0),
            (5, 5),
            ScreenRegion::from_region_i32(0, 0, 5, 5),
            ScreenRegion::from_region_i32(0, 0, 5, 5),
        );

        test(
            ScreenRegion::from_size(10, 10),
            ScreenRegion::from_size(150, 150),
            (-1, -1),
            (100, 100),
            (5, 5),
            ScreenRegion::from_region_i32(0, 0, 4, 4),
            ScreenRegion::from_region_i32(101, 101, 4, 4),
        );

        test(
            ScreenRegion::from_size(10, 10),
            ScreenRegion::from_size(150, 150),
            (-1, -1),
            (100, 100),
            (15, 15),
            ScreenRegion::from_region_i32(0, 0, 10, 10),
            ScreenRegion::from_region_i32(101, 101, 10, 10),
        );

        test(
            ScreenRegion::from_region(10, 10, 20, 20),
            ScreenRegion::from_size(150, 150),
            (15, 5),
            (0, 0),
            (15, 15),
            ScreenRegion::from_region_i32(15, 10, 15, 10),
            ScreenRegion::from_region_i32(0, 5, 15, 10),
        );

        test(
            ScreenRegion::from_size(800, 600),
            ScreenRegion::from_size(200, 40),
            (400, 440),
            (40, 0),
            (40, 40),
            ScreenRegion::from_region_i32(400, 440, 40, 40),
            ScreenRegion::from_region_i32(40, 0, 40, 40),
        );

        test(
            ScreenRegion::from_size(240, 180),
            ScreenRegion::from_size(238, 164),
            (-1, 0),
            (0, 0),
            (240, 180),
            ScreenRegion::from_region_i32(0, 0, 237, 164),
            ScreenRegion::from_region_i32(1, 0, 237, 164),
        );

        test(
            ScreenRegion::from_size(10, 10),
            ScreenRegion::from_size(10, 10),
            (15, 0),
            (0, 15),
            (100, 100),
            ScreenRegion::from_region_i32(0, 0, 0, 0),
            ScreenRegion::from_region_i32(0, 0, 0, 0),
        );
    }

    // Story: a size alone covers exactly that many pixels, while two corners
    // both sit inside the region.
    #[test]
    fn size_forms_cover_exact_pixels() {
        assert_eq!(ScreenRegion::from([32, 32]).width(), 32);
        assert_eq!(ScreenRegion::from((64, 32)).height(), 32);
        assert_eq!(
            ScreenRegion::from_tuple((64, 32)),
            ScreenRegion::from_size(64, 32)
        );
        assert_eq!(
            ScreenRegion::from([2, 4, 8, 6]),
            ScreenRegion::from_region(2, 4, 8, 6)
        );
        assert_eq!(ScreenRegion::from_tuples((0, 0), (31, 31)).width(), 32);
    }

    // Story: Construct regions using helpers and verify geometry utilities.
    #[test]
    fn computes_area_aspect_and_centers() {
        // Arrange
        let r = ScreenRegion::from_region(2, 4, 8, 6); // spans x:[2,10), y:[4,10)

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
        let mut a = ScreenRegion::from_region(0, 0, 4, 4);
        let b = ScreenRegion::from_region(2, 2, 4, 4);

        // Act
        let overlaps = a.intersects(b);
        a.union(b);

        // Assert
        assert!(overlaps);
        assert_eq!(a, ScreenRegion::from_region(0, 0, 6, 6));
    }

    // Story: Clamp, encompass and equals work at the boundaries; radii helpers return intuitive values.
    #[test]
    fn clamp_encompass_equals_and_radii() {
        let mut r = ScreenRegion::from_region(1, 2, 3, 4);
        r.clamp(2, 3);
        assert_eq!(r, ScreenRegion::from_region(1, 2, 1, 1)); // max clamped down

        r.encompass(5, 6);
        assert_eq!(r.max_x, 6);
        assert_eq!(r.max_y, 7);

        let r2 = ScreenRegion::from_region(1, 2, 5, 5);
        assert!(r.equals(r2));

        let square = ScreenRegion::from_region(0, 0, 4, 4);
        assert!((square.inbound_radius() - 2.0).abs() < 1e-6);
        assert!((square.outbound_radius() - (2.0f32.hypot(2.0))).abs() < 1e-6);
    }

    // Story: Clip keeps the part of a region inside an area and orders its corners.
    #[test]
    fn clips_to_an_area() {
        let inside = ScreenRegion::from_region(2, 2, 4, 4);
        assert_eq!(inside.clip(8, 8), inside);

        let past = ScreenRegion::from_region(6, 4, 10, 10);
        assert_eq!(past.clip(8, 8), ScreenRegion::from_region(6, 4, 2, 4));

        let outside = ScreenRegion::from_region(10, 10, 4, 4);
        assert_eq!(outside.clip(8, 8).area(), 0);

        let flipped = ScreenRegion {
            min_x: 5,
            min_y: 6,
            max_x: 1,
            max_y: 2,
        };
        assert_eq!(flipped.clip(8, 8), ScreenRegion::from_region(1, 2, 4, 4));
    }
}
