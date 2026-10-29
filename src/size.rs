#[cfg(wasm)]
use crate::error::WebError;
#[cfg(wasm)]
use crate::platforms::web::{read_given, read_number};
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg(python)]
use crate::platforms::python::{PyUnsigned, read_list};
#[cfg(python)]
use pyo3::FromPyObject;
#[cfg(python)]
use pyo3::prelude::*;
#[cfg(python)]
use pyo3::types::PyMapping;

pub mod error;
#[cfg(any(wasm, python, test))]
use error::SizeError;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object, eq, module = "fragmentcolor"))]
#[cfg_attr(mobile, derive(uniffi::Record))]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd)]
pub struct Size {
    #[cfg_attr(wasm, wasm_bindgen(skip))]
    pub width: u32,
    #[cfg_attr(wasm, wasm_bindgen(skip))]
    pub height: u32,
    #[cfg_attr(wasm, wasm_bindgen(skip))]
    #[cfg_attr(mobile, uniffi(default = None))]
    pub depth: Option<u32>,
}

impl Size {
    pub fn new(width: u32, height: u32, depth: Option<u32>) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }

    /// A size from the 2 or 3 numbers JavaScript passed, each checked by [`read_dimension`].
    #[cfg(wasm)]
    pub(crate) fn from_numbers(numbers: &[f64]) -> Result<Size, SizeError> {
        let dimensions = numbers
            .iter()
            .map(|&number| check_dimension(number))
            .collect::<Result<Vec<u32>, SizeError>>()?;
        Size::from_dimensions(&dimensions)
    }

    /// A size from the 2 or 3 dimensions a binding read, depth last.
    #[cfg(any(wasm, python, test))]
    pub(crate) fn from_dimensions(dimensions: &[u32]) -> Result<Size, SizeError> {
        match *dimensions {
            [width, height] => Ok(Size::new(width, height, None)),
            [width, height, depth] => Ok(Size::new(width, height, Some(depth))),
            _ => Err(SizeError::TypeMismatch(format!(
                "a size takes 2 or 3 numbers, got {}",
                dimensions.len()
            ))),
        }
    }
}

/// One size, region or count number from JavaScript: a whole number from 0 to `u32::MAX`.
/// A fraction, NaN, infinity or a number out of range gives `None`.
#[cfg(any(wasm, test))]
pub(crate) fn read_dimension(number: f64) -> Option<u32> {
    let whole = number.is_finite() && number.fract() == 0.0;
    (whole && (0.0..=u32::MAX as f64).contains(&number)).then_some(number as u32)
}

/// One size dimension from JavaScript, or the error that names the number.
#[cfg(wasm)]
fn check_dimension(number: f64) -> Result<u32, SizeError> {
    read_dimension(number).ok_or_else(|| {
        SizeError::TypeMismatch(format!(
            "a size takes whole numbers from 0 up, got {number}"
        ))
    })
}

crate::impl_from_into_with_refs!(
    Size,
    wgpu::Extent3d,
    |s: Size| wgpu::Extent3d {
        width: s.width,
        height: s.height,
        depth_or_array_layers: s.depth.unwrap_or(1),
    },
    |e: wgpu::Extent3d| Size::new(e.width, e.height, Some(e.depth_or_array_layers))
);

crate::impl_from_into_with_refs!(Size, (u32, u32), |s: Size| (s.width, s.height), |t: (
    u32,
    u32
)| {
    Size::new(t.0, t.1, None)
});

crate::impl_from_into_with_refs!(
    Size,
    (u32, u32, u32),
    |s: Size| (s.width, s.height, s.depth.unwrap_or(1)),
    |t: (u32, u32, u32)| Size::new(t.0, t.1, Some(t.2))
);

crate::impl_from_into_with_refs!(
    Size,
    [u32; 2],
    |s: Size| [s.width, s.height],
    |a: [u32; 2]| Size::new(a[0], a[1], None)
);

crate::impl_from_into_with_refs!(
    Size,
    [u32; 3],
    |s: Size| [s.width, s.height, s.depth.unwrap_or(1)],
    |a: [u32; 3]| Size::new(a[0], a[1], Some(a[2]))
);

