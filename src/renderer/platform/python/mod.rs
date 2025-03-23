use crate::{FragmentColorError, Frame, Pass, Renderer, Shader};
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
use handle::*;

#[pymethods]
impl Renderer {
    #[new]
    /// Creates an uninitialized Renderer.
    ///
    /// At this point we don't know if it should be compatible
    /// with a headless or a windowed environment.
    ///
    /// The render context is initialized when we crate the first target.
    pub fn new_py() -> Renderer {
        Self::new()
    }

    #[pyo3(name = "render")]
    /// Renders a Frame or Shader to a Target.
    pub fn render_py(
        &self,
        renderable: PyObject,
        target: &RenderCanvasTarget,
    ) -> Result<(), PyErr> {
        Python::with_gil(|py| -> Result<(), PyErr> {
            let renderable_type = renderable
                .call_method0(py, "renderable_type")?
                .extract::<String>(py)?;

            match renderable_type.as_str() {
                "Frame" => {
                    let frame = renderable.bind(py).downcast::<Frame>()?;
                    let renderable = PyRenderable::from(frame);

                    self.render(&renderable, target)?;

                    Ok(())
                }
                "Pass" => {
                    let pass = renderable.bind(py).downcast::<Pass>()?;
                    let renderable = PyRenderable::from(pass);

                    self.render(&renderable, target)?;

                    Ok(())
                }
                "Shader" => {
                    let shader = renderable.bind(py).downcast::<Shader>()?;
                    let renderable = PyRenderable::from(shader);

                    self.render(&renderable, target)?;

                    Ok(())
                }
                _ => Err(PyErr::new::<PyTypeError, _>(
                    "Expected a Frame, Pass or Shader object",
                )),
            }
        })?;

        Ok(())
    }

    pub fn create_target(&self, rendercanvas: PyObject) -> Result<Py<RenderCanvasTarget>, PyErr> {
        Python::with_gil(|py| -> Result<Py<RenderCanvasTarget>, PyErr> {
            // If the target is already initialized, return it
            let libname = PyTuple::new(py, &["fragmentcolor"])?;
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

            let (window_handle, display_handle) =
                create_raw_handles(platform, window, Some(display))?;

            let handle = PyWindowHandle {
                window_handle,
                display_handle,
            };

            let (context, surface, config) = pollster::block_on(self.create_surface(handle, size))?;
            target.init(context, surface, config);

            Ok(target.into_pyobject(py)?.unbind())
        })
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

    // Custom error type
    m.add(
        "FragmentColorError",
        m.py().get_type::<FragmentColorError>(),
    )?;

    Ok(())
}
