#![cfg(python)]

use lsp_doc::lsp_doc;
use pyo3::prelude::*;

use crate::scene::{Camera, Light, Model, Scene};
use crate::{Material, Mesh, Pass};

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

#[pymethods]
impl Camera {
    #[staticmethod]
    #[pyo3(name = "perspective")]
    #[lsp_doc("docs/api/scene/camera/perspective.md")]
    pub fn perspective_py(fovy_radians: f32, aspect: f32, near: f32, far: f32) -> Self {
        Camera::perspective(fovy_radians, aspect, near, far)
    }

    #[staticmethod]
    #[pyo3(name = "orthographic")]
    #[lsp_doc("docs/api/scene/camera/orthographic.md")]
    pub fn orthographic_py(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Camera::orthographic(left, right, bottom, top, near, far)
    }

    #[pyo3(name = "look_at")]
    #[lsp_doc("docs/api/scene/camera/look_at.md")]
    pub fn look_at_py(&self, eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Self {
        self.clone().look_at(eye, target, up)
    }

    #[pyo3(name = "view_proj")]
    #[lsp_doc("docs/api/scene/camera/view_proj.md")]
    pub fn view_proj_py(&self) -> [[f32; 4]; 4] {
        self.view_proj()
    }

    #[pyo3(name = "position")]
    #[lsp_doc("docs/api/scene/camera/position.md")]
    pub fn position_py(&self) -> [f32; 3] {
        self.position()
    }
}

#[pymethods]
impl Light {
    #[staticmethod]
    #[pyo3(name = "directional")]
    #[lsp_doc("docs/api/scene/light/directional.md")]
    pub fn directional_py(direction: [f32; 3], color: [f32; 3]) -> Self {
        Light::directional(direction, color)
    }

    #[pyo3(name = "direction")]
    #[lsp_doc("docs/api/scene/light/direction.md")]
    pub fn direction_py(&self) -> [f32; 3] {
        self.direction()
    }

    #[pyo3(name = "color")]
    #[lsp_doc("docs/api/scene/light/color.md")]
    pub fn color_py(&self) -> [f32; 3] {
        self.color()
    }

    #[pyo3(name = "set_direction")]
    #[lsp_doc("docs/api/scene/light/set_direction.md")]
    pub fn set_direction_py(&self, direction: [f32; 3]) -> Self {
        self.set_direction(direction)
    }

    #[pyo3(name = "set_color")]
    #[lsp_doc("docs/api/scene/light/set_color.md")]
    pub fn set_color_py(&self, color: [f32; 3]) -> Self {
        self.set_color(color)
    }
}

#[pymethods]
impl Scene {
    #[new]
    #[lsp_doc("docs/api/scene/scene/new.md")]
    pub fn new_py() -> Self {
        Scene::new()
    }

    #[pyo3(name = "add_model")]
    #[lsp_doc("docs/api/scene/scene/add.md")]
    pub fn add_model_py(&self, model: &Model) -> Result<(), PyErr> {
        self.add(model).map(|_| ()).map_err(|e| e.into())
    }

    #[pyo3(name = "add_camera")]
    #[lsp_doc("docs/api/scene/scene/add.md")]
    pub fn add_camera_py(&self, camera: &Camera) -> Result<(), PyErr> {
        self.add(camera).map(|_| ()).map_err(|e| e.into())
    }

    #[pyo3(name = "add_light")]
    #[lsp_doc("docs/api/scene/scene/add.md")]
    pub fn add_light_py(&self, light: &Light) -> Result<(), PyErr> {
        self.add(light).map(|_| ()).map_err(|e| e.into())
    }

    #[pyo3(name = "add_pass")]
    #[lsp_doc("docs/api/scene/scene/add_pass.md")]
    pub fn add_pass_py(&self, pass: &Pass) {
        self.add_pass(pass);
    }

    // Internal duck-typed interface used by PyRenderable dispatch — not part
    // of public docs.
    #[doc(hidden)]
    #[pyo3(name = "passes")]
    pub fn passes_py(&self) -> crate::PyPassIterator {
        let list = <Self as crate::Renderable>::passes(self);
        crate::PyPassIterator(list.iter().cloned().collect())
    }

    // Internal duck-typed interface used by PyRenderable dispatch — not part
    // of public docs.
    #[doc(hidden)]
    pub fn renderable_type(&self) -> &'static str {
        "Scene"
    }
}
