#![cfg(mobile)]

use lsp_doc::lsp_doc;
use std::sync::Arc;

use crate::renderer::platform::mobile::FragmentColorError;
use crate::renderer::renderable::SceneObjectHandle;
use crate::scene::{Camera, Light, LightKind, Model, Scene};
use crate::{Material, Mesh, Pass};

/// Mobile-marshallable target for [`Scene::add_to`]: a pass index or a pass
/// name. Uniffi can't marshal `impl Into<PassRef>`, so Swift / Kotlin pick a
/// variant; the `Scene+Extensions` / `SceneExtensions` files wrap it behind
/// natural `addTo(index, object)` / `addTo(name, object)` overloads.
///
/// The index is `u32` (→ Kotlin `UInt`, Swift `UInt32`): a pass position is
/// never negative. Kotlin can't coerce a bare `1` into an unsigned parameter,
/// so the transpiler suffixes the literal (`getPass(1u)`) the same way it does
/// for `setComputeDispatch`.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum PassTarget {
    Index(u32),
    Name(String),
}

#[uniffi::export]
impl Model {
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/scene/model/new.md")]
    pub fn new_mobile(mesh: Arc<Mesh>, material: Arc<Material>) -> Arc<Self> {
        Arc::new(Model::new((*mesh).clone(), material_share(&material)))
    }

    #[uniffi::method(name = "mesh")]
    #[lsp_doc("docs/api/scene/model/mesh.md")]
    pub fn mesh_mobile(self: Arc<Self>) -> Arc<Mesh> {
        Arc::new(self.mesh.clone())
    }

    #[uniffi::method(name = "material")]
    #[lsp_doc("docs/api/scene/model/material.md")]
    pub fn material_mobile(self: Arc<Self>) -> Arc<Material> {
        Arc::new(material_share(&self.material))
    }

    #[uniffi::method(name = "transform")]
    #[lsp_doc("docs/api/scene/model/transform.md")]
    pub fn transform_mobile(self: Arc<Self>) -> Vec<f32> {
        let cols = self.transform();
        let mut out = Vec::with_capacity(16);
        for col in cols.iter() {
            out.extend_from_slice(col);
        }
        out
    }

    #[uniffi::method(name = "setTransform")]
    #[lsp_doc("docs/api/scene/model/set_transform.md")]
    pub fn set_transform_mobile(
        self: Arc<Self>,
        matrix: Vec<f32>,
    ) -> Result<(), FragmentColorError> {
        if matrix.len() != 16 {
            return Err(FragmentColorError::Render(
                "Model.setTransform: expected 16 floats (column-major mat4)".into(),
            ));
        }
        let m = [
            [matrix[0], matrix[1], matrix[2], matrix[3]],
            [matrix[4], matrix[5], matrix[6], matrix[7]],
            [matrix[8], matrix[9], matrix[10], matrix[11]],
            [matrix[12], matrix[13], matrix[14], matrix[15]],
        ];
        self.set_transform(m);
        Ok(())
    }

    #[uniffi::method(name = "translate")]
    #[lsp_doc("docs/api/scene/model/translate.md")]
    pub fn translate_mobile(self: Arc<Self>, offset: Vec<f32>) -> Result<(), FragmentColorError> {
        let arr = take_vec3(&offset, "Model.translate")?;
        self.translate(arr);
        Ok(())
    }

    #[uniffi::method(name = "rotate")]
    #[lsp_doc("docs/api/scene/model/rotate.md")]
    pub fn rotate_mobile(
        self: Arc<Self>,
        axis: Vec<f32>,
        radians: f32,
    ) -> Result<(), FragmentColorError> {
        let arr = take_vec3(&axis, "Model.rotate")?;
        self.rotate(arr, radians);
        Ok(())
    }

    #[uniffi::method(name = "scale")]
    #[lsp_doc("docs/api/scene/model/scale.md")]
    pub fn scale_mobile(self: Arc<Self>, factor: Vec<f32>) -> Result<(), FragmentColorError> {
        let arr = take_vec3(&factor, "Model.scale")?;
        self.scale(arr);
        Ok(())
    }

