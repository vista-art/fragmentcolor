#![cfg(python)]

use lsp_doc::lsp_doc;
use pyo3::prelude::*;

use crate::platforms::python::{PyChannels, PyUnsigned, read_unsigned};
use crate::scene::{
    Camera, CameraError, Light, LightError, LightKind, Model, Scene, SceneLoadError,
};
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
        // Material::clone is a shallow Arc-share, so mutations through the
        // returned handle reach the Model's Material.
        self.material.clone()
    }

    #[pyo3(name = "transform")]
    #[lsp_doc("docs/api/scene/model/transform.md")]
    pub fn transform_py(&self) -> [[f32; 4]; 4] {
        self.transform()
    }

    #[pyo3(name = "set_transform")]
    #[lsp_doc("docs/api/scene/model/set_transform.md")]
    pub fn set_transform_py(&self, matrix: PyMatrix) {
        self.set_transform(matrix.into());
    }

    #[pyo3(name = "translate")]
    #[lsp_doc("docs/api/scene/model/translate.md")]
    pub fn translate_py(&self, offset: PyChannels<'_>) -> PyResult<()> {
        self.translate(offset.read_array("Model.translate")?);
        Ok(())
    }

    #[pyo3(name = "rotate")]
    #[lsp_doc("docs/api/scene/model/rotate.md")]
    pub fn rotate_py(&self, axis: PyChannels<'_>, radians: f32) -> PyResult<()> {
        self.rotate(axis.read_array("Model.rotate: axis")?, radians);
        Ok(())
    }

    #[pyo3(name = "scale")]
    #[lsp_doc("docs/api/scene/model/scale.md")]
    pub fn scale_py(&self, factor: PyChannels<'_>) -> PyResult<()> {
        self.scale(factor.read_array("Model.scale")?);
        Ok(())
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
    pub fn perspective_py(fovy_radians: f32, aspect: f32, near: f32, far: f32) -> PyResult<Self> {
        Ok(Camera::perspective(fovy_radians, aspect, near, far)?)
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
    ) -> PyResult<Self> {
        Ok(Camera::orthographic(left, right, bottom, top, near, far)?)
    }

    #[pyo3(name = "look_at")]
    #[lsp_doc("docs/api/scene/camera/look_at.md")]
    pub fn look_at_py(
        &self,
        position: PyChannels<'_>,
        target: PyChannels<'_>,
        up: PyChannels<'_>,
    ) -> PyResult<Self> {
        Ok(self.look_at(
            position.read_array("Camera.look_at: position")?,
            target.read_array("Camera.look_at: target")?,
            up.read_array("Camera.look_at: up")?,
        )?)
    }

    #[pyo3(name = "set_aspect")]
    #[lsp_doc("docs/api/scene/camera/set_aspect.md")]
    pub fn set_aspect_py(&self, aspect: f32) -> PyResult<Self> {
        Ok(self.set_aspect(aspect)?)
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

/// A 4x4 matrix from Python: four columns of four, or 16 numbers column after column.
#[derive(FromPyObject)]
pub enum PyMatrix {
    #[pyo3(annotation = "Sequence[Sequence[float]]")]
    Columns([[f32; 4]; 4]),
    #[pyo3(annotation = "Sequence[float]")]
    Flat([f32; 16]),
}

impl From<PyMatrix> for [[f32; 4]; 4] {
    fn from(matrix: PyMatrix) -> Self {
        match matrix {
            PyMatrix::Columns(columns) => columns,
            PyMatrix::Flat(flat) => {
                std::array::from_fn(|column| std::array::from_fn(|row| flat[column * 4 + row]))
            }
        }
    }
}

/// A light setter that doesn't fit the light raises `LightError`,
/// a `FragmentColorError` that is also a `ValueError`.
impl From<LightError> for PyErr {
    fn from(error: LightError) -> Self {
        crate::error::PythonError::Light.new_err(error.to_string())
    }
}

/// Refused camera parameters raise `CameraError`,
/// a `FragmentColorError` that is also a `ValueError`.
impl From<CameraError> for PyErr {
    fn from(error: CameraError) -> Self {
        crate::error::PythonError::Camera.new_err(error.to_string())
    }
}

/// A scene that fails to load raises `SceneLoadError`,
/// a `FragmentColorError` that is also a `RuntimeError`.
impl From<SceneLoadError> for PyErr {
    fn from(error: SceneLoadError) -> Self {
        crate::error::PythonError::SceneLoad.new_err(error.to_string())
    }
}

/// A `Scene.load` source from Python: a path, or the bytes of a `.glb` file.
#[derive(FromPyObject)]
pub enum PySceneSource<'py> {
    #[pyo3(annotation = "str")]
    Path(String),
    #[pyo3(annotation = "bytes")]
    Bytes(Bound<'py, pyo3::types::PyBytes>),
    #[pyo3(annotation = "bytearray")]
    ByteArray(Bound<'py, pyo3::types::PyByteArray>),
}

impl From<PySceneSource<'_>> for crate::scene::GltfSource {
    fn from(source: PySceneSource<'_>) -> Self {
        match source {
            PySceneSource::Path(path) => path.into(),
            PySceneSource::Bytes(bytes) => bytes.as_bytes().into(),
            PySceneSource::ByteArray(bytes) => bytes.to_vec().into(),
        }
    }
}

#[pymethods]
impl Light {
    #[staticmethod]
    #[pyo3(name = "directional")]
    #[lsp_doc("docs/api/scene/light/directional.md")]
    pub fn directional_py(direction: PyChannels<'_>, color: PyChannels<'_>) -> PyResult<Self> {
        Ok(Light::directional(
            direction.read_array("Light.directional: direction")?,
            color.read_array("Light.directional: color")?,
        ))
    }

    #[staticmethod]
    #[pyo3(name = "point")]
    #[lsp_doc("docs/api/scene/light/point.md")]
    pub fn point_py(position: PyChannels<'_>, color: PyChannels<'_>) -> PyResult<Self> {
        Ok(Light::point(
            position.read_array("Light.point: position")?,
            color.read_array("Light.point: color")?,
        ))
    }

    #[staticmethod]
    #[pyo3(name = "spot")]
    #[lsp_doc("docs/api/scene/light/spot.md")]
    pub fn spot_py(
        position: PyChannels<'_>,
        direction: PyChannels<'_>,
        color: PyChannels<'_>,
    ) -> PyResult<Self> {
        Ok(Light::spot(
            position.read_array("Light.spot: position")?,
            direction.read_array("Light.spot: direction")?,
            color.read_array("Light.spot: color")?,
        ))
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
    pub fn set_color_py(&self, color: PyChannels<'_>) -> PyResult<Self> {
        Ok(self.set_color(color.read_array("Light.set_color")?))
    }

    #[pyo3(name = "set_intensity")]
    #[lsp_doc("docs/api/scene/light/set_intensity.md")]
    pub fn set_intensity_py(&self, value: f32) -> Self {
        self.set_intensity(value)
    }

    #[pyo3(name = "set_position")]
    #[lsp_doc("docs/api/scene/light/set_position.md")]
    pub fn set_position_py(&self, position: PyChannels<'_>) -> Result<Self, PyErr> {
        Ok(self.set_position(position.read_array("Light.set_position")?)?)
    }

    #[pyo3(name = "set_direction")]
    #[lsp_doc("docs/api/scene/light/set_direction.md")]
    pub fn set_direction_py(&self, direction: PyChannels<'_>) -> Result<Self, PyErr> {
        Ok(self.set_direction(direction.read_array("Light.set_direction")?)?)
    }

    #[pyo3(name = "set_range")]
    #[lsp_doc("docs/api/scene/light/set_range.md")]
    pub fn set_range_py(&self, value: f32) -> Result<Self, PyErr> {
        Ok(self.set_range(value)?)
    }

    #[pyo3(name = "set_cone_angles")]
    #[lsp_doc("docs/api/scene/light/set_cone_angles.md")]
    pub fn set_cone_angles_py(
        &self,
        inner_radians: f32,
        outer_radians: f32,
    ) -> Result<Self, PyErr> {
        Ok(self.set_cone_angles(inner_radians, outer_radians)?)
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
    /// pass `bytes` for an in-memory `.glb` payload. `cameras=False` and
    /// `lights=False` leave out the file's own cameras and lights.
    #[staticmethod]
    #[pyo3(name = "load", signature = (source, cameras=true, lights=true))]
    #[lsp_doc("docs/api/scene/scene/load.md")]
    pub fn load_py(source: PySceneSource<'_>, cameras: bool, lights: bool) -> PyResult<Self> {
        let source = crate::scene::SceneSource::gltf(source)
            .cameras(cameras)
            .lights(lights);
        Ok(Scene::load(source)?)
    }

    /// Unified Scene::add — branches on the runtime Python type. Anything
    /// implementing `SceneObject` on the Rust side gets routed here on
    /// the Python side, and adding a new SceneObject means adding one
    /// extra cast arm below.
    #[pyo3(name = "add")]
    #[lsp_doc("docs/api/scene/scene/add.md")]
    pub fn add_py<'py>(
        slf: PyRef<'py, Self>,
        object: &Bound<'py, PyAny>,
    ) -> PyResult<PyRef<'py, Self>> {
        if let Ok(model) = object.cast::<Model>() {
            slf.add(&*model.borrow())?;
        } else if let Ok(camera) = object.cast::<Camera>() {
            slf.add(&*camera.borrow())?;
        } else if let Ok(light) = object.cast::<Light>() {
            slf.add(&*light.borrow())?;
        } else {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "Scene.add: expected a Model, Camera, or Light",
            ));
        }
        Ok(slf)
    }

    #[pyo3(name = "add_pass")]
    #[lsp_doc("docs/api/scene/scene/add_pass.md")]
    pub fn add_pass_py<'py>(slf: PyRef<'py, Self>, pass: &Pass) -> PyRef<'py, Self> {
        slf.add_pass(pass);
        slf
    }

    #[pyo3(name = "remove_pass")]
    #[lsp_doc("docs/api/scene/scene/remove_pass.md")]
    pub fn remove_pass_py(&self, pass: &Pass) -> bool {
        self.remove_pass(pass)
    }

    #[pyo3(name = "get_pass")]
    #[lsp_doc("docs/api/scene/scene/get_pass.md")]
    pub fn get_pass_py(&self, index: PyUnsigned<'_>) -> PyResult<Option<Pass>> {
        Ok(self.get_pass(index.read("Scene.get_pass: index")? as usize))
    }

    #[pyo3(name = "find_pass")]
    #[lsp_doc("docs/api/scene/scene/find_pass.md")]
    pub fn find_pass_py(&self, name: &str) -> Option<Pass> {
        self.find_pass(name)
    }

    #[pyo3(name = "list_passes")]
    #[lsp_doc("docs/api/scene/scene/list_passes.md")]
    pub fn list_passes_py(&self) -> Vec<Pass> {
        self.list_passes()
    }

    /// Add a SceneObject to a specific Pass, addressed by an `int` index or
    /// a `str` name. Branches on the runtime Python type the same way
    /// `Scene.add` does. A target that is neither a str nor an int raises `TypeError`.
    #[pyo3(name = "add_to")]
    #[lsp_doc("docs/api/scene/scene/add_to.md")]
    pub fn add_to_py<'py>(
        slf: PyRef<'py, Self>,
        target: &Bound<'py, PyAny>,
        object: &Bound<'py, PyAny>,
    ) -> PyResult<PyRef<'py, Self>> {
        let target = match target.extract::<String>() {
            Ok(name) => crate::scene::PassRef::Name(name),
            Err(_) => {
                let index: u32 = read_unsigned(target, "Scene.add_to: target")?;
                crate::scene::PassRef::Index(index as usize)
            }
        };
        if let Ok(model) = object.cast::<Model>() {
            slf.add_to(target, &*model.borrow())?;
        } else if let Ok(camera) = object.cast::<Camera>() {
            slf.add_to(target, &*camera.borrow())?;
        } else if let Ok(light) = object.cast::<Light>() {
            slf.add_to(target, &*light.borrow())?;
        } else {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "Scene.add_to: expected a Model, Camera, or Light",
            ));
        }
        Ok(slf)
    }

    #[pyo3(name = "set_passes")]
    #[lsp_doc("docs/api/scene/scene/set_passes.md")]
    pub fn set_passes_py(&self, passes: Vec<Pass>) {
        self.set_passes(passes);
    }

    #[pyo3(name = "no_defaults")]
    #[lsp_doc("docs/api/scene/scene/no_defaults.md")]
    pub fn no_defaults_py(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf.no_defaults();
        slf
    }

    #[pyo3(name = "no_default_camera")]
    #[lsp_doc("docs/api/scene/scene/no_default_camera.md")]
    pub fn no_default_camera_py(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf.no_default_camera();
        slf
    }

    #[pyo3(name = "no_default_light")]
    #[lsp_doc("docs/api/scene/scene/no_default_light.md")]
    pub fn no_default_light_py(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf.no_default_light();
        slf
    }

    #[pyo3(name = "set_default_camera")]
    #[lsp_doc("docs/api/scene/scene/set_default_camera.md")]
    pub fn set_default_camera_py<'py>(slf: PyRef<'py, Self>, camera: &Camera) -> PyRef<'py, Self> {
        slf.set_default_camera(camera);
        slf
    }

    #[pyo3(name = "set_default_light")]
    #[lsp_doc("docs/api/scene/scene/set_default_light.md")]
    pub fn set_default_light_py<'py>(slf: PyRef<'py, Self>, light: &Light) -> PyRef<'py, Self> {
        slf.set_default_light(light);
        slf
    }

    #[pyo3(name = "ambient")]
    #[lsp_doc("docs/api/scene/scene/ambient.md")]
    pub fn ambient_py<'py>(
        slf: PyRef<'py, Self>,
        color: PyChannels<'_>,
    ) -> PyResult<PyRef<'py, Self>> {
        slf.ambient(color.read_array("Scene.ambient")?);
        Ok(slf)
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
