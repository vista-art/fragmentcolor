use std::sync::Arc;

use crate::{
    Frame, InitializationError, Pass, PassObject, RenderCanvasTarget, Renderable, Renderer, Shader,
};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyPassIterator(pub Vec<Arc<PassObject>>);

impl PyPassIterator {
    pub fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.0.iter().map(|pass| pass.as_ref())
    }
}

impl IntoIterator for PyPassIterator {
    type Item = Arc<PassObject>;
    type IntoIter = std::vec::IntoIter<Arc<PassObject>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[pyclass]
pub struct PyRenderable {
    iterator: PyPassIterator,
}

impl<'py> From<&'py Bound<'py, Frame>> for PyRenderable {
    fn from(frame: &Bound<'py, Frame>) -> Self {
        let iterator = Python::with_gil(|_| -> PyResult<Self> {
            let iterator = frame.call_method0("passes")?.extract::<PyPassIterator>()?;

            Ok(Self { iterator })
        })
        .unwrap();

        iterator
    }
}

impl<'py> From<&'py Bound<'py, Shader>> for PyRenderable {
    fn from(shader: &Bound<'py, Shader>) -> Self {
        let iterator = Python::with_gil(|_| -> PyResult<Self> {
            let iterator = shader.call_method0("passes")?.extract::<PyPassIterator>()?;

            Ok(Self { iterator })
        })
        .unwrap();

        iterator
    }
}

impl<'py> From<&'py Bound<'py, Pass>> for PyRenderable {
    fn from(pass: &Bound<'py, Pass>) -> Self {
        let iterator = Python::with_gil(|_| -> PyResult<Self> {
            let iterator = pass.call_method0("passes")?.extract::<PyPassIterator>()?;

            Ok(Self { iterator })
        })
        .unwrap();

        iterator
    }
}

impl Renderable for PyRenderable {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.iterator.0.iter().map(|pass| pass.as_ref())
    }
}

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

    #[pyo3(name = "headless")]
    #[staticmethod]
    /// Creates a headless renderer
    pub async fn headless_py() -> Result<Renderer, InitializationError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = crate::platform::all::request_headless_adapter(&instance).await?;
        let (device, queue) = crate::platform::all::request_device(&adapter).await?;

        let mut renderer = Renderer::new();
        renderer.init(device, queue);

        Ok(renderer)
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
}

#[pymethods]
impl Renderer {
    pub fn create_target(&self, rendercanvas: PyObject) -> Result<Py<RenderCanvasTarget>, PyErr> {
        Python::with_gil(|py| -> Result<Py<RenderCanvasTarget>, PyErr> {
            // If the context is already initialized, return the renderer and target
            let args = PyTuple::new(py, &["fragmentcolor"])?;
            let py_context = rendercanvas.call_method1(py, "get_context", args)?; // calls hook
            let bound_context = py_context.downcast_bound::<RenderCanvasContext>(py)?;
            let mut context = bound_context.borrow_mut();
            if context.is_ready() {
                let target = context.target()?;
                return Ok(target);
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

            let target = self.init_target(size, handle)?;

            let target = Py::new(py, target)?;

            context.init_context(renderer.clone_ref(py), target.clone_ref(py));

            Ok(target)
        })
    }
}

impl Renderer {
    fn init_target(
        &self,
        size: wgpu::Extent3d,
        handle: PyWindowHandle<'static>,
    ) -> Result<RenderCanvasTarget, PyErr> {
        let context = self.get_context(handle);

        let target = RenderCanvasTarget::new(surface, config);
        let renderer = Renderer::init(device, queue);

        Ok(target)
    }
}