/// Reads `[w, h]` or `[w, h, d]` from a plain or typed array, or a `{ width, height, depth? }` object.
/// A value of another type, or a number that isn't one, is a `TypeError`, as Python raises,
/// and a fraction, a negative number, a wrong count or a missing side is an `InvalidInputError`.
#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for Size {
    type Error = WebError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use crate::platforms::web::{read_list, read_number};
        use js_sys::Reflect;

        if let Some(numbers) = read_list(value, "a size")? {
            return Ok(Size::from_numbers(&numbers)?);
        }

        if value.is_object() {
            let field = |name: &str| {
                Reflect::get(value, &JsValue::from_str(name))
                    .ok()
                    .filter(|item| !item.is_undefined() && !item.is_null())
                    .map(|item| read_number(&item, &format!("a size's {name}")))
                    .transpose()
            };
            let (Some(width), Some(height)) = (field("width")?, field("height")?) else {
                let missing = SizeError::TypeMismatch("a size object needs a width and a height".into());
                return Err(missing.into());
            };
            let numbers = match field("depth")? {
                Some(depth) => vec![width, height, depth],
                None => vec![width, height],
            };
            return Ok(Size::from_numbers(&numbers)?);
        }

        Err(WebError::type_error(&format!(
            "a size is [w, h], [w, h, d] or {{ width, height, depth? }}, got {value:?}"
        )))
    }
}

#[cfg(wasm)]
#[wasm_bindgen]
impl Size {
    // Size is a value type, like ScreenRegion. The parity audit tracks the
    // methods that take and return it, so its constructor has no doc page.
    #[doc(hidden)]
    #[wasm_bindgen(constructor)]
    pub fn new_js(
        #[wasm_bindgen(unchecked_param_type = "number")] width: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] height: &JsValue,
        #[wasm_bindgen(unchecked_optional_param_type = "number")] depth: Option<JsValue>,
    ) -> Result<Size, WebError> {
        let mut numbers = vec![
            read_number(width, "Size: width")?,
            read_number(height, "Size: height")?,
        ];
        if let Some(depth) = read_given(depth) {
            numbers.push(read_number(&depth, "Size: depth")?);
        }
        Ok(Size::from_numbers(&numbers)?)
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = width)]
    pub fn width_js(&self) -> u32 {
        self.width
    }

    /// Checks the number as the constructor does, and keeps the old width on an error.
    #[doc(hidden)]
    #[wasm_bindgen(setter = width)]
    pub fn set_width_js(&mut self, value: JsValue) -> Result<(), WebError> {
        self.width = check_dimension(read_number(&value, "Size.width")?)?;
        Ok(())
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = height)]
    pub fn height_js(&self) -> u32 {
        self.height
    }

    #[doc(hidden)]
    #[wasm_bindgen(setter = height)]
    pub fn set_height_js(&mut self, value: JsValue) -> Result<(), WebError> {
        self.height = check_dimension(read_number(&value, "Size.height")?)?;
        Ok(())
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = depth)]
    pub fn depth_js(&self) -> Option<u32> {
        self.depth
    }

    /// `undefined` or `null` clears the depth.
    #[doc(hidden)]
    #[wasm_bindgen(setter = depth)]
    pub fn set_depth_js(&mut self, value: Option<JsValue>) -> Result<(), WebError> {
        self.depth = match read_given(value) {
            Some(depth) => Some(check_dimension(read_number(&depth, "Size.depth")?)?),
            None => None,
        };
        Ok(())
    }
}

#[cfg(wasm)]
crate::impl_tryfrom_owned_via_ref!(Size, wasm_bindgen::JsValue, WebError);

#[cfg(python)]
#[pymethods]
impl Size {
    // Size is a value type, like ScreenRegion. The parity audit tracks the
    // methods that take and return it, so its constructor has no doc page.
    #[doc(hidden)]
    #[new]
    #[pyo3(signature = (width, height, depth=None))]
    pub fn new_py(
        width: PyUnsigned<'_>,
        height: PyUnsigned<'_>,
        depth: Option<PyUnsigned<'_>>,
    ) -> PyResult<Self> {
        Ok(Size::new(
            width.read("Size: width")?,
            height.read("Size: height")?,
            depth.map(|depth| depth.read("Size: depth")).transpose()?,
        ))
    }

    #[doc(hidden)]
    #[getter(width)]
    pub fn width_py(&self) -> u32 {
        self.width
    }

