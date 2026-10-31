//! The `Renderer` methods Python calls.

use crate::bindings::python::PyUnsigned;
use crate::{
    PySize, Renderer, Size,
    target::{PyTextureTarget, RenderCanvasTarget},
    texture::{PyTextureId, PyTextureSource},
};
use lsp_doc::lsp_doc;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

mod pass_iterator;
pub use pass_iterator::*;

mod renderable;
pub use renderable::*;

mod window_handles;
pub(crate) use window_handles::*;

/// The target cached on the canvas, if `renderer` made it.
/// A target made by another renderer holds a surface for another device.
fn cached_target(canvas: &Bound<'_, PyAny>, renderer: &Renderer) -> Option<Py<RenderCanvasTarget>> {
    let existing = canvas.getattr("_fragmentcolor_target").ok()?;
    let target = existing.cast::<RenderCanvasTarget>().ok()?;
    let context = renderer.context.read().clone()?;
    let same = target
        .borrow()
        .context()
        .is_some_and(|own| std::sync::Arc::ptr_eq(own, &context));
    same.then(|| target.clone().unbind())
}

/// The screen handles of `canvas`, read through the context FragmentColor
/// registers on it, since rendercanvas draws only for a canvas with a context.
/// A rendercanvas without contexts, or a canvas that already has another
/// context, which then does the drawing, hands them over directly.
fn get_screen_info<'py>(
    canvas: &pyo3::Bound<'py, PyAny>,
) -> Result<pyo3::Bound<'py, PyDict>, PyErr> {
    let context = canvas
        .py()
        .import("fragmentcolor._context")
        .and_then(|module| module.getattr("FragmentColorContext"))
        .and_then(|class| canvas.call_method1("get_context", (class,)));
    let info = match context {
        Ok(context) => context.getattr("_present_info")?,
        Err(_) => canvas.call_method1("_rc_get_present_info", (vec!["screen"],))?,
    };
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

    #[staticmethod]
    #[pyo3(name = "set_log_level")]
    #[lsp_doc("docs/api/core/renderer/set_log_level.md")]
    pub fn set_log_level_py(level: crate::LogLevel) {
        Self::set_log_level(level);
    }

    #[lsp_doc("docs/api/core/renderer/create_target.md")]
    #[pyo3(name = "create_target")]
    pub fn create_target_py(
        &self,
        rendercanvas: Py<PyAny>,
    ) -> Result<Py<RenderCanvasTarget>, PyErr> {
        Python::attach(|py| -> Result<Py<RenderCanvasTarget>, PyErr> {
            let canvas = rendercanvas.bind(py);
            if let Some(target) = cached_target(canvas, self) {
                return Ok(target);
            }
            let none = py.None();
            let target = Py::new(
                py,
                RenderCanvasTarget::new(rendercanvas.clone_ref(py), none),
            )?;
            {
                let mut target_ref = target.bind(py).borrow_mut();
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
            canvas.setattr("_fragmentcolor_target", &target)?;
            Ok(target)
        })
    }

    #[lsp_doc("docs/api/core/renderer/create_texture_target.md")]
    #[pyo3(name = "create_texture_target")]
    pub fn create_texture_target_py(&self, size: PySize<'_>) -> Result<Py<PyTextureTarget>, PyErr> {
        Python::attach(|py| -> Result<Py<PyTextureTarget>, PyErr> {
            let target = pollster::block_on(self.create_texture_target(Size::try_from(size)?))?;
            let py_target: PyTextureTarget = target.into();
            Ok(py_target.into_pyobject(py)?.unbind())
        })
    }

    /// Single Python entry point. Accepts a Texture, bytes, a bytearray, a
    /// memoryview, a list or tuple of ints, a path or URL string, an
    /// os.PathLike, a uint8 numpy ndarray, or a Mipmap. Optional kwargs:
    /// `size=(w, h)` (forces raw-pixel interpretation of bytes),
    /// `format=TextureFormat.X`, `mipmaps=True/False`, `sampler=SamplerOptions(...)`
    /// and `usage=TextureUsage.COPY_SRC`.
    #[pyo3(
        name = "create_texture",
        signature = (input, size=None, format=None, mipmaps=None, sampler=None, usage=None)
    )]
    #[lsp_doc("docs/api/core/renderer/create_texture.md")]
    pub fn create_texture_py<'py>(
        &self,
        input: PyTextureSource<'py>,
        size: Option<PySize<'py>>,
        format: Option<crate::TextureFormat>,
        mipmaps: Option<bool>,
        sampler: Option<crate::texture::PySamplerOptions<'py>>,
        usage: Option<crate::texture::PyTextureUsage<'py>>,
    ) -> Result<crate::texture::Texture, PyErr> {
        let keywords = TextureKeywords::new(size, format, mipmaps, sampler, usage);
        let spec = py_to_texture_spec(input, keywords, "Renderer.create_texture")?;
        Ok(pollster::block_on(self.create_texture(spec))?)
    }

    #[pyo3(name = "load")]
    #[lsp_doc("docs/api/core/renderer/load.md")]
    pub fn load_py(&self, renderable: Py<PyAny>) -> Result<(), PyErr> {
        Python::attach(|py| -> Result<(), PyErr> {
            let r = crate::PyRenderable::from_any(renderable.bind(py))?;
            pollster::block_on(self.load(&r))?;
            Ok(())
        })
    }

    #[pyo3(name = "read_storage")]
    #[lsp_doc("docs/api/core/renderer/read_storage.md")]
    pub fn read_storage_py(&self, shader: &crate::Shader, binding: &str) -> Result<Vec<u8>, PyErr> {
        Ok(pollster::block_on(self.read_storage(shader, binding))?)
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
                    "Unsupported target type. Expected WindowTarget or TextureTarget",
                ))
            }
        })?;

        Ok(())
    }

    // Depth texture
    #[pyo3(name = "create_depth_texture")]
    #[lsp_doc("docs/api/core/renderer/hidden/create_depth_texture_py.md")]
    pub fn create_depth_texture_py(
        &self,
        size: PySize<'_>,
    ) -> Result<crate::texture::Texture, PyErr> {
        Python::attach(|_py| -> Result<crate::texture::Texture, PyErr> {
            let tex = pollster::block_on(self.create_depth_texture(Size::try_from(size)?))?;
            Ok(tex)
        })
    }

    // Storage texture — single entry, optional data + usage kwargs.
    // `usage_bits` is the old name of `usage` and warns when used.
    #[pyo3(
        name = "create_storage_texture",
        signature = (size, format, data=None, usage=None, usage_bits=None)
    )]
    #[lsp_doc("docs/api/core/renderer/hidden/create_storage_texture_py.md")]
    pub fn create_storage_texture_py(
        &self,
        py: Python<'_>,
        size: PySize<'_>,
        format: crate::TextureFormat,
        data: Option<crate::texture::PyTextureBytes<'_>>,
        usage: Option<crate::texture::PyTextureUsage<'_>>,
        usage_bits: Option<PyUnsigned<'_>>,
    ) -> Result<crate::texture::Texture, PyErr> {
        if usage_bits.is_some() {
            let warning = py.get_type::<pyo3::exceptions::PyDeprecationWarning>();
            PyErr::warn(py, &warning, c"usage_bits is deprecated, pass usage", 1)?;
        }
        let input = crate::TextureInput {
            data: match data {
                Some(bytes) => {
                    crate::TextureData::Bytes(bytes.read("Renderer.create_storage_texture: data")?)
                }
                None => crate::TextureData::Empty,
            },
            options: crate::TextureOptions {
                size: Some(Size::try_from(size)?),
                format,
                usage: match (usage, usage_bits) {
                    (Some(usage), _) => Some(usage.read("usage")?),
                    (None, Some(bits)) => Some(bits.read("usage_bits")?),
                    (None, None) => None,
                },
                ..Default::default()
            },
        };
        Ok(pollster::block_on(self.create_storage_texture(input))?)
    }

    #[pyo3(name = "unregister_texture")]
    #[lsp_doc("docs/api/core/renderer/unregister_texture.md")]
    pub fn unregister_texture_py(&self, texture_id: PyTextureId<'_>) -> Result<(), PyErr> {
        self.unregister_texture(texture_id.read("Renderer.unregister_texture: texture_id")?)?;
        Ok(())
    }

    /// Read back the mip-0 contents of a registered texture as tightly-packed
    /// bytes in the texture's native format. Blocks the Python thread
    /// synchronously (via `pollster::block_on`) — the async readback is the
    /// canonical implementation; the blocking wrapper exists because Python
    /// does not have a native async runtime that integrates with the GPU
    /// device loop.
    #[pyo3(name = "read_texture")]
    #[lsp_doc("docs/api/core/renderer/read_texture.md")]
    pub fn read_texture_py(&self, texture_id: PyTextureId<'_>) -> Result<Vec<u8>, PyErr> {
        let id = texture_id.read("Renderer.read_texture: texture_id")?;
        Ok(pollster::block_on(self.read_texture(id))?)
    }
}

