use crate::{
    PySize, Renderer,
    target::{PyTextureTarget, RenderCanvasTarget},
};
use lsp_doc::lsp_doc;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

pub mod iterator;
pub use iterator::*;

pub mod renderable;
pub use renderable::*;

pub(crate) mod handle;
pub(crate) use handle::*;

fn get_render_canvas_target(
    py: Python<'_>,
    canvas: &Py<PyAny>,
) -> Result<Py<RenderCanvasTarget>, PyErr> {
    let bound = canvas.bind(py);
    if let Ok(existing) = bound.getattr("_fragmentcolor_target")
        && let Ok(target) = existing.cast::<RenderCanvasTarget>()
    {
        return Ok(target.clone().unbind());
    }

    let none = py.None();
    let target = Py::new(py, RenderCanvasTarget::new(canvas.clone_ref(py), none))?;
    bound.setattr("_fragmentcolor_target", &target)?;
    Ok(target)
}

fn get_screen_info<'py>(
    canvas: &pyo3::Bound<'py, PyAny>,
) -> Result<pyo3::Bound<'py, PyDict>, PyErr> {
    let info = canvas.call_method1("_rc_get_present_info", (vec!["screen"],))?;
    if info.is_none() {
        return Err(crate::error::PyFragmentColorError::new_err(
            "Object can't render to screen",
        ));
    }

    let dict = info.cast_into::<PyDict>()?;
    let method = dict
        .get_item("method")?
        .ok_or(crate::error::PyFragmentColorError::new_err(
            "Missing present method",
        ))?
        .extract::<String>()?;
    if method != "screen" {
        return Err(crate::error::PyFragmentColorError::new_err(
            "Object can't render to screen",
        ));
    }

    Ok(dict)
}

fn get_required_u64(dict: &pyo3::Bound<'_, PyDict>, name: &str) -> Result<u64, PyErr> {
    dict.get_item(name)?
        .ok_or(crate::error::PyFragmentColorError::new_err(format!(
            "Missing {name} handle",
        )))?
        .extract::<u64>()
}

fn get_optional_u64(dict: &pyo3::Bound<'_, PyDict>, name: &str) -> Result<u64, PyErr> {
    if let Some(value) = dict.get_item(name)? {
        value.extract::<u64>()
    } else {
        Ok(0)
    }
}

