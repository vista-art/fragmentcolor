#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd)]
pub struct Size {
    pub width: u32,
    pub height: u32,
    pub depth: Option<u32>,
}

#[cfg_attr(wasm, wasm_bindgen)]
impl Size {
    #[cfg_attr(wasm, wasm_bindgen(constructor))]
    #[cfg_attr(python, pyo3::new)]
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
