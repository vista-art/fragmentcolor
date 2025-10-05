#[cfg(wasm)]
use wasm_bindgen::JsCast;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg(python)]
use pyo3::FromPyObject;
#[cfg(python)]
use pyo3::prelude::*;

pub mod error;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd)]
pub struct Size {
    pub width: u32,
    pub height: u32,
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

#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for Size {
    type Error = crate::size::error::SizeError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::{Array, Float32Array, Int32Array, Reflect, Uint32Array};

        if let Some(arr) = value.dyn_ref::<Uint32Array>() {
            let len = arr.length();
            if len == 2 {
                let mut buf = [0u32; 2];
                arr.copy_to(&mut buf);
                return Ok(Size::new(buf[0], buf[1], None));
            }
            if len == 3 {
                let mut buf = [0u32; 3];
                arr.copy_to(&mut buf);
                return Ok(Size::new(buf[0], buf[1], Some(buf[2])));
            }
        }
        if let Some(arr) = value.dyn_ref::<Int32Array>() {
            let len = arr.length();
            if len == 2 {
                let mut buf = [0i32; 2];
                arr.copy_to(&mut buf);
                return Ok(Size::new(buf[0].max(0) as u32, buf[1].max(0) as u32, None));
            }
            if len == 3 {
                let mut buf = [0i32; 3];
                arr.copy_to(&mut buf);
                return Ok(Size::new(
                    buf[0].max(0) as u32,
                    buf[1].max(0) as u32,
                    Some(buf[2].max(0) as u32),
                ));
            }
        }
        if let Some(arr) = value.dyn_ref::<Float32Array>() {
            let len = arr.length();
            if len == 2 {
                let mut buf = [0.0f32; 2];
                arr.copy_to(&mut buf);
                return Ok(Size::new(
                    buf[0].max(0.0) as u32,
                    buf[1].max(0.0) as u32,
                    None,
                ));
            }
            if len == 3 {
                let mut buf = [0.0f32; 3];
                arr.copy_to(&mut buf);
                return Ok(Size::new(
                    buf[0].max(0.0) as u32,
                    buf[1].max(0.0) as u32,
                    Some(buf[2].max(0.0) as u32),
                ));
            }
        }

        if let Some(arr) = value.dyn_ref::<Array>() {
            let len = arr.length();
            let num_at = |i: u32| arr.get(i).as_f64().unwrap_or(0.0);
            if len == 2 {
                let w = num_at(0).max(0.0) as u32;
                let h = num_at(1).max(0.0) as u32;
                return Ok(Size::new(w, h, None));
            }
            if len == 3 {
                let w = num_at(0).max(0.0) as u32;
                let h = num_at(1).max(0.0) as u32;
                let d = num_at(2).max(0.0) as u32;
                return Ok(Size::new(w, h, Some(d)));
            }
        }

        if value.is_object() {
            let width = Reflect::get(&value, &JsValue::from_str("width"))
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as u32;
            let height = Reflect::get(&value, &JsValue::from_str("height"))
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as u32;
            let depth = Reflect::get(&value, &JsValue::from_str("depth"))
                .ok()
                .and_then(|v| v.as_f64())
                .map(|v| v as u32);

            if width != 0 && height != 0 {
                return Ok(Size::new(width, height, depth));
            }
        }

        Err(crate::size::error::SizeError::TypeMismatch(
            "Cannot convert JavaScript value to Size (expected [w,h] or [w,h,d])".into(),
        ))
    }
}

#[cfg(wasm)]
crate::impl_tryfrom_owned_via_ref!(Size, wasm_bindgen::JsValue, crate::size::error::SizeError);

#[cfg(python)]
#[derive(FromPyObject, IntoPyObject)]
pub enum PySize {
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

#[cfg(python)]
impl From<PySize> for Size {
    fn from(value: PySize) -> Self {
        match value {
            PySize::TupleF64((w, h)) => Size::new(w as u32, h as u32, None),
            PySize::TupleU32((w, h)) => Size::new(w, h, None),
            PySize::TupleI32((w, h)) => Size::new(w as u32, h as u32, None),
            PySize::Tuple3F64((w, h, d)) => Size::new(w as u32, h as u32, Some(d as u32)),
            PySize::Tuple3U32((w, h, d)) => Size::new(w, h, Some(d)),
            PySize::Tuple3I32((w, h, d)) => Size::new(w as u32, h as u32, Some(d as u32)),
            PySize::ListF64(vals) => match vals.as_slice() {
                [w, h] => Size::new(*w as u32, *h as u32, None),
                [w, h, d] => Size::new(*w as u32, *h as u32, Some(*d as u32)),
                _ => Size::default(),
            },
            PySize::ListU32(vals) => match vals.as_slice() {
                [w, h] => Size::new(*w, *h, None),
                [w, h, d] => Size::new(*w, *h, Some(*d)),
                _ => Size::default(),
            },
            PySize::ListI32(vals) => match vals.as_slice() {
                [w, h] => Size::new(*w as u32, *h as u32, None),
                [w, h, d] => Size::new(*w as u32, *h as u32, Some(*d as u32)),
                _ => Size::default(),
            },
            PySize::DictF64(map) => {
                let w = map.get("width").copied().unwrap_or(0.0) as u32;
                let h = map.get("height").copied().unwrap_or(0.0) as u32;
                let d = map.get("depth").copied().map(|v| v as u32);
                Size::new(w, h, d)
            }
            PySize::DictU32(map) => {
                let w = map.get("width").copied().unwrap_or(0);
                let h = map.get("height").copied().unwrap_or(0);
                let d = map.get("depth").copied();
                Size::new(w, h, d)
            }
            PySize::DictI32(map) => {
                let w = map.get("width").copied().unwrap_or(0) as u32;
                let h = map.get("height").copied().unwrap_or(0) as u32;
                let d = map.get("depth").copied().map(|v| v as u32);
                Size::new(w, h, d)
            }
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
        // TupleF64 and TupleU32
        let s1: Size = PySize::TupleF64((10.5, 20.5)).into();
        assert_eq!((s1.width, s1.height, s1.depth), (10, 20, None));
        let s2: Size = PySize::TupleU32((7, 9)).into();
        assert_eq!((s2.width, s2.height, s2.depth), (7, 9, None));

        // Tuple3I32 with negative depth coerced via cast
        let s3: Size = PySize::Tuple3I32((3, 4, 5)).into();
        assert_eq!((s3.width, s3.height, s3.depth), (3, 4, Some(5)));

        // Lists
        let s4: Size = PySize::ListU32(vec![1, 2]).into();
        assert_eq!((s4.width, s4.height, s4.depth), (1, 2, None));
        let s5: Size = PySize::ListF64(vec![2.0, 3.0, 4.0]).into();
        assert_eq!((s5.width, s5.height, s5.depth), (2, 3, Some(4)));

        // Dicts
        let mut d_f: HashMap<String, f64> = HashMap::new();
        d_f.insert("width".into(), 11.0);
        d_f.insert("height".into(), 12.0);
        d_f.insert("depth".into(), 13.0);
        let s6: Size = PySize::DictF64(d_f).into();
        assert_eq!((s6.width, s6.height, s6.depth), (11, 12, Some(13)));

        let mut d_u: HashMap<String, u32> = HashMap::new();
        d_u.insert("width".into(), 21);
        d_u.insert("height".into(), 22);
        let s7: Size = PySize::DictU32(d_u).into();
        assert_eq!((s7.width, s7.height, s7.depth), (21, 22, None));
    }
}
