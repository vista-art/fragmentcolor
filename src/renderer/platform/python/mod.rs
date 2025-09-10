use crate::{FragmentColorError, Frame, Pass, PySize, Renderer, Shader};
use lsp_doc::lsp_doc;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

pub mod target;
pub use target::*;

pub mod iterator;
pub use iterator::*;

pub mod renderable;
pub use renderable::*;

pub mod handle;
use crate::renderer::WindowHandles;
use handle::create_raw_handles;

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
                .ok_or(FragmentColorError::new_err("Object can't render to screen"))?;
            let screen_info = dict.downcast::<PyDict>()?;

            // Mandatory WindowHandle for all platforms
            let window: u64 = screen_info
                .get_item("window")?
                .ok_or(FragmentColorError::new_err("Missing window handle"))?
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

            let handles: WindowHandles = create_raw_handles(platform, window, Some(display))?;

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
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
pub fn fragmentcolor(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Renderer>()?;
    m.add_class::<Shader>()?;
    m.add_class::<Pass>()?;
    m.add_class::<Frame>()?;

    // RenderCanvas API
    m.add_function(wrap_pyfunction!(rendercanvas_context_hook, m)?)?;
    m.add_class::<RenderCanvasTarget>()?;
    m.add_class::<RenderCanvasFrame>()?;

    // Headless TextureTarget API
    m.add_class::<PyTextureTarget>()?;

    // Custom error type
    m.add(
        "FragmentColorError",
        m.py().get_type::<FragmentColorError>(),
    )?;

    Ok(())
}