    #[uniffi::method(name = "visible")]
    #[lsp_doc("docs/api/scene/model/visible.md")]
    pub fn visible_mobile(self: Arc<Self>) -> bool {
        self.visible()
    }

    #[uniffi::method(name = "setVisible")]
    #[lsp_doc("docs/api/scene/model/set_visible.md")]
    pub fn set_visible_mobile(self: Arc<Self>, visible: bool) {
        self.set_visible(visible);
    }
}

#[uniffi::export]
impl Camera {
    #[uniffi::constructor(name = "perspective")]
    #[lsp_doc("docs/api/scene/camera/perspective.md")]
    pub fn perspective_mobile(fovy_radians: f32, aspect: f32, near: f32, far: f32) -> Arc<Self> {
        Arc::new(Camera::perspective(fovy_radians, aspect, near, far))
    }

    #[uniffi::constructor(name = "orthographic")]
    #[lsp_doc("docs/api/scene/camera/orthographic.md")]
    pub fn orthographic_mobile(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Arc<Self> {
        Arc::new(Camera::orthographic(left, right, bottom, top, near, far))
    }

    #[uniffi::method(name = "lookAt")]
    #[lsp_doc("docs/api/scene/camera/look_at.md")]
    pub fn look_at_mobile(
        self: Arc<Self>,
        position: Vec<f32>,
        target: Vec<f32>,
        up: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let pos = take_vec3(&position, "Camera.lookAt position")?;
        let target = take_vec3(&target, "Camera.lookAt target")?;
        let up = take_vec3(&up, "Camera.lookAt up")?;
        Ok(Arc::new(self.look_at(pos, target, up)))
    }

    #[uniffi::method(name = "setAspect")]
    #[lsp_doc("docs/api/scene/camera/set_aspect.md")]
    pub fn set_aspect_mobile(self: Arc<Self>, aspect: f32) -> Arc<Self> {
        Arc::new(self.set_aspect(aspect))
    }

    #[uniffi::method(name = "viewProj")]
    #[lsp_doc("docs/api/scene/camera/view_proj.md")]
    pub fn view_proj_mobile(self: Arc<Self>) -> Vec<f32> {
        let cols = self.view_proj();
        let mut out = Vec::with_capacity(16);
        for col in cols.iter() {
            out.extend_from_slice(col);
        }
        out
    }

    #[uniffi::method(name = "position")]
    #[lsp_doc("docs/api/scene/camera/position.md")]
    pub fn position_mobile(self: Arc<Self>) -> Vec<f32> {
        self.position().to_vec()
    }
}

#[uniffi::export]
impl Light {
    #[uniffi::constructor(name = "directional")]
    #[lsp_doc("docs/api/scene/light/directional.md")]
    pub fn directional_mobile(
        direction: Vec<f32>,
        color: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let direction = take_vec3(&direction, "Light.directional direction")?;
        let color = take_vec3(&color, "Light.directional color")?;
        Ok(Arc::new(Light::directional(direction, color)))
    }

    #[uniffi::constructor(name = "point")]
    #[lsp_doc("docs/api/scene/light/point.md")]
    pub fn point_mobile(
        position: Vec<f32>,
        color: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let position = take_vec3(&position, "Light.point position")?;
        let color = take_vec3(&color, "Light.point color")?;
        Ok(Arc::new(Light::point(position, color)))
    }

    #[uniffi::constructor(name = "spot")]
    #[lsp_doc("docs/api/scene/light/spot.md")]
    pub fn spot_mobile(
        position: Vec<f32>,
        direction: Vec<f32>,
        color: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let position = take_vec3(&position, "Light.spot position")?;
        let direction = take_vec3(&direction, "Light.spot direction")?;
        let color = take_vec3(&color, "Light.spot color")?;
        Ok(Arc::new(Light::spot(position, direction, color)))
    }