    /// Checks the int as the constructor does, and keeps the old width on an error.
    #[doc(hidden)]
    #[setter(width)]
    pub fn set_width_py(&mut self, value: PyUnsigned<'_>) -> PyResult<()> {
        self.width = value.read("Size.width")?;
        Ok(())
    }

    #[doc(hidden)]
    #[getter(height)]
    pub fn height_py(&self) -> u32 {
        self.height
    }

    #[doc(hidden)]
    #[setter(height)]
    pub fn set_height_py(&mut self, value: PyUnsigned<'_>) -> PyResult<()> {
        self.height = value.read("Size.height")?;
        Ok(())
    }

    #[doc(hidden)]
    #[getter(depth)]
    pub fn depth_py(&self) -> Option<u32> {
        self.depth
    }

    /// `None` clears the depth.
    #[doc(hidden)]
    #[setter(depth)]
    pub fn set_depth_py(&mut self, value: Option<PyUnsigned<'_>>) -> PyResult<()> {
        self.depth = value.map(|depth| depth.read("Size.depth")).transpose()?;
        Ok(())
    }
}

/// A size from Python: a `Size`, a mapping of its fields such as a dict, or `[w, h]` or `[w, h, d]`
/// in any sequence. Each number is an int that [`PyUnsigned::read`] checks.
#[cfg(python)]
#[derive(FromPyObject)]
pub enum PySize<'py> {
    Size(Size),
    /// Tried before the list, which a mapping such as `UserDict` also passes as its keys.
    #[pyo3(annotation = "Mapping[str, int]")]
    Fields(Bound<'py, PyMapping>),
    Numbers(Vec<PyUnsigned<'py>>),
}

/// Checks a size from Python with the same rule as every other binding.
#[cfg(python)]
impl TryFrom<PySize<'_>> for Size {
    type Error = PyErr;

    fn try_from(value: PySize<'_>) -> Result<Self, Self::Error> {
        let dimensions = match value {
            PySize::Size(size) => return Ok(size),
            PySize::Numbers(items) => read_list(&items, "size")?,
            PySize::Fields(fields) => {
                let field = |name: &str| -> PyResult<Option<u32>> {
                    if !fields.contains(name)? {
                        return Ok(None);
                    }
                    let value = PyUnsigned::Int(fields.get_item(name)?);
                    value.read(&format!("size: {name}")).map(Some)
                };
                let (Some(width), Some(height)) = (field("width")?, field("height")?) else {
                    return Err(SizeError::TypeMismatch(
                        "a size dict needs a width and a height".into(),
                    )
                    .into());
                };
                match field("depth")? {
                    Some(depth) => vec![width, height, depth],
                    None => vec![width, height],
                }
            }
        };
        Ok(Size::from_dimensions(&dimensions)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn size_wgpu_ref_conversions() {
        let e = wgpu::Extent3d {
            width: 640,
            height: 480,
            depth_or_array_layers: 1,
        };
        let s: Size = (&e).into();
        assert_eq!(s.width, 640);
        assert_eq!(s.height, 480);
        assert_eq!(s.depth, Some(1));

        let e2: wgpu::Extent3d = (&s).into();
        assert_eq!(e2, e);
    }

    // Story: Conversions between tuples/arrays and Size (owned and borrowed) behave as expected.
    #[test]
    fn size_tuple_array_conversions() {
        // (w,h) -> Size and back
        let s: Size = (320u32, 240u32).into();
        assert_eq!((320, 240), <(u32, u32)>::from(s));

        // & (w,h)
        let t = (800u32, 600u32);
        let s2: Size = (&t).into();
        let t2: (u32, u32) = (&s2).into();
        assert_eq!(t2, t);

        // (w,h,d) -> Size and back
        let s3: Size = (1u32, 2u32, 3u32).into();
        let back3: (u32, u32, u32) = s3.into();
        assert_eq!(back3, (1, 2, 3));

        // arrays [w,h] and [w,h,d]
        let s4: Size = [10u32, 20u32].into();
        let arr2: [u32; 2] = s4.into();
        assert_eq!(arr2, [10, 20]);

        let s5: Size = [3u32, 4u32, 5u32].into();
        let arr3: [u32; 3] = s5.into();
        assert_eq!(arr3, [3, 4, 5]);

        // From &Size into arrays
        let s6 = Size::new(7, 8, Some(9));
        let arr2b: [u32; 2] = (&s6).into();
        let arr3b: [u32; 3] = (&s6).into();
        assert_eq!(arr2b, [7, 8]);
        assert_eq!(arr3b, [7, 8, 9]);
    }

    // Story: Default and new() constructors, and depth fallback when converting to Extent3d.
    #[test]
    fn size_new_default_and_depth_fallback() {
        let d = Size::default();
        assert_eq!(d.width, 0);
        assert_eq!(d.height, 0);
        assert_eq!(d.depth, None);

        let s = Size::new(4, 5, None);
        let e: wgpu::Extent3d = s.into();
        assert_eq!(e.width, 4);
        assert_eq!(e.height, 5);
        // Depth should fallback to 1 when None
        assert_eq!(e.depth_or_array_layers, 1);
    }

    // Property: tuple2 <-> Size <-> tuple2 roundtrips width/height
    proptest! {
        #[test]
        fn prop_tuple2_roundtrip((w, h) in any::<(u32, u32)>()) {
            let s: Size = (w, h).into();
            let back: (u32, u32) = s.into();
            prop_assert_eq!(back, (w, h));

            let s_ref: Size = (&(w, h)).into();
            let back_ref: (u32, u32) = (&s_ref).into();
            prop_assert_eq!(back_ref, (w, h));
        }
    }

    // Property: tuple3 <-> Size <-> tuple3 preserves all three components
    proptest! {
        #[test]
        fn prop_tuple3_roundtrip((w, h, d) in any::<(u32, u32, u32)>()) {
            let s: Size = (w, h, d).into();
            let back: (u32, u32, u32) = s.into();
            prop_assert_eq!(back, (w, h, d));

            let s_ref: Size = (&(w, h, d)).into();
            let back_ref: (u32, u32, u32) = (&s_ref).into();
            prop_assert_eq!(back_ref, (w, h, d));
        }
    }

    // Property: arrays [w,h] and [w,h,d] roundtrip width/height/depth
    proptest! {
        #[test]
        fn prop_arrays_roundtrip(a2 in any::<[u32;2]>(), a3 in any::<[u32;3]>()) {
            let s2: Size = a2.into();
            let back2: [u32; 2] = s2.into();
            prop_assert_eq!(back2, a2);

            let s3: Size = a3.into();
            let back3: [u32; 3] = s3.into();
            prop_assert_eq!(back3, a3);
        }
    }

    // Property: Extent3d conversions preserve width/height and treat depth None as 1
    proptest! {
        #[test]
        fn prop_extent3d_roundtrip((w, h, d_opt) in any::<(u32, u32, Option<u32>)>()) {
            let s = Size::new(w, h, d_opt);
            let e: wgpu::Extent3d = s.into();
            prop_assert_eq!(e.width, w);
            prop_assert_eq!(e.height, h);
            // Compare effective depth
            let eff = d_opt.unwrap_or(1);
            prop_assert_eq!(e.depth_or_array_layers, eff);

            let s2: Size = (&e).into();
            prop_assert_eq!(s2.width, w);
            prop_assert_eq!(s2.height, h);
            prop_assert_eq!(s2.depth.unwrap_or(1), eff);
        }
    }

    // Story: a JavaScript size, region or count takes a whole number from 0
    // up, and a fraction fails where 0.12.6 rounded it down.
    #[test]
    fn refuses_fractions() {
        assert_eq!(read_dimension(4.0), Some(4));
        assert_eq!(read_dimension(4294967295.0), Some(u32::MAX));
        for number in [1.5, 0.5, -1.0, f64::NAN, f64::INFINITY, 4294967296.0] {
            assert_eq!(read_dimension(number), None, "{number}");
        }
    }

    // Story: a binding's size has 2 or 3 dimensions, depth last. The Python
    // healthcheck covers the int checks, which need a Python interpreter.
    #[test]
    fn reads_two_or_three_dimensions() {
        assert_eq!(
            Size::from_dimensions(&[4, 2]).ok(),
            Some(Size::new(4, 2, None))
        );
        assert_eq!(
            Size::from_dimensions(&[4, 2, 3]).ok(),
            Some(Size::new(4, 2, Some(3)))
        );
        assert!(Size::from_dimensions(&[4]).is_err());
        assert!(Size::from_dimensions(&[1, 2, 3, 4]).is_err());
    }
}
