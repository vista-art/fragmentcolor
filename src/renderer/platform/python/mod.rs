use crate::{
    PySize, Renderer,
    target::{PyTextureTarget, RenderCanvasTarget},
};
use lsp_doc::lsp_doc;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

pub mod iterator;
pub use iterator::*;

pub mod renderable;
pub use renderable::*;

pub(crate) mod handle;
pub(crate) use handle::*;

#[pymethods]
impl Renderer {
    #[new]
    #[lsp_doc("docs/api/core/renderer/new.md")]
    pub fn new_py() -> Renderer {
        Self::new()
    }

    #[lsp_doc("docs/api/core/renderer/create_target.md")]
    #[pyo3(name = "create_target")]
    pub fn create_target_py(
        &self,
        rendercanvas: Py<PyAny>,
    ) -> Result<Py<RenderCanvasTarget>, PyErr> {
        Python::attach(|py| -> Result<Py<RenderCanvasTarget>, PyErr> {
            // If the target is already initialized, return it
            let libname = PyTuple::new(py, ["fragmentcolor"])?;
            let py_target = rendercanvas.call_method1(py, "get_context", libname)?; // calls hook
            let bound_target = py_target.downcast_bound::<RenderCanvasTarget>(py)?;
            let mut target = bound_target.borrow_mut();
            if target.is_ready() {
                return Ok(target.into_pyobject(py)?.unbind());
            }

            // Returns a list of the possible present methods ("screen", "bitmap")
            let present_methods = rendercanvas.call_method0(py, "_rc_get_present_methods")?;

            // Gets the screen info dictionary (window, platform, display)
            let dict = present_methods
                .downcast_bound::<PyDict>(py)?
                .get_item("screen")?
                .ok_or(crate::error::PyFragmentColorError::new_err(
                    "Object can't render to screen",
                ))?;
            let screen_info = dict.downcast::<PyDict>()?;

            // Mandatory WindowHandle for all platforms
            let window: u64 = screen_info
                .get_item("window")?
                .ok_or(crate::error::PyFragmentColorError::new_err(
                    "Missing window handle",
                ))?
                .extract()?;
            // Optional platform and display (only present on Linux)
            let platform: String = screen_info
                .get_item("platform")?
                .unwrap_or("".into_pyobject(py)?.into_any())
                .extract()?;
            let display: u64 = screen_info
                .get_item("display")?
                .unwrap_or(0u64.into_pyobject(py)?.into_any())
                .extract()?;

            // Gets the window size to configure the surface
            let size: (u32, u32) = rendercanvas
                .call_method0(py, "get_physical_size")?
                .downcast_bound(py)?
                .extract()?;

            let size = wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            };

            let handles: WindowHandles = create_raw_handles(&platform, window, Some(display))?;

            let (context, surface, config) =
                pollster::block_on(self.create_surface(handles, size))?;
            target.init(context, surface, config);

            Ok(target.into_pyobject(py)?.unbind())
        })
    }

    #[lsp_doc("docs/api/core/renderer/create_texture_target.md")]
    #[pyo3(name = "create_texture_target")]
    pub fn create_texture_target_py(&self, size: PySize) -> Result<Py<PyTextureTarget>, PyErr> {
        Python::attach(|py| -> Result<Py<PyTextureTarget>, PyErr> {
            let target = pollster::block_on(self.create_texture_target(size))?;
            let py_target: PyTextureTarget = target.into();
            Ok(py_target.into_pyobject(py)?.unbind())
        })
    }

    /// Python: Create a Texture from bytes, path string, or numpy ndarray (H,W[,C]).
    #[lsp_doc("docs/api/core/renderer/create_texture.md")]
    #[pyo3(name = "create_texture")]
    pub fn create_texture_py(&self, input: Py<PyAny>) -> Result<crate::texture::Texture, PyErr> {
        Python::attach(|py| -> Result<crate::texture::Texture, PyErr> {
            // list of bytes
            if let Ok(pylist) = input.bind(py).extract::<Py<pyo3::types::PyBytes>>() {
                let vec: Vec<u8> = pylist.extract(py)?;
                let tex = pollster::block_on(self.create_texture(&vec))?;
                return Ok(tex);
            }
            // Python list
            if let Ok(pylist) = input.bind(py).extract::<Py<pyo3::types::PyList>>() {
                let vec: Vec<u8> = pylist.extract(py)?;
                let tex = pollster::block_on(self.create_texture(&vec))?;
                return Ok(tex);
            }
            // str path
            if let Ok(path) = input.bind(py).extract::<String>() {
                let tex = pollster::block_on(self.create_texture(std::path::Path::new(&path)))?;
                return Ok(tex);
            }
            // numpy ndarray to RGBA8
            {
                if let Ok(any) = input.bind(py).downcast::<numpy::PyArrayDyn<u8>>() {
                    use numpy::{PyArrayMethods, PyUntypedArrayMethods};

                    let arr = any.readonly();
                    let shape = arr.shape();
                    if shape.len() == 2 || shape.len() == 3 {
                        let h = shape[0] as u32;
                        let w = shape[1] as u32;
                        let c = if shape.len() == 3 { shape[2] } else { 1 };
                        let mut rgba = vec![0u8; (w as usize) * (h as usize) * 4];
                        let data = arr.as_slice().map_err(|_| {
                            crate::error::PyFragmentColorError::new_err(
                                "ndarray must be contiguous",
                            )
                        })?;
                        for y in 0..(h as usize) {
                            for x in 0..(w as usize) {
                                let src_idx = (y * (w as usize) + x) * c;
                                let dst_idx = (y * (w as usize) + x) * 4;
                                match c {
                                    1 => {
                                        let v = data[src_idx];
                                        rgba[dst_idx..dst_idx + 4].copy_from_slice(&[v, v, v, 255]);
                                    }
                                    3 => {
                                        let r = data[src_idx];
                                        let g = data[src_idx + 1];
                                        let b = data[src_idx + 2];
                                        rgba[dst_idx..dst_idx + 4].copy_from_slice(&[r, g, b, 255]);
                                    }
                                    4 => {
                                        rgba[dst_idx..dst_idx + 4]
                                            .copy_from_slice(&data[src_idx..src_idx + 4]);
                                    }
                                    _ => {
                                        return Err(crate::error::PyFragmentColorError::new_err(
                                            "ndarray last dimension must be 1, 3, or 4",
                                        ));
                                    }
                                }
                            }
                        }
                        let size = crate::Size::new(w, h, None);
                        let options = crate::TextureOptions {
                            size: Some(size),
                            ..Default::default()
                        };
                        let tex = pollster::block_on(self.create_texture_with(
                            crate::texture::TextureInput::Bytes(rgba),
                            options,
                        ))?;
                        return Ok(tex);
                    }
                }
            }
            Err(crate::error::PyFragmentColorError::new_err(
                "Unsupported input for create_texture (expected bytes, str path, or numpy ndarray)",
            ))
        })
    }

    #[pyo3(name = "render")]
    #[lsp_doc("docs/api/core/renderer/render.md")]
    pub fn render_py(&self, renderable: Py<PyAny>, target: Py<PyAny>) -> Result<(), PyErr> {
        Python::attach(|py| -> Result<(), PyErr> {
            // Convert any supported Python input (single object or sequence) into a PyRenderable
            let r = crate::PyRenderable::from_any(renderable.bind(py))?;

            // Downcast target to supported targets
            if let Ok(bound) = target.bind(py).downcast::<RenderCanvasTarget>() {
                self.render(&r, &*bound.borrow())?;
                Ok(())
            } else if let Ok(bound) = target.bind(py).downcast::<PyTextureTarget>() {
                self.render(&r, &*bound.borrow())?;
                Ok(())
            } else {
                Err(PyErr::new::<PyTypeError, _>(
                    "Unsupported target type. Expected RenderCanvasTarget or TextureTarget",
                ))
            }
        })?;

        Ok(())
    }

    // Depth texture
    #[pyo3(name = "create_depth_texture")]
    #[lsp_doc("docs/api/core/renderer/hidden/create_depth_texture_py.md")]
    pub fn create_depth_texture_py(&self, size: PySize) -> Result<crate::texture::Texture, PyErr> {
        Python::attach(|_py| -> Result<crate::texture::Texture, PyErr> {
            let tex = pollster::block_on(self.create_depth_texture(size))?;
            Ok(tex)
        })
    }

    // Storage texture
    #[pyo3(name = "create_storage_texture")]
    #[lsp_doc("docs/api/core/renderer/hidden/create_storage_texture_py.md")]
    pub fn create_storage_texture_py(
        &self,
        size: PySize,
        format: crate::TextureFormat,
        usage_bits: Option<u32>,
    ) -> Result<crate::texture::Texture, PyErr> {
        Python::attach(|_py| -> Result<crate::texture::Texture, PyErr> {
            let usage = usage_bits.map(wgpu::TextureUsages::from_bits_truncate);
            let tex = pollster::block_on(self.create_storage_texture(size, format, usage))?;
            Ok(tex)
        })
    }

    // Texture creation with helpers
    #[pyo3(name = "create_texture_with_size")]
    #[lsp_doc("docs/api/core/renderer/hidden/create_texture_with_size_py.md")]
    pub fn create_texture_with_size_py(
        &self,
        input: Py<PyAny>,
        size: PySize,
    ) -> Result<crate::texture::Texture, PyErr> {
        Python::attach(|py| -> Result<crate::texture::Texture, PyErr> {
            // Byte array
            if let Ok(pylist) = input.bind(py).extract::<Py<pyo3::types::PyBytes>>() {
                let vec: Vec<u8> = pylist.extract(py)?;
                let tex = pollster::block_on(self.create_texture_with_size(&vec, size))?;
                return Ok(tex);
            }
            // list of bytes
            if let Ok(pylist) = input.bind(py).extract::<Py<pyo3::types::PyList>>() {
                let vec: Vec<u8> = pylist.extract(py)?;
                let tex = pollster::block_on(self.create_texture_with_size(&vec, size))?;
                return Ok(tex);
            }
            // str path
            if let Ok(path) = input.bind(py).extract::<String>() {
                let tex = pollster::block_on(
                    self.create_texture_with_size(std::path::Path::new(&path), size),
                )?;
                return Ok(tex);
            }
            // numpy ndarray -> RGBA8
            if let Ok(any) = input.bind(py).downcast::<numpy::PyArrayDyn<u8>>() {
                use numpy::{PyArrayMethods, PyUntypedArrayMethods};
                let arr = any.readonly();
                let shape = arr.shape();
                if shape.len() == 2 || shape.len() == 3 {
                    let h = shape[0] as u32;
                    let w = shape[1] as u32;
                    let c = if shape.len() == 3 { shape[2] } else { 1 };
                    let mut rgba = vec![0u8; (w as usize) * (h as usize) * 4];
                    let data = arr.as_slice().map_err(|_| {
                        crate::error::PyFragmentColorError::new_err("ndarray must be contiguous")
                    })?;
                    for y in 0..(h as usize) {
                        for x in 0..(w as usize) {
                            let src_idx = (y * (w as usize) + x) * c;
                            let dst_idx = (y * (w as usize) + x) * 4;
                            match c {
                                1 => {
                                    let v = data[src_idx];
                                    rgba[dst_idx..dst_idx + 4].copy_from_slice(&[v, v, v, 255]);
                                }
                                3 => {
                                    let r = data[src_idx];
                                    let g = data[src_idx + 1];
                                    let b = data[src_idx + 2];
                                    rgba[dst_idx..dst_idx + 4].copy_from_slice(&[r, g, b, 255]);
                                }
                                4 => {
                                    rgba[dst_idx..dst_idx + 4]
                                        .copy_from_slice(&data[src_idx..src_idx + 4]);
                                }
                                _ => {
                                    return Err(crate::error::PyFragmentColorError::new_err(
                                        "ndarray last dimension must be 1, 3, or 4",
                                    ));
                                }
                            }
                        }
                    }
                    let tex = pollster::block_on(self.create_texture_with_size(&rgba, size))?;
                    return Ok(tex);
                }
            }
            Err(crate::error::PyFragmentColorError::new_err(
                "Unsupported input for create_texture_with_size (expected bytes, str path, or numpy ndarray)",
            ))
        })
    }

    #[pyo3(name = "create_texture_with_format")]
    #[lsp_doc("docs/api/core/renderer/hidden/create_texture_with_format_py.md")]
    pub fn create_texture_with_format_py(
        &self,
        input: Py<PyAny>,
        format: crate::texture::TextureFormat,
    ) -> Result<crate::texture::Texture, PyErr> {
        Python::attach(|py| -> Result<crate::texture::Texture, PyErr> {
            // Byte array
            if let Ok(pylist) = input.bind(py).extract::<Py<pyo3::types::PyBytes>>() {
                let vec: Vec<u8> = pylist.extract(py)?;
                let tex = pollster::block_on(self.create_texture_with_format(&vec, format))?;
                return Ok(tex);
            }
            // Python list
            if input.bind(py).extract::<Py<pyo3::types::PyList>>().is_ok() {
                let pylist = input.bind(py).extract::<Py<pyo3::types::PyList>>()?;
                let vec: Vec<u8> = pylist.extract(py)?;
                let tex = pollster::block_on(self.create_texture_with_format(&vec, format))?;
                return Ok(tex);
            }
            // str path
            if let Ok(path) = input.bind(py).extract::<String>() {
                // path decoding ignores explicit format; load image and upload
                let tex = pollster::block_on(self.create_texture(std::path::Path::new(&path)))?;
                return Ok(tex);
            }
            // numpy ndarray -> RGBA8, infer size
            if let Ok(any) = input.bind(py).downcast::<numpy::PyArrayDyn<u8>>() {
                use numpy::{PyArrayMethods, PyUntypedArrayMethods};
                let arr = any.readonly();
                let shape = arr.shape();
                if shape.len() == 2 || shape.len() == 3 {
                    let h = shape[0] as u32;
                    let w = shape[1] as u32;
                    let c = if shape.len() == 3 { shape[2] } else { 1 };
                    let mut rgba = vec![0u8; (w as usize) * (h as usize) * 4];
                    let data = arr.as_slice().map_err(|_| {
                        crate::error::PyFragmentColorError::new_err("ndarray must be contiguous")
                    })?;
                    for y in 0..(h as usize) {
                        for x in 0..(w as usize) {
                            let src_idx = (y * (w as usize) + x) * c;
                            let dst_idx = (y * (w as usize) + x) * 4;
                            match c {
                                1 => {
                                    let v = data[src_idx];
                                    rgba[dst_idx..dst_idx + 4].copy_from_slice(&[v, v, v, 255]);
                                }
                                3 => {
                                    let r = data[src_idx];
                                    let g = data[src_idx + 1];
                                    let b = data[src_idx + 2];
                                    rgba[dst_idx..dst_idx + 4].copy_from_slice(&[r, g, b, 255]);
                                }
                                4 => {
                                    rgba[dst_idx..dst_idx + 4]
                                        .copy_from_slice(&data[src_idx..src_idx + 4]);
                                }
                                _ => {
                                    return Err(crate::error::PyFragmentColorError::new_err(
                                        "ndarray last dimension must be 1, 3, or 4",
                                    ));
                                }
                            }
                        }
                    }
                    // Use create_texture_with to specify both size and format explicitly
                    let options = crate::TextureOptions {
                        size: Some(crate::Size::new(w, h, None)),
                        format,
                        sampler: crate::texture::SamplerOptions::default(),
                    };
                    let tex = pollster::block_on(self.create_texture_with(&rgba, options))?;
                    return Ok(tex);
                }
            }
            Err(crate::error::PyFragmentColorError::new_err(
                "Unsupported input for create_texture_with_format (expected str path or numpy ndarray)",
            ))
        })
    }

    #[pyo3(name = "create_texture_with")]
    #[lsp_doc("docs/api/core/renderer/hidden/create_texture_with_py.md")]
    pub fn create_texture_with_py(
        &self,
        input: Py<PyAny>,
        size: Option<PySize>,
    ) -> Result<crate::texture::Texture, PyErr> {
        Python::attach(|py| -> Result<crate::texture::Texture, PyErr> {
            // list of bytes
            if let Ok(pylist) = input.bind(py).extract::<Py<pyo3::types::PyList>>() {
                let vec: Vec<u8> = pylist.extract(py)?;
                if let Some(s) = size {
                    let options = crate::TextureOptions {
                        size: Some(s.into()),
                        ..Default::default()
                    };
                    let tex = pollster::block_on(self.create_texture_with(&vec, options))?;
                    return Ok(tex);
                }
                let tex = pollster::block_on(self.create_texture(&vec))?;
                return Ok(tex);
            }
            // str path
            if let Ok(path) = input.bind(py).extract::<String>() {
                if let Some(s) = size {
                    let tex = pollster::block_on(
                        self.create_texture_with_size(std::path::Path::new(&path), s),
                    )?;
                    return Ok(tex);
                }
                let tex = pollster::block_on(self.create_texture(std::path::Path::new(&path)))?;
                return Ok(tex);
            }
            // numpy ndarray -> RGBA8
            if let Ok(any) = input.bind(py).downcast::<numpy::PyArrayDyn<u8>>() {
                use numpy::{PyArrayMethods, PyUntypedArrayMethods};
                let arr = any.readonly();
                let shape = arr.shape();
                if shape.len() == 2 || shape.len() == 3 {
                    let h = shape[0] as u32;
                    let w = shape[1] as u32;
                    let c = if shape.len() == 3 { shape[2] } else { 1 };
                    let mut rgba = vec![0u8; (w as usize) * (h as usize) * 4];
                    let data = arr.as_slice().map_err(|_| {
                        crate::error::PyFragmentColorError::new_err("ndarray must be contiguous")
                    })?;
                    for y in 0..(h as usize) {
                        for x in 0..(w as usize) {
                            let src_idx = (y * (w as usize) + x) * c;
                            let dst_idx = (y * (w as usize) + x) * 4;
                            match c {
                                1 => {
                                    let v = data[src_idx];
                                    rgba[dst_idx..dst_idx + 4].copy_from_slice(&[v, v, v, 255]);
                                }
                                3 => {
                                    let r = data[src_idx];
                                    let g = data[src_idx + 1];
                                    let b = data[src_idx + 2];
                                    rgba[dst_idx..dst_idx + 4].copy_from_slice(&[r, g, b, 255]);
                                }
                                4 => {
                                    rgba[dst_idx..dst_idx + 4]
                                        .copy_from_slice(&data[src_idx..src_idx + 4]);
                                }
                                _ => {
                                    return Err(crate::error::PyFragmentColorError::new_err(
                                        "ndarray last dimension must be 1, 3, or 4",
                                    ));
                                }
                            }
                        }
                    }
                    if let Some(s) = size {
                        let options = crate::TextureOptions {
                            size: Some(s.into()),
                            ..Default::default()
                        };
                        let tex = pollster::block_on(self.create_texture_with(&rgba, options))?;
                        return Ok(tex);
                    }
                    let tex = pollster::block_on(
                        self.create_texture_with_size(&rgba, crate::Size::new(w, h, None)),
                    )?;
                    return Ok(tex);
                }
            }
            Err(crate::error::PyFragmentColorError::new_err(
                "Unsupported input for create_texture_with (expected bytes, str path, or numpy ndarray)",
            ))
        })
    }
}