    #[uniffi::method(name = "kind")]
    #[lsp_doc("docs/api/scene/light/kind.md")]
    pub fn kind_mobile(self: Arc<Self>) -> String {
        match self.kind() {
            LightKind::Directional => "directional".to_string(),
            LightKind::Point => "point".to_string(),
            LightKind::Spot => "spot".to_string(),
        }
    }

    #[uniffi::method(name = "color")]
    #[lsp_doc("docs/api/scene/light/color.md")]
    pub fn color_mobile(self: Arc<Self>) -> Vec<f32> {
        self.color().to_vec()
    }

    #[uniffi::method(name = "intensity")]
    #[lsp_doc("docs/api/scene/light/intensity.md")]
    pub fn intensity_mobile(self: Arc<Self>) -> f32 {
        self.intensity()
    }

    #[uniffi::method(name = "position")]
    #[lsp_doc("docs/api/scene/light/position.md")]
    pub fn position_mobile(self: Arc<Self>) -> Option<Vec<f32>> {
        self.position().map(|p| p.to_vec())
    }

    #[uniffi::method(name = "direction")]
    #[lsp_doc("docs/api/scene/light/direction.md")]
    pub fn direction_mobile(self: Arc<Self>) -> Option<Vec<f32>> {
        self.direction().map(|d| d.to_vec())
    }

    #[uniffi::method(name = "range")]
    #[lsp_doc("docs/api/scene/light/range.md")]
    pub fn range_mobile(self: Arc<Self>) -> Option<f32> {
        self.range()
    }

    #[uniffi::method(name = "innerConeAngle")]
    #[lsp_doc("docs/api/scene/light/inner_cone_angle.md")]
    pub fn inner_cone_angle_mobile(self: Arc<Self>) -> Option<f32> {
        self.inner_cone_angle()
    }

    #[uniffi::method(name = "outerConeAngle")]
    #[lsp_doc("docs/api/scene/light/outer_cone_angle.md")]
    pub fn outer_cone_angle_mobile(self: Arc<Self>) -> Option<f32> {
        self.outer_cone_angle()
    }

    #[uniffi::method(name = "setColor")]
    #[lsp_doc("docs/api/scene/light/set_color.md")]
    pub fn set_color_mobile(
        self: Arc<Self>,
        color: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let color = take_vec3(&color, "Light.setColor")?;
        Ok(Arc::new(self.set_color(color)))
    }

    #[uniffi::method(name = "setIntensity")]
    #[lsp_doc("docs/api/scene/light/set_intensity.md")]
    pub fn set_intensity_mobile(self: Arc<Self>, value: f32) -> Arc<Self> {
        Arc::new(self.set_intensity(value))
    }

    #[uniffi::method(name = "setPosition")]
    #[lsp_doc("docs/api/scene/light/set_position.md")]
    pub fn set_position_mobile(
        self: Arc<Self>,
        position: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let position = take_vec3(&position, "Light.setPosition")?;
        self.set_position(position)
            .map(Arc::new)
            .map_err(|e| FragmentColorError::Render(e.to_string()))
    }

    #[uniffi::method(name = "setDirection")]
    #[lsp_doc("docs/api/scene/light/set_direction.md")]
    pub fn set_direction_mobile(
        self: Arc<Self>,
        direction: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let direction = take_vec3(&direction, "Light.setDirection")?;
        self.set_direction(direction)
            .map(Arc::new)
            .map_err(|e| FragmentColorError::Render(e.to_string()))
    }

    #[uniffi::method(name = "setRange")]
    #[lsp_doc("docs/api/scene/light/set_range.md")]
    pub fn set_range_mobile(self: Arc<Self>, value: f32) -> Result<Arc<Self>, FragmentColorError> {
        self.set_range(value)
            .map(Arc::new)
            .map_err(|e| FragmentColorError::Render(e.to_string()))
    }

