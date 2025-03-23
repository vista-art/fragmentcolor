use crate::{RenderContext, Target, TargetFrame, UniformData, WindowTarget};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;

#[pyfunction]
/// The context hook that will be called from Python by RenderCanvas
/// When the user calls `RenderCanvas.get_context("fragmentcolor")`
pub fn rendercanvas_context_hook(
    canvas: PyObject,
    present_methods: PyObject,
) -> RenderCanvasTarget {
    RenderCanvasTarget::new(canvas, present_methods)
}

#[pyclass(dict)]
pub struct RenderCanvasTarget {
    canvas: PyObject,
    _present_methods: PyObject, // @TODO figure how RenderCanvas expects me to use this
    target: Option<WindowTarget>,
}

impl RenderCanvasTarget {
    pub(crate) fn init(
        &mut self,
        context: Arc<RenderContext>,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
    ) {
        self.target = Some(WindowTarget::new(context, surface, config))
    }
}

#[pymethods]
impl RenderCanvasTarget {
    #[new]
    pub fn new(canvas: PyObject, _present_methods: PyObject) -> Self {
        Self {
            canvas,
            _present_methods,
            target: None,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.target.is_some()
    }

    pub fn size(&self) -> [u32; 2] {
        let size = <Self as Target>::size(self);
        [size.width, size.height]
    }

    pub fn resize(&mut self, size: UniformData) {
        <Self as Target>::resize(self, size.into());
    }

    // duck-typed interface that a context must implement, to be usable with RenderCanvas.
    // Upstream documentation: https://rendercanvas.readthedocs.io/stable/contextapi.html
    //
    // fn canvas(&self) -> PyObject;
    // fn present(&self) -> Result<Py<PyDict>, PyErr>;
    //
    // We can't export a impl Trait block with Pyo3.

    #[getter]
    pub fn canvas(&self) -> PyObject {
        Python::with_gil(|py| self.canvas.clone_ref(py))
    }

    pub fn present(&self) -> Result<Py<PyDict>, PyErr> {
        Python::with_gil(|py| -> PyResult<Py<PyDict>> {
            let dict = PyDict::new(py);

            if let Some(target) = &self.target {
                match target.get_current_frame() {
                    Ok(frame) => {
                        frame.present();
                        dict.set_item("method", "screen")?;
                    }
                    Err(e) => {
                        dict.set_item("method", "fail")?;
                        dict.set_item("message", e.to_string())?;
                    }
                }
            } else {
                dict.set_item("method", "fail")?;
                dict.set_item("message", "Target not initialized")?;
            };

            Ok(dict.unbind())
        })
    }
}

#[pyclass]
pub struct RenderCanvasFrame {
    surface_texture: wgpu::SurfaceTexture,
    format: wgpu::TextureFormat,
    view: wgpu::TextureView,
}

impl Target for RenderCanvasTarget {
    fn size(&self) -> wgpu::Extent3d {
        if let Some(target) = &self.target {
            target.size()
        } else {
            wgpu::Extent3d::default()
        }
    }

    fn resize(&mut self, size: wgpu::Extent3d) {
        if let Some(target) = &mut self.target {
            target.resize(size);
        }
    }

    fn get_current_frame(&self) -> Result<Box<dyn crate::TargetFrame>, wgpu::SurfaceError> {
        let target = if let Some(target) = &self.target {
            target
        } else {
            return Err(wgpu::SurfaceError::Lost);
        };

        let surface_texture = target.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        Ok(Box::new(RenderCanvasFrame {
            surface_texture,
            format: target.config.format,
            view,
        }))
    }
}

impl TargetFrame for RenderCanvasFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    fn present(self: Box<Self>) {
        self.surface_texture.present();
    }

    /// Prevents the Renderer to call present() automatically
    /// to allow RenderCanvas to control the presentation
    fn auto_present(&self) -> bool {
        false
    }
}
