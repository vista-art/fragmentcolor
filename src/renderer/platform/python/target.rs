use crate::{RenderContext, Size, Target, TargetFrame, WindowTarget};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;

#[pyfunction]
/// The context hook that will be called from Python by RenderCanvas
/// When the user calls `RenderCanvas.get_context("fragmentcolor")`
pub fn rendercanvas_context_hook(
    canvas: Py<PyAny>,
    present_methods: Py<PyAny>,
) -> RenderCanvasTarget {
    RenderCanvasTarget::new(canvas, present_methods)
}

#[lsp_doc("docs/api/render_canvas_target/render_canvas_target.md")]
#[pyclass(dict)]
pub struct RenderCanvasTarget {
    canvas: Py<PyAny>,
    _present_methods: Py<PyAny>, // @TODO figure how RenderCanvas expects me to use this
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
    pub fn new(canvas: Py<PyAny>, _present_methods: Py<PyAny>) -> Self {
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

    pub fn resize(&mut self, size: crate::PySize) {
        let size: Size = size.into();
        <Self as Target>::resize(self, size);
    }

    // We can't export a impl Trait block with Pyo3, so this is a
    // duck-typed interface that a context must implement, to be usable with RenderCanvas.
    // Upstream documentation: https://rendercanvas.readthedocs.io/stable/contextapi.html
    //
    // fn canvas(&self) -> Py<PyAny>;
    // fn present(&self) -> Result<Py<PyDict>, PyErr>;

    #[getter]
    pub fn canvas(&self) -> Py<PyAny> {
        Python::attach(|py| self.canvas.clone_ref(py))
    }

    pub fn present(&self) -> Result<Py<PyDict>, PyErr> {
        Python::attach(|py| -> PyResult<Py<PyDict>> {
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
    fn size(&self) -> Size {
        if let Some(target) = &self.target {
            target.size()
        } else {
            Size::default()
        }
    }

    fn resize(&mut self, size: impl Into<Size>) {
        if let Some(target) = &mut self.target {
            target.resize(size.into());
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

    fn get_image(&self) -> Vec<u8> {
        // Window-backed targets are not readback-friendly; prefer TextureTarget for screenshots.
        Vec::new()
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

// Headless texture target for Python
#[lsp_doc("docs/api/py_texture_target/py_texture_target.md")]
#[pyclass(name = "TextureTarget")]
pub struct PyTextureTarget {
    inner: crate::TextureTarget,
}

#[pymethods]
impl PyTextureTarget {
    #[getter]
    pub fn size(&self) -> [u32; 2] {
        let size = <Self as Target>::size(self);
        [size.width, size.height]
    }

    pub fn resize(&mut self, size: crate::PySize) {
        let size: Size = size.into();
        <Self as Target>::resize(self, size);
    }
}

impl From<crate::TextureTarget> for PyTextureTarget {
    fn from(value: crate::TextureTarget) -> Self {
        Self { inner: value }
    }
}

impl Target for PyTextureTarget {
    fn size(&self) -> Size {
        self.inner.size()
    }

    fn resize(&mut self, size: impl Into<Size>) {
        <crate::TextureTarget as Target>::resize(&mut self.inner, size);
    }

    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        self.inner.get_current_frame()
    }

    fn get_image(&self) -> Vec<u8> {
        <crate::TextureTarget as Target>::get_image(&self.inner)
    }
}
