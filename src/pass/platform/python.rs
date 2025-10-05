#![cfg(python)]

use crate::{Pass, PassInput, PassObject, PassType, Shader};
use lsp_doc::lsp_doc;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::sync::Arc;

#[pymethods]
impl Pass {
    #[new]
    pub fn new_py(name: &str) -> Self {
        Self {
            object: Arc::new(PassObject::new(name, PassType::Render)),
        }
    }

    #[staticmethod]
    #[pyo3(name = "compute")]
    pub fn compute_py(name: &str) -> Self {
        Self {
            object: Arc::new(PassObject::new(name, PassType::Compute)),
        }
    }

    #[staticmethod]
    #[pyo3(name = "from_shader")]
    pub fn from_shader_py(name: &str, shader: &Shader) -> Self {
        Self {
            object: Arc::new(PassObject::from_shader_object(name, shader.object.clone())),
        }
    }

    #[pyo3(name = "passes")]
    pub fn passes_py(&self) -> crate::PyPassIterator {
        crate::PyPassIterator(vec![self.object.clone()])
    }

    #[pyo3(name = "load_previous")]
    pub fn load_previous_py(&self) {
        *self.object.input.write() = PassInput::load();
    }

    #[pyo3(name = "get_input")]
    pub fn get_input_py(&self) -> PassInput {
        self.object.get_input()
    }

    #[pyo3(name = "add_shader")]
    pub fn add_shader_py(&self, shader: &Shader) {
        self.object.add_shader(shader);
    }

    #[pyo3(name = "add_mesh")]
    pub fn add_mesh_py(&self, mesh: &crate::mesh::Mesh) -> Result<(), PyErr> {
        self.add_mesh(mesh).map_err(|e| e.into())
    }

    #[pyo3(name = "add_mesh_to_shader")]
    pub fn add_mesh_to_shader_py(
        &self,
        mesh: &crate::mesh::Mesh,
        shader: &crate::Shader,
    ) -> Result<(), PyErr> {
        self.add_mesh_to_shader(mesh, shader).map_err(|e| e.into())
    }

    #[pyo3(name = "set_clear_color")]
    pub fn set_clear_color_py(&self, color: [f32; 4]) {
        self.object.set_clear_color(color);
    }

    #[pyo3(name = "set_viewport")]
    pub fn set_viewport_py(&self, viewport: crate::Region) {
        self.object.set_viewport(viewport);
    }

    #[pyo3(name = "set_compute_dispatch")]
    #[lsp_doc("docs/api/core/pass/set_compute_dispatch.md")]
    pub fn set_compute_dispatch_py(&self, x: u32, y: u32, z: u32) {
        self.object.set_compute_dispatch(x, y, z);
    }

    #[pyo3(name = "add_target")]
    #[lsp_doc("docs/api/core/pass/add_target.md")]
    pub fn add_target_py(&self, target: Py<PyAny>) -> Result<(), PyErr> {
        Python::attach(|py| -> Result<(), PyErr> {
            // Try TextureTarget wrapper first
            if let Ok(bound) = target.bind(py).downcast::<crate::target::PyTextureTarget>() {
                let tt = bound.borrow();
                return self.add_target(&tt.inner).map_err(|e| e.into());
            }
            // Try Texture handle
            if let Ok(tex) = target.bind(py).downcast::<crate::texture::Texture>() {
                let t = tex.borrow();
                return self.add_target(&*t).map_err(|e| e.into());
            }
            Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Unsupported target type. Expected TextureTarget or Texture",
            ))
        })
    }

    #[pyo3(name = "add_depth_target")]
    #[lsp_doc("docs/api/core/pass/add_depth_target.md")]
    pub fn add_depth_target_py(&self, target: Py<PyAny>) -> Result<(), PyErr> {
        Python::attach(|py| -> Result<(), PyErr> {
            // Depth textures are Texture handles
            if let Ok(tex) = target.bind(py).downcast::<crate::texture::Texture>() {
                let t = tex.borrow();
                return self.add_depth_target(&*t).map_err(|e| e.into());
            }
            // Or a TextureTarget (if provided)
            if let Ok(bound) = target.bind(py).downcast::<crate::target::PyTextureTarget>() {
                let tt = bound.borrow();
                return self.add_depth_target(&tt.inner).map_err(|e| e.into());
            }
            Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Unsupported depth target type. Expected Texture or TextureTarget",
            ))
        })
    }

    #[pyo3(name = "is_compute")]
    #[lsp_doc("docs/api/core/pass/is_compute.md")]
    pub fn is_compute_py(&self) -> bool {
        self.object.is_compute()
    }

    #[pyo3(name = "require")]
    #[lsp_doc("docs/api/core/pass/require.md")]
    pub fn require_py(&self, deps: Py<PyAny>) -> Result<(), PyErr> {
        Python::attach(|py| -> Result<(), PyErr> {
            let any = deps.bind(py);
            let r = crate::renderer::PyRenderable::from_any(any)?;
            self.require(&r).map_err(|e| e.into())
        })
    }

    pub fn renderable_type(&self) -> &'static str {
        "Pass"
    }
}
