#![cfg(python)]

use crate::target::FrameSource;
use crate::{RenderContext, Size, Target, TargetFrame, WindowTarget};
use lsp_doc::lsp_doc;
use numpy::PyArrayMethods;
use numpy::pyo3::Python;
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

/// The window target Python gets from `Renderer.create_target`.
/// Python names it `WindowTarget`, as every other binding does, and
/// `fragmentcolor.RenderCanvasTarget` stays as an alias.
#[pyclass(dict, name = "WindowTarget")]
#[lsp_doc("docs/api/hidden/platforms/python/render_canvas_target/render_canvas_target.md")]
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

    /// The render context of the surface, once the target has one.
    pub(crate) fn context(&self) -> Option<&Arc<RenderContext>> {
        self.target.as_ref().map(|target| &target.context)
    }
}

#[pymethods]
impl RenderCanvasTarget {
    // Constructor for the RenderCanvas protocol object — not user-facing API;
    // instantiated internally by Renderer::create_target_py().
    #[doc(hidden)]
    #[new]
    pub fn new(canvas: Py<PyAny>, _present_methods: Py<PyAny>) -> Self {
        Self {
            canvas,
            _present_methods,
            target: None,
        }
    }

    // Internal RenderCanvas protocol sentinel — not a public documented method.
    #[doc(hidden)]
    pub fn is_ready(&self) -> bool {
        self.target.is_some()
    }

    #[lsp_doc("docs/api/targets/window_target/size.md")]
    pub fn size(&self) -> Size {
        <Self as Target>::size(self)
    }

    #[lsp_doc("docs/api/targets/window_target/resize.md")]
    pub fn resize(&mut self, size: crate::PySize) -> PyResult<()> {
        <Self as Target>::resize(self, Size::try_from(size)?);
        Ok(())
    }

    /// Returns an empty byte array.
    ///
    /// Window-backed targets are not readable across all GPU backends.
    /// Use a `TextureTarget` and call `get_image()` there instead.
    #[lsp_doc("docs/api/targets/window_target/get_image.md")]
    pub fn get_image(&self) -> Vec<u8> {
        Vec::new()
    }

    // We can't export a impl Trait block with Pyo3, so this is a
    // duck-typed interface that a context must implement, to be usable with RenderCanvas.
    // Upstream documentation: https://rendercanvas.readthedocs.io/stable/contextapi.html
    //
    // fn canvas(&self) -> Py<PyAny>;
    // fn present(&self) -> Result<Py<PyDict>, PyErr>;

    // Internal RenderCanvas protocol property — not a public documented method.
    #[doc(hidden)]
    #[getter]
    pub fn canvas(&self) -> Py<PyAny> {
        Python::attach(|py| self.canvas.clone_ref(py))
    }

