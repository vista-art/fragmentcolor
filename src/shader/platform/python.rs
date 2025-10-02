#![cfg(python)]

use crate::shader::lsp_doc;
use crate::{Shader, ShaderError, UniformData};
use pyo3::prelude::*;

#[pymethods]
impl Shader {
    #[new]
    pub fn new_py(source: &str) -> Result<Self, ShaderError> {
        Shader::new(source)
    }

    #[pyo3(name = "set")]
    pub fn set_py(&self, key: &str, value: Py<PyAny>) -> Result<(), PyErr> {
        Python::attach(|py| -> Result<(), PyErr> {
            // If it's a Texture object, map to UniformData::Texture with id only
            if let Ok(tex) = value.bind(py).downcast::<crate::texture::Texture>() {
                let meta = crate::texture::TextureMeta::with_id_only(tex.borrow().id);
                return self
                    .object
                    .set(key, UniformData::Texture(meta))
                    .map_err(|e| e.into());
            }
            // Fallback: try to extract as UniformData via derived conversions
            let ud: UniformData = value.bind(py).extract()?;
            self.object.set(key, ud).map_err(|e| e.into())
        })
    }

    #[pyo3(name = "get")]
    pub fn get_py(&self, key: &str) -> Result<Py<PyAny>, PyErr> {
        Python::attach(|py| -> Result<Py<PyAny>, PyErr> {
            let data = self.object.get_uniform_data(key)?;

            let object = data.into_pyobject(py)?;

            Ok(object.unbind())
        })
    }

    #[pyo3(name = "list_uniforms")]
    pub fn list_uniforms_py(&self) -> Vec<String> {
        self.list_uniforms()
    }

    #[pyo3(name = "list_keys")]
    pub fn list_keys_py(&self) -> Vec<String> {
        self.list_keys()
    }

    #[pyo3(name = "add_mesh")]
    #[lsp_doc("docs/api/core/shader/add_mesh.md")]
    pub fn add_mesh_py(&self, mesh: &crate::mesh::Mesh) -> Result<(), PyErr> {
        self.add_mesh(mesh).map_err(|e| e.into())
    }

    #[pyo3(name = "remove_mesh")]
    #[lsp_doc("docs/api/core/shader/remove_mesh.md")]
    pub fn remove_mesh_py(&self, mesh: &crate::mesh::Mesh) {
        self.remove_mesh(mesh)
    }

    #[pyo3(name = "remove_meshes")]
    #[lsp_doc("docs/api/core/shader/remove_meshes.md")]
    pub fn remove_meshes_py(&self, list: Vec<crate::mesh::Mesh>) {
        for m in list.iter() {
            self.remove_mesh(m);
        }
    }

    #[pyo3(name = "clear_meshes")]
    #[lsp_doc("docs/api/core/shader/clear_meshes.md")]
    pub fn clear_meshes_py(&self) {
        self.clear_meshes()
    }

    #[pyo3(name = "validate_mesh")]
    #[lsp_doc("docs/api/core/shader/validate_mesh.md")]
    pub fn validate_mesh_py(&self, mesh: &crate::mesh::Mesh) -> Result<(), PyErr> {
        self.validate_mesh(mesh).map_err(|e| e.into())
    }

    #[pyo3(name = "is_compute")]
    #[lsp_doc("docs/api/core/shader/is_compute.md")]
    pub fn is_compute_py(&self) -> bool {
        self.is_compute()
    }

    pub fn renderable_type(&self) -> &'static str {
        "Shader"
    }

    #[pyo3(name = "passes")]
    pub fn passes_py(&self) -> crate::PyPassIterator {
        crate::PyPassIterator(vec![self.pass.clone()])
    }

    #[staticmethod]
    #[pyo3(name = "default")]
    pub fn default_py() -> Self {
        Shader::default()
    }

    #[staticmethod]
    #[pyo3(name = "from_vertex")]
    #[lsp_doc("docs/api/core/shader/from_vertex.md")]
    pub fn from_vertex_py(v: &crate::mesh::Vertex) -> Self {
        Shader::from_vertex(v)
    }

    #[staticmethod]
    #[pyo3(name = "from_mesh")]
    #[lsp_doc("docs/api/core/shader/from_mesh.md")]
    pub fn from_mesh_py(mesh: &crate::mesh::Mesh) -> Self {
        Shader::from_mesh(mesh)
    }
}
