use crate::{FragmentColor, FragmentColorError, Frame, Pass, Renderer, Shader, Target};
use pyo3::prelude::*;
use pyo3::types::PyDict;

pub mod target;
pub use target::*;

#[pymethods]
impl FragmentColor {
    #[staticmethod]
    pub fn init(context: &RenderCanvasContext) -> Result<(Renderer, PyWindowTarget), PyErr> {
        let rendercanvas = context.canvas();

        let (size, handle) = Python::with_gil(|py| -> PyResult<_> {
            // Returns a list of the possible present methods ("screen", "bitmap")
            let present_methods =
                rendercanvas.call_method(py, "_rc_get_present_methods", (), None)?;

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

            let (window_handle, display_handle) =
                create_raw_handles(platform, window, Some(display))?;

            Ok((
                wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                PyWindowHandle {
                    window_handle,
                    display_handle,
                },
            ))
        })?;

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(handle).map_err(|e| {
            FragmentColorError::new_err(format!("Failed to create surface: {:?}", e))
        })?;
        let adapter = crate::platform::all::request_adapter_sync(&instance, Some(&surface))?;
        let (device, queue) = crate::platform::all::request_device_sync(&adapter)?;
        let config = crate::platform::all::configure_surface(&device, &adapter, &surface, &size);

        let target = PyWindowTarget { surface, config };
        let renderer = Renderer::init(device, queue);

        Ok((renderer, target))
    }
}

#[pyclass(dict)]
#[derive(Debug)]
pub struct RenderCanvasContext {
    canvas: PyObject,
    _present_methods: PyObject,
    renderer: Option<Renderer>,
    target: Option<PyWindowTarget>,
}

#[pymethods]
impl RenderCanvasContext {
    #[new]
    pub fn new(canvas: PyObject, _present_methods: PyObject) -> Self {
        let mut context = RenderCanvasContext {
            canvas,
            _present_methods,
            renderer: None,
            target: None,
        };

        let (renderer, target) =
            FragmentColor::init(&context).expect("Failed to initialize FragmentColor");

        context.renderer = Some(renderer);
        context.target = Some(target);

        context
    }

    // Interface that a context must implement, to be usable with a ``RenderCanvas``.
    // Upstream documentation: https://rendercanvas.readthedocs.io/stable/contextapi.html
    //
    // fn canvas(&self) -> &PyObject;
    // fn present(&self) -> Py<PyDict>;
    //
    // Note this is a duck-typed interface, we can't export a impl Trait block with Pyo3.

    #[getter]
    pub fn canvas(&self) -> &PyObject {
        &self.canvas
    }

    pub fn present(&self) -> Py<PyDict> {
        let dict = if let Some(target) = &self.target {
            let frame = target
                .get_current_frame()
                .expect("Failed to get current frame");
            frame.present();

            // must return confirmation: {"method": "screen"}
            let dict = Python::with_gil(|py| -> PyResult<Py<PyDict>> {
                let dict = PyDict::new(py);
                dict.set_item("method", "screen")?;
                Ok(dict.unbind())
            });

            // RenderCanvas expects a dictionary, we can't panic or return Result
            match dict {
                Ok(d) => d,

                // If the dict creation fails, return a fail dict
                Err(e) => {
                    let dict = Python::with_gil(|py| -> PyResult<Py<PyDict>> {
                        let dict = PyDict::new(py);
                        dict.set_item("method", "fail")?;
                        dict.set_item("message", e.to_string())?;
                        Ok(dict.unbind())
                    });

                    // If the fail message fails, return an empty dict (fail silently)
                    match dict {
                        Ok(d) => d,
                        Err(_) => Python::with_gil(|py| -> Py<PyDict> { PyDict::new(py).unbind() }),
                    }
                }
            }
        } else {
            let dict = Python::with_gil(|py| -> PyResult<Py<PyDict>> {
                let dict = PyDict::new(py);
                dict.set_item("method", "fail")?;
                dict.set_item("message", "Context not Initialized")?;
                Ok(dict.unbind())
            });

            // If the fail message fails, return an empty dict (fail silently)
            match dict {
                Ok(d) => d,
                Err(_) => Python::with_gil(|py| -> Py<PyDict> { PyDict::new(py).unbind() }),
            }
        };

        dict
    }
}

#[pyfunction]
/// The context hook that will be called from Python by RenderCanvas
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
    m.add_class::<PyWindowTarget>()?;
    m.add_class::<PyWindowFrame>()?;
    m.add_class::<RenderCanvasContext>()?;
    m.add_class::<Renderer>()?;
    m.add_class::<Shader>()?;
    m.add_class::<Pass>()?;
    m.add_class::<Frame>()?;

    // Custom error type
    m.add(
        "FragmentColorError",
        m.py().get_type::<FragmentColorError>(),
    )?;

    // initializer for RenderCanvas
    m.add_function(wrap_pyfunction!(rendercanvas_context_hook, m)?)?;

    Ok(())
}