    // Internal RenderCanvas protocol method — not a public documented method.
    #[doc(hidden)]
    pub fn present(&self) -> Result<Py<PyDict>, PyErr> {
        Python::attach(|py| -> PyResult<Py<PyDict>> {
            let dict = PyDict::new(py);

            if let Some(target) = &self.target {
                match target.get_current_frame() {
                    Ok(frame) => {
                        frame.present(&target.context.queue);
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

impl crate::target::FrameSource for RenderCanvasTarget {
    fn get_current_frame(&self) -> Result<Box<dyn crate::TargetFrame>, crate::SurfaceError> {
        let target = if let Some(target) = &self.target {
            target
        } else {
            return Err(crate::SurfaceError::Lost);
        };

        let surface_texture =
            crate::target::surface_texture_from(target.surface.get_current_texture())?;
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

    async fn get_image(&self) -> Vec<u8> {
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

    fn present(self: Box<Self>, queue: &wgpu::Queue) {
        queue.present(self.surface_texture);
    }

    /// Prevents the Renderer to call present() automatically
    /// to allow RenderCanvas to control the presentation
    fn auto_present(&self) -> bool {
        false
    }
}

// Headless texture target for Python
#[lsp_doc("docs/api/hidden/platforms/python/py_texture_target/py_texture_target.md")]
#[pyclass(name = "TextureTarget")]
pub struct PyTextureTarget {
    pub(crate) inner: crate::TextureTarget,
}

#[pymethods]
impl PyTextureTarget {
    #[lsp_doc("docs/api/targets/texture_target/size.md")]
    #[getter]
    pub fn size(&self) -> Size {
        <Self as Target>::size(self)
    }

    #[lsp_doc("docs/api/targets/texture_target/resize.md")]
    pub fn resize(&mut self, size: crate::PySize) -> PyResult<()> {
        <Self as Target>::resize(self, Size::try_from(size)?);
        Ok(())
    }

    /// Acquire a frame (Python-facing wrapper). Internal — callers use get_image().
    // PyTargetFrame is an internal plumbing type that exposes a minimal
    // RenderCanvas-compatible interface; it has no canonical API doc entry.
    #[doc(hidden)]
    pub fn get_current_frame(&self) -> PyTargetFrame {
        // TextureTarget::get_current_frame format mirrors inner texture format; fall back to a sane default
        let format = match <crate::TextureTarget as FrameSource>::get_current_frame(&self.inner) {
            Ok(frame) => frame.format(),
            Err(_) => wgpu::TextureFormat::Rgba8Unorm,
        };
        PyTargetFrame { format }
    }

    /// Read back the RGBA image as a `(height, width, 4)` uint8 NumPy array.
    #[lsp_doc("docs/api/targets/texture_target/get_image.md")]
    pub fn get_image(&self) -> Result<Py<numpy::PyArray3<u8>>, PyErr> {
        const BPP: usize = 4; // Bytes per pixel (RGBA8)
        let data = pollster::block_on(self.inner.get_image());
        let width = self.size().width as usize;
        let height = self.size().height as usize;

        if data.len() != width * height * BPP {
            return Err(crate::error::PyFragmentColorError::new_err(format!(
                "Unexpected image data length: expected {}, got {}",
                width * height * BPP,
                data.len()
            )));
        }

        Python::attach(|py| -> Result<Py<numpy::PyArray3<u8>>, PyErr> {
            // The readback is row-major, so it takes the image shape without a copy.
            let arr = numpy::PyArray1::from_vec(py, data).reshape([height, width, BPP])?;
            Ok(arr.unbind())
        })
    }

    /// Return a sampleable `Texture` handle that aliases this target's
    /// offscreen storage. Mirrors the Rust-side `TextureTarget::texture`
    /// inherent method so the Python binding has the same single-call
    /// shape `target.texture()` the docs example uses.
    #[lsp_doc("docs/api/targets/texture_target/texture.md")]
    pub fn texture(&self) -> crate::texture::Texture {
        self.inner.texture()
    }
}

impl From<crate::TextureTarget> for PyTextureTarget {
    fn from(value: crate::TextureTarget) -> Self {
        Self { inner: value }
    }
}

#[pyclass(name = "TargetFrame")]
pub struct PyTargetFrame {
    format: wgpu::TextureFormat,
}

#[pymethods]
impl PyTargetFrame {
    // PyTargetFrame is an internal RenderCanvas-protocol type — not part of public docs.
    #[doc(hidden)]
    pub fn format(&self) -> String {
        format!("{:?}", self.format)
    }

    #[doc(hidden)]
    pub fn present(&self) {
        // No-op for offscreen textures in Python bindings
    }
}

impl crate::target::FrameSource for PyTextureTarget {
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, crate::SurfaceError> {
        self.inner.get_current_frame()
    }
}

impl Target for PyTextureTarget {
    fn size(&self) -> Size {
        self.inner.size()
    }

    fn resize(&mut self, size: impl Into<Size>) {
        <crate::TextureTarget as Target>::resize(&mut self.inner, size);
    }

    async fn get_image(&self) -> Vec<u8> {
        self.inner.get_image().await
    }
}