fn get_optional_string(dict: &pyo3::Bound<'_, PyDict>, name: &str) -> Result<String, PyErr> {
    if let Some(value) = dict.get_item(name)? {
        value.extract::<String>()
    } else {
        Ok(String::new())
    }
}

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
            let target = get_render_canvas_target(py, &rendercanvas)?;
            if target.bind(py).borrow().is_ready() {
                return Ok(target);
            }
            {
                let mut target_ref = target.bind(py).borrow_mut();
                let canvas = rendercanvas.bind(py);
                let screen_info = get_screen_info(canvas)?;
                let window = get_required_u64(&screen_info, "window")?;
                let platform = get_optional_string(&screen_info, "platform")?;
                let display = get_optional_u64(&screen_info, "display")?;

                let size: (u32, u32) = canvas.call_method0("get_physical_size")?.extract()?;
                let size = wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                };
                let handles = create_raw_handles(&platform, window, Some(display))?;
                let (context, surface, config) =
                    pollster::block_on(self.create_surface(handles, size))?;
                target_ref.init(context, surface, config);
            }

            Ok(target)
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

    /// Single Python entry point. Accepts bytes, list, str (path or URL),
    /// numpy ndarray, or a TextureMipChain handle. Optional kwargs:
    /// `size=(w, h)` (forces raw-pixel interpretation of bytes),
    /// `format=TextureFormat.X`, `mipmaps=True/False`.
    #[pyo3(
        name = "create_texture",
        signature = (input, size=None, format=None, mipmaps=None)
    )]
    #[lsp_doc("docs/api/core/renderer/create_texture.md")]
    pub fn create_texture_py(
        &self,
        input: Py<PyAny>,
        size: Option<PySize>,
        format: Option<crate::TextureFormat>,
        mipmaps: Option<bool>,
    ) -> Result<crate::texture::Texture, PyErr> {
        Python::attach(|py| -> Result<crate::texture::Texture, PyErr> {
            let spec = py_to_texture_spec(input.bind(py), size, format, mipmaps)?;
            let tex = pollster::block_on(self.create_texture(spec))?;
            Ok(tex)
        })
    }

    #[pyo3(name = "render")]
    #[lsp_doc("docs/api/core/renderer/render.md")]
    pub fn render_py(&self, renderable: Py<PyAny>, target: Py<PyAny>) -> Result<(), PyErr> {
        Python::attach(|py| -> Result<(), PyErr> {
            // Convert any supported Python input (single object or sequence) into a PyRenderable
            let r = crate::PyRenderable::from_any(renderable.bind(py))?;

            // Downcast target to supported targets
            if let Ok(bound) = target.bind(py).cast::<RenderCanvasTarget>() {
                self.render(&r, &*bound.borrow())?;
                Ok(())
            } else if let Ok(bound) = target.bind(py).cast::<PyTextureTarget>() {
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

    // Storage texture — single entry, optional data + usage kwargs.
    #[pyo3(
        name = "create_storage_texture",
        signature = (size, format, data=None, usage_bits=None)
    )]
    #[lsp_doc("docs/api/core/renderer/hidden/create_storage_texture_py.md")]
    pub fn create_storage_texture_py(
        &self,
        size: PySize,
        format: crate::TextureFormat,
        data: Option<Vec<u8>>,
        usage_bits: Option<u32>,
    ) -> Result<crate::texture::Texture, PyErr> {
        Python::attach(|_py| -> Result<crate::texture::Texture, PyErr> {
            let input = crate::TextureInput {
                data: match data {
                    Some(bytes) => crate::TextureData::Bytes(bytes),
                    None => crate::TextureData::Empty,
                },
                options: crate::TextureOptions {
                    size: Some(size.into()),
                    format,
                    usage: usage_bits,
                    ..Default::default()
                },
            };
            let tex = pollster::block_on(self.create_storage_texture(input))?;
            Ok(tex)
        })
    }

    #[pyo3(name = "unregister_texture")]
    #[lsp_doc("docs/api/core/renderer/unregister_texture.md")]
    pub fn unregister_texture_py(&self, texture_id: Py<PyAny>) -> Result<(), PyErr> {
        Python::attach(|py| -> Result<(), PyErr> {
            let id = crate::texture::py_to_texture_id(texture_id.bind(py))?;
            self.unregister_texture(id)?;
            Ok(())
        })
    }

    /// Read back the mip-0 contents of a registered texture as tightly-packed
    /// bytes in the texture's native format. Blocks the Python thread
    /// synchronously (via `pollster::block_on`) — the async readback is the
    /// canonical implementation; the blocking wrapper exists because Python
    /// does not have a native async runtime that integrates with the GPU
    /// device loop.
    #[pyo3(name = "read_texture")]
    #[lsp_doc("docs/api/core/renderer/read_texture.md")]
    pub fn read_texture_py(&self, texture_id: Py<PyAny>) -> Result<Vec<u8>, PyErr> {
        Python::attach(|py| -> Result<Vec<u8>, PyErr> {
            let id = crate::texture::py_to_texture_id(texture_id.bind(py))?;
            pollster::block_on(self.read_texture(id))
                .map_err(|e| crate::error::PyFragmentColorError::new_err(e.to_string()))
        })
    }
}

/// Convert any supported Python input into a `TextureInput`. Centralizes the
/// dispatch that used to live duplicated across four `create_texture_*`
/// methods. Numpy arrays auto-infer their size into the spec; explicit
/// kwargs (`size`, `format`, `mipmaps`) override the inferred values.
fn py_to_texture_spec(
    input: &pyo3::Bound<'_, PyAny>,
    explicit_size: Option<PySize>,
    explicit_format: Option<crate::TextureFormat>,
    explicit_mipmaps: Option<bool>,
) -> Result<crate::texture::TextureInput, PyErr> {
    use crate::texture::{TextureData, TextureInput, TextureOptions};

    let mut options = TextureOptions::default();
    if let Some(s) = explicit_size {
        options.size = Some(s.into());
    }
    if let Some(f) = explicit_format {
        options.format = f;
    }
    if let Some(m) = explicit_mipmaps {
        options.mipmaps = m;
    }

    // PyBytes
    if let Ok(b) = input.extract::<Py<pyo3::types::PyBytes>>() {
        let vec: Vec<u8> = pyo3::Python::attach(|py| b.extract(py))?;
        return Ok(TextureInput {
            data: TextureData::Bytes(vec),
            options,
        });
    }
    // PyList of u8
    if let Ok(pylist) = input.extract::<Py<pyo3::types::PyList>>() {
        let vec: Vec<u8> = pyo3::Python::attach(|py| pylist.extract(py))?;
        return Ok(TextureInput {
            data: TextureData::Bytes(vec),
            options,
        });
    }
    // str path or URL — Path (URLs go through the Url variant elsewhere; the
    // canonical create_texture handles both via TextureData's own dispatch).
    if let Ok(path) = input.extract::<String>() {
        return Ok(TextureInput {
            data: TextureData::Path(std::path::PathBuf::from(path)),
            options,
        });
    }
    // numpy ndarray → RGBA8 + auto-fill size if not explicitly provided.
    if let Ok(any) = input.cast::<numpy::PyArrayDyn<u8>>() {
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
                            rgba[dst_idx..dst_idx + 4].copy_from_slice(&data[src_idx..src_idx + 4]);
                        }
                        _ => {
                            return Err(crate::error::PyFragmentColorError::new_err(
                                "ndarray last dimension must be 1, 3, or 4",
                            ));
                        }
                    }
                }
            }
            if options.size.is_none() {
                options.size = Some(crate::Size::new(w, h, None));
            }
            return Ok(TextureInput {
                data: TextureData::Bytes(rgba),
                options,
            });
        }
    }
    // TextureMipChain handle (built off-thread via TextureMipChain.prepare).
    if let Ok(chain_ref) = input.cast::<crate::texture::TextureMipChain>() {
        let chain = chain_ref.borrow().clone();
        return Ok(TextureInput {
            data: TextureData::Prepared(chain),
            options,
        });
    }
    Err(crate::error::PyFragmentColorError::new_err(
        "Unsupported input for create_texture (expected bytes, list, str path, numpy ndarray, or TextureMipChain)",
    ))
}
