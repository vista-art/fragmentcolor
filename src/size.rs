#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg(python)]
use pyo3::FromPyObject;
#[cfg(python)]
use pyo3::Py;
#[cfg(python)]
use pyo3::prelude::*;

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

impl From<wgpu::Extent3d> for Size {
    fn from(extent: wgpu::Extent3d) -> Self {
        Self {
            width: extent.width,
            height: extent.height,
            depth: Some(extent.depth_or_array_layers),
        }
    }
}

impl From<Size> for wgpu::Extent3d {
    fn from(size: Size) -> Self {
        Self {
            width: size.width,
            height: size.height,
            depth_or_array_layers: size.depth.unwrap_or(1),
        }
    }
}

impl From<(u32, u32)> for Size {
    fn from(value: (u32, u32)) -> Self {
        Self {
            width: value.0,
            height: value.1,
            depth: None,
        }
    }
}

impl From<Size> for (u32, u32) {
    fn from(size: Size) -> Self {
        (size.width, size.height)
    }
}

impl From<&(u32, u32)> for Size {
    fn from(value: &(u32, u32)) -> Self {
        Self {
            width: value.0,
            height: value.1,
            depth: None,
        }
    }
}

impl From<&Size> for (u32, u32) {
    fn from(size: &Size) -> Self {
        (size.width, size.height)
    }
}

impl From<(u32, u32, u32)> for Size {
    fn from(value: (u32, u32, u32)) -> Self {
        Self {
            width: value.0,
            height: value.1,
            depth: Some(value.2),
        }
    }
}

impl From<Size> for (u32, u32, u32) {
    fn from(size: Size) -> Self {
        (size.width, size.height, size.depth.unwrap_or(1))
    }
}

impl From<&(u32, u32, u32)> for Size {
    fn from(value: &(u32, u32, u32)) -> Self {
        Self {
            width: value.0,
            height: value.1,
            depth: Some(value.2),
        }
    }
}

impl From<&Size> for (u32, u32, u32) {
    fn from(size: &Size) -> Self {
        (size.width, size.height, size.depth.unwrap_or(1))
    }
}

impl From<[u32; 2]> for Size {
    fn from(value: [u32; 2]) -> Self {
        Self {
            width: value[0],
            height: value[1],
            depth: None,
        }
    }
}

impl From<Size> for [u32; 2] {
    fn from(size: Size) -> Self {
        [size.width, size.height]
    }
}

impl From<&[u32; 2]> for Size {
    fn from(value: &[u32; 2]) -> Self {
        Self {
            width: value[0],
            height: value[1],
            depth: None,
        }
    }
}

impl From<&Size> for [u32; 2] {
    fn from(size: &Size) -> Self {
        [size.width, size.height]
    }
}

impl From<[u32; 3]> for Size {
    fn from(value: [u32; 3]) -> Self {
        Self {
            width: value[0],
            height: value[1],
            depth: Some(value[2]),
        }
    }
}

impl From<Size> for [u32; 3] {
    fn from(size: Size) -> Self {
        [size.width, size.height, size.depth.unwrap_or(1)]
    }
}

impl From<&[u32; 3]> for Size {
    fn from(value: &[u32; 3]) -> Self {
        Self {
            width: value[0],
            height: value[1],
            depth: Some(value[2]),
        }
    }
}

impl From<&Size> for [u32; 3] {
    fn from(size: &Size) -> Self {
        [size.width, size.height, size.depth.unwrap_or(1)]
    }
}

#[cfg(python)]
#[derive(FromPyObject, IntoPyObject)]
pub enum PySize {
    List_f64(Vec<f64>),
    List_u32(Vec<u32>),
    List_i32(Vec<i32>),
    Tuple_f64((f64, f64)),
    Tuple_u32((u32, u32)),
    Tuple_i32((i32, i32)),
    Tuple3_f64((f64, f64, f64)),
    Tuple3_u32((u32, u32, u32)),
    Tuple3_i32((i32, i32, i32)),
    Dict_f64(std::collections::HashMap<String, f64>),
    Dict_u32(std::collections::HashMap<String, u32>),
    Dict_i32(std::collections::HashMap<String, i32>),
}

#[cfg(python)]
impl From<PySize> for Size {
    fn from(value: PySize) -> Self {
        match value {
            PySize::Tuple_f64((w, h)) => Size::new(w as u32, h as u32, None),
            PySize::Tuple_u32((w, h)) => Size::new(w, h, None),
            PySize::Tuple_i32((w, h)) => Size::new(w as u32, h as u32, None),
            PySize::Tuple3_f64((w, h, d)) => Size::new(w as u32, h as u32, Some(d as u32)),
            PySize::Tuple3_u32((w, h, d)) => Size::new(w, h, Some(d)),
            PySize::Tuple3_i32((w, h, d)) => Size::new(w as u32, h as u32, Some(d as u32)),
            PySize::List_f64(vals) => match vals.as_slice() {
                [w, h] => Size::new(*w as u32, *h as u32, None),
                [w, h, d] => Size::new(*w as u32, *h as u32, Some(*d as u32)),
                _ => Size::default(),
            },
            PySize::List_u32(vals) => match vals.as_slice() {
                [w, h] => Size::new(*w, *h, None),
                [w, h, d] => Size::new(*w, *h, Some(*d)),
                _ => Size::default(),
            },
            PySize::List_i32(vals) => match vals.as_slice() {
                [w, h] => Size::new(*w as u32, *h as u32, None),
                [w, h, d] => Size::new(*w as u32, *h as u32, Some(*d as u32)),
                _ => Size::default(),
            },
            PySize::Dict_f64(map) => {
                let w = map.get("width").copied().unwrap_or(0.0) as u32;
                let h = map.get("height").copied().unwrap_or(0.0) as u32;
                let d = map.get("depth").copied().map(|v| v as u32);
                Size::new(w, h, d)
            }
            PySize::Dict_u32(map) => {
                let w = map.get("width").copied().unwrap_or(0);
                let h = map.get("height").copied().unwrap_or(0);
                let d = map.get("depth").copied();
                Size::new(w, h, d)
            }
            PySize::Dict_i32(map) => {
                let w = map.get("width").copied().unwrap_or(0) as u32;
                let h = map.get("height").copied().unwrap_or(0) as u32;
                let d = map.get("depth").copied().map(|v| v as u32);
                Size::new(w, h, d)
            }
        }
    }
}
