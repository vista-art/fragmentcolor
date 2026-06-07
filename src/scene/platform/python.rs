#![cfg(python)]

use lsp_doc::lsp_doc;
use pyo3::prelude::*;

use crate::scene::{Camera, Light, LightError, LightKind, Model, Scene};
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

    #[pyo3(name = "visible")]
    #[lsp_doc("docs/api/scene/model/visible.md")]
    pub fn visible_py(&self) -> bool {
        self.visible()
    }

    #[pyo3(name = "set_visible")]
    #[lsp_doc("docs/api/scene/model/set_visible.md")]
    pub fn set_visible_py(&self, visible: bool) {
        self.set_visible(visible);
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
    pub fn look_at_py(&self, position: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Self {
        self.look_at(position, target, up)
    }

    #[pyo3(name = "set_aspect")]
    #[lsp_doc("docs/api/scene/camera/set_aspect.md")]
    pub fn set_aspect_py(&self, aspect: f32) -> Self {
        self.set_aspect(aspect)
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

/// Convert a `LightError` into a Python `ValueError`. Used by every
/// kind-specific setter on the `Light` Python binding.
fn light_err_to_py(e: LightError) -> PyErr {
    pyo3::exceptions::PyValueError::new_err(e.to_string())
}

#[pymethods]
impl Light {
    #[staticmethod]
    #[pyo3(name = "directional")]
    #[lsp_doc("docs/api/scene/light/directional.md")]
    pub fn directional_py(direction: [f32; 3], color: [f32; 3]) -> Self {
        Light::directional(direction, color)
    }

    #[staticmethod]
    #[pyo3(name = "point")]
    #[lsp_doc("docs/api/scene/light/point.md")]
    pub fn point_py(position: [f32; 3], color: [f32; 3]) -> Self {
        Light::point(position, color)
    }

    #[staticmethod]
    #[pyo3(name = "spot")]
    #[lsp_doc("docs/api/scene/light/spot.md")]
    pub fn spot_py(position: [f32; 3], direction: [f32; 3], color: [f32; 3]) -> Self {
        Light::spot(position, direction, color)
    }

    #[pyo3(name = "kind")]
    #[lsp_doc("docs/api/scene/light/kind.md")]
    pub fn kind_py(&self) -> &'static str {
        match self.kind() {
            LightKind::Directional => "directional",
            LightKind::Point => "point",
            LightKind::Spot => "spot",
        }
    }

    #[pyo3(name = "color")]
    #[lsp_doc("docs/api/scene/light/color.md")]
    pub fn color_py(&self) -> [f32; 3] {
        self.color()
    }

    #[pyo3(name = "intensity")]
    #[lsp_doc("docs/api/scene/light/intensity.md")]
    pub fn intensity_py(&self) -> f32 {
        self.intensity()
    }

    #[pyo3(name = "position")]
    #[lsp_doc("docs/api/scene/light/position.md")]
    pub fn position_py(&self) -> Option<[f32; 3]> {
        self.position()
    }

    #[pyo3(name = "direction")]
    #[lsp_doc("docs/api/scene/light/direction.md")]
    pub fn direction_py(&self) -> Option<[f32; 3]> {
        self.direction()
    }

    #[pyo3(name = "range")]
    #[lsp_doc("docs/api/scene/light/range.md")]
    pub fn range_py(&self) -> Option<f32> {
        self.range()
    }

    #[pyo3(name = "inner_cone_angle")]
    #[lsp_doc("docs/api/scene/light/inner_cone_angle.md")]
    pub fn inner_cone_angle_py(&self) -> Option<f32> {
        self.inner_cone_angle()
    }

    #[pyo3(name = "outer_cone_angle")]
    #[lsp_doc("docs/api/scene/light/outer_cone_angle.md")]
    pub fn outer_cone_angle_py(&self) -> Option<f32> {
        self.outer_cone_angle()
    }

    #[pyo3(name = "set_color")]
    #[lsp_doc("docs/api/scene/light/set_color.md")]
    pub fn set_color_py(&self, color: [f32; 3]) -> Self {
        self.set_color(color)
    }

    #[pyo3(name = "set_intensity")]
    #[lsp_doc("docs/api/scene/light/set_intensity.md")]
    pub fn set_intensity_py(&self, value: f32) -> Self {
        self.set_intensity(value)
    }

    #[pyo3(name = "set_position")]
    #[lsp_doc("docs/api/scene/light/set_position.md")]
    pub fn set_position_py(&self, position: [f32; 3]) -> Result<Self, PyErr> {
        self.set_position(position).map_err(light_err_to_py)
    }

    #[pyo3(name = "set_direction")]
    #[lsp_doc("docs/api/scene/light/set_direction.md")]
    pub fn set_direction_py(&self, direction: [f32; 3]) -> Result<Self, PyErr> {
        self.set_direction(direction).map_err(light_err_to_py)
    }

    #[pyo3(name = "set_range")]
    #[lsp_doc("docs/api/scene/light/set_range.md")]
    pub fn set_range_py(&self, value: f32) -> Result<Self, PyErr> {
        self.set_range(value).map_err(light_err_to_py)
    }

    #[pyo3(name = "set_cone_angles")]
    #[lsp_doc("docs/api/scene/light/set_cone_angles.md")]
    pub fn set_cone_angles_py(
        &self,
        inner_radians: f32,
        outer_radians: f32,
    ) -> Result<Self, PyErr> {
        self.set_cone_angles(inner_radians, outer_radians)
            .map_err(light_err_to_py)
    }
}

#[pymethods]
impl Scene {
    #[new]
    #[lsp_doc("docs/api/scene/scene/new.md")]
    pub fn new_py() -> Self {
        Scene::new()
    }

    /// Load a scene file. Pass a path string for `.gltf` / `.glb` files;
    /// pass a `bytes` object for an in-memory `.glb` payload.
    #[staticmethod]
    #[pyo3(name = "load")]
    #[lsp_doc("docs/api/scene/scene/load.md")]
    pub fn load_py(source: Py<PyAny>) -> Result<Self, PyErr> {
        Python::attach(|py| -> Result<Self, PyErr> {
            let bound = source.bind(py);
            let scene_source = if let Ok(s) = bound.extract::<String>() {
                crate::scene::SceneSource::gltf(s)
            } else if let Ok(b) = bound.extract::<Vec<u8>>() {
                crate::scene::SceneSource::gltf(b)
            } else {
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "Scene.load: expected a path string or a bytes object",
                ));
            };
            Scene::load(scene_source)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Unified Scene::add — branches on the runtime Python type. Anything
    /// implementing `SceneObject` on the Rust side gets routed here on
    /// the Python side, and adding a new SceneObject means adding one
    /// extra cast arm below.
    #[pyo3(name = "add")]
    #[lsp_doc("docs/api/scene/scene/add.md")]
    pub fn add_py(&self, object: Py<PyAny>) -> Result<(), PyErr> {
        Python::attach(|py| -> Result<(), PyErr> {
            let bound = object.bind(py);
            if let Ok(model) = bound.cast::<Model>() {
                let m = model.borrow();
                return self.add(&*m).map(|_| ()).map_err(|e| e.into());
            }
            if let Ok(camera) = bound.cast::<Camera>() {
                let c = camera.borrow();
                return self.add(&*c).map(|_| ()).map_err(|e| e.into());
            }
            if let Ok(light) = bound.cast::<Light>() {
                let l = light.borrow();
                return self.add(&*l).map(|_| ()).map_err(|e| e.into());
            }
            Err(pyo3::exceptions::PyTypeError::new_err(
                "Scene.add: expected a Model, Camera, or Light",
            ))
        })
    }

    #[pyo3(name = "add_pass")]
    #[lsp_doc("docs/api/scene/scene/add_pass.md")]
    pub fn add_pass_py(&self, pass: &Pass) {
        self.add_pass(pass);
    }

    #[pyo3(name = "ambient")]
    #[lsp_doc("docs/api/scene/scene/ambient.md")]
    pub fn ambient_py(&self, color: [f32; 3]) {
        self.ambient(color);
    }

    #[pyo3(name = "models")]
    #[lsp_doc("docs/api/scene/scene/models.md")]
    pub fn models_py(&self) -> Vec<Model> {
        self.models()
    }

    #[pyo3(name = "cameras")]
    #[lsp_doc("docs/api/scene/scene/cameras.md")]
    pub fn cameras_py(&self) -> Vec<Camera> {
        self.cameras()
    }

    #[pyo3(name = "lights")]
    #[lsp_doc("docs/api/scene/scene/lights.md")]
    pub fn lights_py(&self) -> Vec<Light> {
        self.lights()
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
