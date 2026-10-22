#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg(python)]
use pyo3::FromPyObject;
#[cfg(python)]
use pyo3::prelude::*;

pub mod error;
#[cfg(any(wasm, python))]
use error::SizeError;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(
    python,
    pyclass(from_py_object, get_all, set_all, eq, module = "fragmentcolor")
)]
#[cfg_attr(mobile, derive(uniffi::Record))]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd)]
pub struct Size {
    pub width: u32,
    pub height: u32,
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

    /// A size from the 2 or 3 numbers a binding received, each checked by [`read_dimension`].
    #[cfg(any(wasm, python))]
    pub(crate) fn from_numbers(numbers: &[f64]) -> Result<Size, SizeError> {
        let dimension = |number: f64| {
            read_dimension(number).ok_or_else(|| {
                SizeError::TypeMismatch(format!("a size takes numbers from 0 up, got {number}"))
            })
        };
        match *numbers {
            [width, height] => Ok(Size::new(dimension(width)?, dimension(height)?, None)),
            [width, height, depth] => Ok(Size::new(
                dimension(width)?,
                dimension(height)?,
                Some(dimension(depth)?),
            )),
            _ => Err(SizeError::TypeMismatch(format!(
                "a size takes 2 or 3 numbers, got {}",
                numbers.len()
            ))),
        }
    }
}

/// One size or region dimension from a binding.
/// It must be a finite number from 0 to `u32::MAX`, and a fraction rounds down.
#[cfg(any(wasm, python))]
pub(crate) fn read_dimension(number: f64) -> Option<u32> {
    (number.is_finite() && (0.0..=u32::MAX as f64).contains(&number)).then_some(number as u32)
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
#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for Size {
    type Error = SizeError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::{Array, ArrayBuffer, Reflect};

        let number = |item: JsValue| {
            item.as_f64().ok_or_else(|| {
                SizeError::TypeMismatch(format!("a size takes numbers, got {item:?}"))
            })
        };

        if Array::is_array(value) || ArrayBuffer::is_view(value) {
            let numbers = Array::from(value)
                .iter()
                .map(number)
                .collect::<Result<Vec<f64>, SizeError>>()?;
            return Size::from_numbers(&numbers);
        }

        if value.is_object() {
            let field = |name: &str| {
                Reflect::get(value, &JsValue::from_str(name))
                    .ok()
                    .filter(|item| !item.is_undefined() && !item.is_null())
                    .map(number)
                    .transpose()
            };
            let (Some(width), Some(height)) = (field("width")?, field("height")?) else {
                return Err(SizeError::TypeMismatch(
                    "a size object needs a width and a height".into(),
                ));
            };
            let numbers = match field("depth")? {
                Some(depth) => vec![width, height, depth],
                None => vec![width, height],
            };
            return Size::from_numbers(&numbers);
        }

        Err(SizeError::TypeMismatch(
            "Cannot convert JavaScript value to Size (expected [w,h] or [w,h,d])".into(),
        ))
    }
}

#[cfg(wasm)]
#[wasm_bindgen]
impl Size {
    // Size is a value type, like ScreenRegion. The parity audit tracks the
    // methods that take and return it, so its constructor has no doc page.
    #[doc(hidden)]
    #[wasm_bindgen(constructor)]
    pub fn new_js(width: f64, height: f64, depth: Option<f64>) -> Result<Size, JsError> {
        let numbers = match depth {
            Some(depth) => vec![width, height, depth],
            None => vec![width, height],
        };
        Ok(Size::from_numbers(&numbers)?)
    }
}

#[cfg(wasm)]
crate::impl_tryfrom_owned_via_ref!(Size, wasm_bindgen::JsValue, SizeError);

#[cfg(python)]
#[pymethods]
impl Size {
    // Size is a value type, like ScreenRegion. The parity audit tracks the
    // methods that take and return it, so its constructor has no doc page.
    #[doc(hidden)]
    #[new]
    #[pyo3(signature = (width, height, depth=None))]
    pub fn new_py(width: u32, height: u32, depth: Option<u32>) -> Self {
        Size::new(width, height, depth)
    }
}

#[cfg(python)]
#[derive(FromPyObject, IntoPyObject)]
pub enum PySize {
    Size(Size),
    ListF64(Vec<f64>),
    ListU32(Vec<u32>),
    ListI32(Vec<i32>),
    TupleF64((f64, f64)),
    TupleU32((u32, u32)),
    TupleI32((i32, i32)),
    Tuple3F64((f64, f64, f64)),
    Tuple3U32((u32, u32, u32)),
    Tuple3I32((i32, i32, i32)),
    DictF64(std::collections::HashMap<String, f64>),
    DictU32(std::collections::HashMap<String, u32>),
    DictI32(std::collections::HashMap<String, i32>),
}