    #[uniffi::method(name = "setConeAngles")]
    #[lsp_doc("docs/api/scene/light/set_cone_angles.md")]
    pub fn set_cone_angles_mobile(
        self: Arc<Self>,
        inner_radians: f32,
        outer_radians: f32,
    ) -> Result<Arc<Self>, FragmentColorError> {
        self.set_cone_angles(inner_radians, outer_radians)
            .map(Arc::new)
            .map_err(|e| FragmentColorError::Render(e.to_string()))
    }
}

#[uniffi::export]
impl Scene {
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/scene/scene/new.md")]
    pub fn new_mobile() -> Arc<Self> {
        Arc::new(Scene::new())
    }

    /// Load a scene from a file on disk. Pass a path for `.gltf` / `.glb`
    /// files. In-memory `.glb` bytes are not exposed on mobile yet — call
    /// from Rust if you need that shape.
    #[uniffi::constructor(name = "load")]
    #[lsp_doc("docs/api/scene/scene/load.md")]
    pub fn load_mobile(path: String) -> Result<Arc<Self>, FragmentColorError> {
        Scene::load(crate::scene::SceneSource::gltf(path))
            .map(Arc::new)
            .map_err(|e| FragmentColorError::Render(e.to_string()))
    }

    /// Unified `Scene.add` — branches on the runtime mobile handle. Adding
    /// a new `SceneObject` Rust-side means adding one extra variant to
    /// [`SceneObjectHandle`](crate::SceneObjectHandle) and one arm here.
    #[uniffi::method(name = "add")]
    #[lsp_doc("docs/api/scene/scene/add.md")]
    pub fn add_mobile(
        self: Arc<Self>,
        object: SceneObjectHandle,
    ) -> Result<(), FragmentColorError> {
        match object {
            SceneObjectHandle::Model(m) => self
                .add(m.as_ref())
                .map(|_| ())
                .map_err(|e| FragmentColorError::Render(e.to_string())),
            SceneObjectHandle::Camera(c) => self
                .add(c.as_ref())
                .map(|_| ())
                .map_err(|e| FragmentColorError::Render(e.to_string())),
            SceneObjectHandle::Light(l) => self
                .add(l.as_ref())
                .map(|_| ())
                .map_err(|e| FragmentColorError::Render(e.to_string())),
        }
    }

    #[uniffi::method(name = "addPass")]
    #[lsp_doc("docs/api/scene/scene/add_pass.md")]
    pub fn add_pass_mobile(self: Arc<Self>, pass: Arc<Pass>) {
        self.add_pass(pass.as_ref());
    }

    #[uniffi::method(name = "removePass")]
    #[lsp_doc("docs/api/scene/scene/remove_pass.md")]
    pub fn remove_pass_mobile(self: Arc<Self>, pass: Arc<Pass>) -> bool {
        self.remove_pass(pass.as_ref())
    }

    #[uniffi::method(name = "getPass")]
    #[lsp_doc("docs/api/scene/scene/get_pass.md")]
    pub fn get_pass_mobile(self: Arc<Self>, index: u32) -> Option<Arc<Pass>> {
        self.get_pass(index as usize).map(Arc::new)
    }

    #[uniffi::method(name = "findPass")]
    #[lsp_doc("docs/api/scene/scene/find_pass.md")]
    pub fn find_pass_mobile(self: Arc<Self>, name: String) -> Option<Arc<Pass>> {
        self.find_pass(&name).map(Arc::new)
    }

    #[uniffi::method(name = "listPasses")]
    #[lsp_doc("docs/api/scene/scene/list_passes.md")]
    pub fn list_passes_mobile(self: Arc<Self>) -> Vec<Arc<Pass>> {
        self.list_passes().into_iter().map(Arc::new).collect()
    }