/// Builds the `TextureInput` for `create_texture` and the Material texture setters.
/// An ndarray brings its own size, which a `size` keyword must match, and the other kwargs apply to it too.
/// A `Texture` keeps the options it was created with, so a kwarg beside one raises an error that names `call`.
pub(crate) fn py_to_texture_spec(
    input: PyTextureSource<'_>,
    options: TextureKeywords<'_>,
    call: &str,
) -> Result<crate::texture::TextureInput, PyErr> {
    if matches!(input, PyTextureSource::Texture(_)) && !options.is_empty() {
        return Err(crate::texture::TextureError::InvalidInput(format!(
            "{call}: a Texture keeps the options it was created with, so it takes no size, format, mipmaps, sampler or usage"
        ))
        .into());
    }
    let size = options.size.map(Size::try_from).transpose()?;
    let mut spec = match input {
        PyTextureSource::Array(array) => {
            crate::texture::ndarray_input(&array, size, options.format)?
        }
        source => source.read(call)?,
    };
    spec.options.size = spec.options.size.or(size);
    if let Some(format) = options.format {
        spec.options.format = format;
    }
    if let Some(mipmaps) = options.mipmaps {
        spec.options.mipmaps = mipmaps;
    }
    if let Some(sampler) = options.sampler {
        spec.options.sampler = crate::SamplerOptions::try_from(sampler)?;
    }
    if let Some(usage) = options.usage {
        spec.options.usage = Some(usage.read("usage")?);
    }
    Ok(spec)
}

/// The keyword arguments of `create_texture` and the Material texture setters, each optional.
pub(crate) struct TextureKeywords<'py> {
    size: Option<PySize<'py>>,
    format: Option<crate::TextureFormat>,
    mipmaps: Option<bool>,
    sampler: Option<crate::texture::PySamplerOptions<'py>>,
    usage: Option<crate::texture::PyTextureUsage<'py>>,
}

impl<'py> TextureKeywords<'py> {
    pub(crate) fn new(
        size: Option<PySize<'py>>,
        format: Option<crate::TextureFormat>,
        mipmaps: Option<bool>,
        sampler: Option<crate::texture::PySamplerOptions<'py>>,
        usage: Option<crate::texture::PyTextureUsage<'py>>,
    ) -> Self {
        Self {
            size,
            format,
            mipmaps,
            sampler,
            usage,
        }
    }

    /// Whether the call passed none of the keywords.
    fn is_empty(&self) -> bool {
        self.size.is_none()
            && self.format.is_none()
            && self.mipmaps.is_none()
            && self.sampler.is_none()
            && self.usage.is_none()
    }
}
