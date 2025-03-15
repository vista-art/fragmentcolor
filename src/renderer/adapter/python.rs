use crate::{
    Frame, InitializationError, Pass, PassObject, PyPassIterator, PyWindowTarget, Renderable,
    Renderer, Shader, ShaderError,
};
use pyo3::prelude::*;

#[pyclass]
pub struct PyRenderable {
    iterator: PyPassIterator,
}

impl From<Frame> for PyRenderable {
    fn from(frame: Frame) -> Self {
        Self {
            iterator: frame.passes(),
        }
    }
}

impl From<Shader> for PyRenderable {
    fn from(shader: Shader) -> Self {
        Self {
            iterator: shader.passes(),
        }
    }
}

impl From<Pass> for PyRenderable {
    fn from(pass: Pass) -> Self {
        Self {
            iterator: pass.passes(),
        }
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
    /// Creates a headless renderer by default
    pub fn new_py() -> Result<Renderer, InitializationError> {
        pollster::block_on(Self::headless())
    }

    #[pyo3(name = "headless")]
    #[staticmethod]
    /// Creates a headless renderer
    pub async fn headless_py() -> Result<Renderer, InitializationError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = crate::platform::all::request_headless_adapter(&instance).await?;
        let (device, queue) = crate::platform::all::request_device(&adapter).await?;

        Ok(Renderer::init(device, queue))
    }

    #[pyo3(name = "render")]
    /// Renders a Frame or Shader to a Target.
    pub fn render_py(
        &self,
        renderable: &PyRenderable,
        target: &PyWindowTarget,
    ) -> Result<(), ShaderError> {
        self.render(renderable, target)
    }
}
