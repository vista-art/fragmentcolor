#![cfg(python)]

use lsp_doc::lsp_doc;
use pyo3::prelude::*;

use crate::scene::Model;
use crate::{Material, Mesh};

#[pymethods]
impl Model {
    #[new]
    #[lsp_doc("docs/api/scene/model/new.md")]
    pub fn new_py(mesh: Mesh, material: Material) -> Self {
        Model::new(mesh, material)
    }

    #[pyo3(name = "mesh")]
    #[lsp_doc("docs/api/scene/model/mesh.md")]
    pub fn mesh_py(&self) -> Mesh {
        self.mesh.clone()
    }

    #[pyo3(name = "material")]
    #[lsp_doc("docs/api/scene/model/material.md")]
    pub fn material_py(&self) -> Material {
        // Material is Clone (deep clone via Shader::duplicate). For the
        // Python accessor we want share semantics: cheap, mutates propagate.
        // We can't reach into Material here without duplicating; expose a
        // shallow share by reconstructing from the existing Arc-shared Shader.
        Material::custom(self.material.shader.clone())
    }

    #[pyo3(name = "transform")]
    #[lsp_doc("docs/api/scene/model/transform.md")]
    pub fn transform_py(&self) -> [[f32; 4]; 4] {
        self.transform()
    }

    #[pyo3(name = "set_transform")]
    #[lsp_doc("docs/api/scene/model/set_transform.md")]
    pub fn set_transform_py(&self, matrix: [[f32; 4]; 4]) {
        self.set_transform(matrix);
    }

    #[pyo3(name = "translate")]
    #[lsp_doc("docs/api/scene/model/translate.md")]
    pub fn translate_py(&self, offset: [f32; 3]) {
        self.translate(offset);
    }

    #[pyo3(name = "rotate")]
    #[lsp_doc("docs/api/scene/model/rotate.md")]
    pub fn rotate_py(&self, axis: [f32; 3], radians: f32) {
        self.rotate(axis, radians);
    }

    #[pyo3(name = "scale")]
    #[lsp_doc("docs/api/scene/model/scale.md")]
    pub fn scale_py(&self, factor: [f32; 3]) {
        self.scale(factor);
    }
}