/// Checks a size from Python with the same rule as every other binding.
#[cfg(python)]
impl TryFrom<PySize> for Size {
    type Error = SizeError;

    fn try_from(value: PySize) -> Result<Self, Self::Error> {
        fn from_map<N: Copy + Into<f64>>(
            map: &std::collections::HashMap<String, N>,
        ) -> Result<Size, SizeError> {
            let field = |name: &str| map.get(name).map(|&number| number.into());
            let (Some(width), Some(height)) = (field("width"), field("height")) else {
                return Err(SizeError::TypeMismatch(
                    "a size dict needs a width and a height".into(),
                ));
            };
            match field("depth") {
                Some(depth) => Size::from_numbers(&[width, height, depth]),
                None => Size::from_numbers(&[width, height]),
            }
        }
        fn from_list<N: Copy + Into<f64>>(list: &[N]) -> Result<Size, SizeError> {
            let numbers: Vec<f64> = list.iter().map(|&number| number.into()).collect();
            Size::from_numbers(&numbers)
        }
        match value {
            PySize::Size(size) => Ok(size),
            PySize::ListF64(list) => from_list(&list),
            PySize::ListU32(list) => from_list(&list),
            PySize::ListI32(list) => from_list(&list),
            PySize::TupleF64((w, h)) => from_list(&[w, h]),
            PySize::TupleU32((w, h)) => from_list(&[w, h]),
            PySize::TupleI32((w, h)) => from_list(&[w, h]),
            PySize::Tuple3F64((w, h, d)) => from_list(&[w, h, d]),
            PySize::Tuple3U32((w, h, d)) => from_list(&[w, h, d]),
            PySize::Tuple3I32((w, h, d)) => from_list(&[w, h, d]),
            PySize::DictF64(map) => from_map(&map),
            PySize::DictU32(map) => from_map(&map),
            PySize::DictI32(map) => from_map(&map),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    #[cfg(python)]
    use std::collections::HashMap;

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

    // Python-specific conversion tests (compile when feature=python)
    #[cfg(python)]
    #[test]
    fn py_size_conversions_cover_variants() {
        let size = |value: PySize| Size::try_from(value).expect("valid size");

        // A Size passes through unchanged
        assert_eq!(size(PySize::Size(Size::new(5, 6, Some(2)))), Size::new(5, 6, Some(2)));

        // Tuples, with fractions rounded down
        assert_eq!(size(PySize::TupleF64((10.5, 20.5))), Size::new(10, 20, None));
        assert_eq!(size(PySize::TupleU32((7, 9))), Size::new(7, 9, None));
        assert_eq!(size(PySize::Tuple3I32((3, 4, 5))), Size::new(3, 4, Some(5)));

        // Lists
        assert_eq!(size(PySize::ListU32(vec![1, 2])), Size::new(1, 2, None));
        assert_eq!(size(PySize::ListF64(vec![2.0, 3.0, 4.0])), Size::new(2, 3, Some(4)));

        // Dicts
        let mut d_f: HashMap<String, f64> = HashMap::new();
        d_f.insert("width".into(), 11.0);
        d_f.insert("height".into(), 12.0);
        d_f.insert("depth".into(), 13.0);
        assert_eq!(size(PySize::DictF64(d_f)), Size::new(11, 12, Some(13)));

        let mut d_u: HashMap<String, u32> = HashMap::new();
        d_u.insert("width".into(), 21);
        d_u.insert("height".into(), 22);
        assert_eq!(size(PySize::DictU32(d_u)), Size::new(21, 22, None));
    }

    // Story: a Python size of the wrong length, a negative or NaN dimension,
    // or a dict without a width raises instead of turning into 0 x 0.
    #[cfg(python)]
    #[test]
    fn py_size_rejects_invalid_input() {
        assert!(Size::try_from(PySize::ListU32(vec![1, 2, 3, 4])).is_err());
        assert!(Size::try_from(PySize::ListI32(vec![-1, 2])).is_err());
        assert!(Size::try_from(PySize::TupleF64((f64::NAN, 2.0))).is_err());
        let mut no_width: HashMap<String, u32> = HashMap::new();
        no_width.insert("height".into(), 2);
        assert!(Size::try_from(PySize::DictU32(no_width)).is_err());
    }

    // Story: the shared rule takes 2 or 3 finite numbers from 0 up.
    #[cfg(python)]
    #[test]
    fn from_numbers_applies_one_rule() {
        assert_eq!(Size::from_numbers(&[4.0, 2.0]).ok(), Some(Size::new(4, 2, None)));
        assert_eq!(Size::from_numbers(&[4.9, 2.0, 1.0]).ok(), Some(Size::new(4, 2, Some(1))));
        assert!(Size::from_numbers(&[4.0]).is_err());
        assert!(Size::from_numbers(&[-1.0, 2.0]).is_err());
        assert!(Size::from_numbers(&[f64::INFINITY, 2.0]).is_err());
    }
}