    /// Add a SceneObject to a specific Pass, addressed by index or name via
    /// [`PassTarget`]. The Swift / Kotlin extension files supply natural
    /// `addTo(index, object)` / `addTo(name, object)` overloads on top.
    #[uniffi::method(name = "addTo")]
    #[lsp_doc("docs/api/scene/scene/add_to.md")]
    pub fn add_to_mobile(
        self: Arc<Self>,
        target: PassTarget,
        object: SceneObjectHandle,
    ) -> Result<(), FragmentColorError> {
        let target = match target {
            PassTarget::Index(index) => crate::scene::PassRef::Index(index as usize),
            PassTarget::Name(name) => crate::scene::PassRef::Name(name),
        };
        let result = match object {
            SceneObjectHandle::Model(m) => self.add_to(target, m.as_ref()).map(|_| ()),
            SceneObjectHandle::Camera(c) => self.add_to(target, c.as_ref()).map(|_| ()),
            SceneObjectHandle::Light(l) => self.add_to(target, l.as_ref()).map(|_| ()),
        };
        result.map_err(|e| FragmentColorError::Render(e.to_string()))
    }

    #[uniffi::method(name = "setPasses")]
    #[lsp_doc("docs/api/scene/scene/set_passes.md")]
    pub fn set_passes_mobile(self: Arc<Self>, passes: Vec<Arc<Pass>>) {
        self.set_passes(passes.iter().map(|p| (**p).clone()).collect());
    }

    #[uniffi::method(name = "noDefaults")]
    #[lsp_doc("docs/api/scene/scene/no_defaults.md")]
    pub fn no_defaults_mobile(self: Arc<Self>) {
        self.no_defaults();
    }

    #[uniffi::method(name = "noDefaultCamera")]
    #[lsp_doc("docs/api/scene/scene/no_default_camera.md")]
    pub fn no_default_camera_mobile(self: Arc<Self>) {
        self.no_default_camera();
    }

    #[uniffi::method(name = "noDefaultLight")]
    #[lsp_doc("docs/api/scene/scene/no_default_light.md")]
    pub fn no_default_light_mobile(self: Arc<Self>) {
        self.no_default_light();
    }

    #[uniffi::method(name = "setDefaultCamera")]
    #[lsp_doc("docs/api/scene/scene/set_default_camera.md")]
    pub fn set_default_camera_mobile(self: Arc<Self>, camera: Arc<Camera>) {
        self.set_default_camera(camera.as_ref());
    }

    #[uniffi::method(name = "setDefaultLight")]
    #[lsp_doc("docs/api/scene/scene/set_default_light.md")]
    pub fn set_default_light_mobile(self: Arc<Self>, light: Arc<Light>) {
        self.set_default_light(light.as_ref());
    }

    #[uniffi::method(name = "ambient")]
    #[lsp_doc("docs/api/scene/scene/ambient.md")]
    pub fn ambient_mobile(self: Arc<Self>, color: Vec<f32>) -> Result<(), FragmentColorError> {
        let c = take_vec3(&color, "Scene.ambient")?;
        self.ambient(c);
        Ok(())
    }

    #[uniffi::method(name = "models")]
    #[lsp_doc("docs/api/scene/scene/models.md")]
    pub fn models_mobile(self: Arc<Self>) -> Vec<Arc<Model>> {
        self.models().into_iter().map(Arc::new).collect()
    }

    #[uniffi::method(name = "cameras")]
    #[lsp_doc("docs/api/scene/scene/cameras.md")]
    pub fn cameras_mobile(self: Arc<Self>) -> Vec<Arc<Camera>> {
        self.cameras().into_iter().map(Arc::new).collect()
    }

    #[uniffi::method(name = "lights")]
    #[lsp_doc("docs/api/scene/scene/lights.md")]
    pub fn lights_mobile(self: Arc<Self>) -> Vec<Arc<Light>> {
        self.lights().into_iter().map(Arc::new).collect()
    }
}

fn material_share(material: &Material) -> Material {
    Material::custom(material.shader.clone())
}

fn take_vec3(v: &[f32], field: &str) -> Result<[f32; 3], FragmentColorError> {
    if v.len() != 3 {
        return Err(FragmentColorError::Render(format!(
            "{field}: expected an array of length 3, got {}",
            v.len()
        )));
    }
    Ok([v[0], v[1], v[2]])
}
