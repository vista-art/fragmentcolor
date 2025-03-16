use crate::{FragmentColor, FragmentColorError, Frame, Pass, Renderer, Shader, Target};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

pub mod target;
pub use target::*;

#[pymethods]
impl FragmentColor {
    #[staticmethod]
    /// This is the canonical way to start the library
    pub fn init(rendercanvas: PyObject) -> Result<(Py<Renderer>, Py<RenderCanvasTarget>), PyErr> {
        Python::with_gil(
            |py| -> Result<(Py<Renderer>, Py<RenderCanvasTarget>), PyErr> {
                // If the context is already initialized, return the renderer and target
                let args = PyTuple::new(py, &["fragmentcolor"])?;
                let py_context = rendercanvas.call_method1(py, "get_context", args)?; // calls hook
                let bound_context = py_context.downcast_bound::<RenderCanvasContext>(py)?;
                let mut context = bound_context.borrow_mut();
                if context.is_ready() {
                    let renderer = context.renderer()?;
                    let target = context.target()?;
                    return Ok((renderer, target));
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
                    .call_method(py, "get_physical_size", (), None)?
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

                let (renderer, target) = Self::init_renderer_and_target(size, handle)?;

                let renderer = Py::new(py, renderer)?;
                let target = Py::new(py, target)?;

                context.init_context(renderer.clone_ref(py), target.clone_ref(py));

                Ok((renderer, target))
            },
        )
    }
}

impl FragmentColor {
    fn init_renderer_and_target(
        size: wgpu::Extent3d,
        handle: PyWindowHandle<'static>,
    ) -> Result<(Renderer, RenderCanvasTarget), PyErr> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(handle).map_err(|e| {
            FragmentColorError::new_err(format!("Failed to create surface: {:?}", e))
        })?;
        let adapter = crate::platform::all::request_adapter_sync(&instance, Some(&surface))?;
        let (device, queue) = crate::platform::all::request_device_sync(&adapter)?;
        let config = crate::platform::all::configure_surface(&device, &adapter, &surface, &size);

        let target = RenderCanvasTarget { surface, config };
        let renderer = Renderer::init(device, queue);

        Ok((renderer, target))
    }
}

#[pyclass(dict)]
#[derive(Debug)]
pub struct RenderCanvasContext {
    pub(crate) canvas: PyObject,
    _present_methods: PyObject, // @TODO figure how RenderCanvas expects me to use this

    // Reference-counted for Python.
    renderer: Option<Py<Renderer>>,
    target: Option<Py<RenderCanvasTarget>>,
}

fn clone_py<T>(object: &Py<T>) -> Py<T> {
    Python::with_gil(|py| object.clone_ref(py))
}

#[pymethods]
impl RenderCanvasContext {
    #[new]
    pub fn new(canvas: PyObject, _present_methods: PyObject) -> Self {
        RenderCanvasContext {
            canvas,
            _present_methods,
            renderer: None,
            target: None,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.renderer.is_some() && self.target.is_some()
    }

    // @TODO consider storing them as PyObject to be able to return them directly
    #[getter]
    pub fn renderer(&self) -> PyResult<Py<Renderer>> {
        if let Some(renderer) = &self.renderer {
            Ok(clone_py::<Renderer>(renderer))
        } else {
            Err(FragmentColorError::new_err("Renderer not initialized"))
        }
    }

    #[getter]
    pub fn target(&self) -> PyResult<Py<RenderCanvasTarget>> {
        if let Some(target) = &self.target {
            Ok(clone_py::<RenderCanvasTarget>(target))
        } else {
            Err(FragmentColorError::new_err("Target not initialized"))
        }
    }

    // duck-typed interface that a context must implement, to be usable with a `RenderCanvas`.
    // Upstream documentation: https://rendercanvas.readthedocs.io/stable/contextapi.html
    //
    // fn canvas(&self) -> PyObject;
    // fn present(&self) -> Result<Py<PyDict>, PyErr>;
    //
    // We can't export a impl Trait block with Pyo3.

    #[getter]
    pub fn canvas(&self) -> PyObject {
        clone_py::<PyAny>(&self.canvas)
    }

    pub fn present(&self) -> Result<Py<PyDict>, PyErr> {
        Python::with_gil(|py| -> PyResult<Py<PyDict>> {
            if let Some(py_target) = &self.target {
                let target = py_target.borrow(py);
                match target.get_current_frame() {
                    Ok(frame) => {
                        frame.present();

                        // must return confirmation: {"method": "screen"}
                        Python::with_gil(|py| -> PyResult<Py<PyDict>> {
                            let dict = PyDict::new(py);
                            dict.set_item("method", "screen")?;
                            Ok(dict.unbind())
                        })
                    }
                    Err(e) => {
                        // must return error: {"method": "fail", "message": "error message"}
                        Python::with_gil(|py| -> PyResult<Py<PyDict>> {
                            let dict = PyDict::new(py);
                            dict.set_item("method", "fail")?;
                            dict.set_item("message", e.to_string())?;
                            Ok(dict.unbind())
                        })
                    }
                }
            } else {
                // must return error: {"method": "fail", "message": "error message"}
                Python::with_gil(|py| -> PyResult<Py<PyDict>> {
                    let dict = PyDict::new(py);
                    dict.set_item("method", "fail")?;
                    dict.set_item("message", "Target not initialized")?;
                    Ok(dict.unbind())
                })
            }
        })
    }
}

impl RenderCanvasContext {
    pub(crate) fn init_context(&mut self, renderer: Py<Renderer>, target: Py<RenderCanvasTarget>) {
        self.renderer = Some(renderer);
        self.target = Some(target);
    }
}

#[pyfunction]
/// The context hook that will be called from Python by RenderCanvas
/// When the user calls `RenderCanvas.get_context("fragmentcolor")`
pub fn rendercanvas_context_hook(
    canvas: PyObject,
    present_methods: PyObject,
) -> RenderCanvasContext {
    RenderCanvasContext::new(canvas, present_methods)
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
pub fn fragmentcolor(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FragmentColor>()?;
    m.add_class::<Renderer>()?;
    m.add_class::<Shader>()?;
    m.add_class::<Pass>()?;
    m.add_class::<Frame>()?;

    // RenderCanvas API
    m.add_function(wrap_pyfunction!(rendercanvas_context_hook, m)?)?;
    m.add_class::<RenderCanvasTarget>()?;
    m.add_class::<RenderCanvasFrame>()?;
    m.add_class::<RenderCanvasContext>()?;

    // Custom error type
    m.add(
        "FragmentColorError",
        m.py().get_type::<FragmentColorError>(),
    )?;

    Ok(())
}
