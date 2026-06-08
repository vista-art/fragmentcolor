#![cfg(wasm)]

use lsp_doc::lsp_doc;
use wasm_bindgen::prelude::*;

use crate::scene::{Camera, Light, LightKind, Model, Scene};
use crate::{Material, Mesh, Pass};

#[wasm_bindgen]
impl Model {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/scene/model/new.md")]
    pub fn new_js(mesh: &Mesh, material: &Material) -> Self {
        // Materials and Meshes are wrapped around Arc internally, so cloning
        // here is an Arc::clone — cheap, share semantics. The JS caller keeps
        // their original handle live.
        Model::new(mesh.clone(), material_share(material))
    }

    #[wasm_bindgen(js_name = "mesh")]
    #[lsp_doc("docs/api/scene/model/mesh.md")]
    pub fn mesh_js(&self) -> Mesh {
        self.mesh.clone()
    }

    #[wasm_bindgen(js_name = "material")]
    #[lsp_doc("docs/api/scene/model/material.md")]
    pub fn material_js(&self) -> Material {
        material_share(&self.material)
    }

    #[wasm_bindgen(js_name = "transform")]
    #[lsp_doc("docs/api/scene/model/transform.md")]
    pub fn transform_js(&self) -> Vec<f32> {
        let cols = self.transform();
        let mut flat = Vec::with_capacity(16);
        for col in cols.iter() {
            flat.extend_from_slice(col);
        }
        flat
    }

    #[wasm_bindgen(js_name = "setTransform")]
    #[lsp_doc("docs/api/scene/model/set_transform.md")]
    pub fn set_transform_js(&self, matrix: Vec<f32>) -> Result<(), JsError> {
        if matrix.len() != 16 {
            return Err(JsError::new(
                "Model.setTransform: expected 16 floats (column-major mat4)",
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

    #[wasm_bindgen(js_name = "translate")]
    #[lsp_doc("docs/api/scene/model/translate.md")]
    pub fn translate_js(&self, offset: Vec<f32>) -> Result<(), JsError> {
        let arr = vec3(&offset, "Model.translate")?;
        self.translate(arr);
        Ok(())
    }

    #[wasm_bindgen(js_name = "rotate")]
    #[lsp_doc("docs/api/scene/model/rotate.md")]
    pub fn rotate_js(&self, axis: Vec<f32>, radians: f32) -> Result<(), JsError> {
        let arr = vec3(&axis, "Model.rotate")?;
        self.rotate(arr, radians);
        Ok(())
    }

    #[wasm_bindgen(js_name = "scale")]
    #[lsp_doc("docs/api/scene/model/scale.md")]
    pub fn scale_js(&self, factor: Vec<f32>) -> Result<(), JsError> {
        let arr = vec3(&factor, "Model.scale")?;
        self.scale(arr);
        Ok(())
    }

    #[wasm_bindgen(js_name = "visible")]
    #[lsp_doc("docs/api/scene/model/visible.md")]
    pub fn visible_js(&self) -> bool {
        self.visible()
    }

    #[wasm_bindgen(js_name = "setVisible")]
    #[lsp_doc("docs/api/scene/model/set_visible.md")]
    pub fn set_visible_js(&self, visible: bool) {
        self.set_visible(visible);
    }
}

#[wasm_bindgen]
impl Camera {
    #[wasm_bindgen(js_name = "perspective")]
    #[lsp_doc("docs/api/scene/camera/perspective.md")]
    pub fn perspective_js(fovy_radians: f32, aspect: f32, near: f32, far: f32) -> Camera {
        Camera::perspective(fovy_radians, aspect, near, far)
    }

    #[wasm_bindgen(js_name = "orthographic")]
    #[lsp_doc("docs/api/scene/camera/orthographic.md")]
    pub fn orthographic_js(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Camera {
        Camera::orthographic(left, right, bottom, top, near, far)
    }

    #[wasm_bindgen(js_name = "lookAt")]
    #[lsp_doc("docs/api/scene/camera/look_at.md")]
    pub fn look_at_js(
        &self,
        position: Vec<f32>,
        target: Vec<f32>,
        up: Vec<f32>,
    ) -> Result<Camera, JsError> {
        let pos = vec3(&position, "Camera.lookAt position")?;
        let target = vec3(&target, "Camera.lookAt target")?;
        let up = vec3(&up, "Camera.lookAt up")?;
        Ok(self.look_at(pos, target, up))
    }

    #[wasm_bindgen(js_name = "setAspect")]
    #[lsp_doc("docs/api/scene/camera/set_aspect.md")]
    pub fn set_aspect_js(&self, aspect: f32) -> Camera {
        self.set_aspect(aspect)
    }

    #[wasm_bindgen(js_name = "viewProj")]
    #[lsp_doc("docs/api/scene/camera/view_proj.md")]
    pub fn view_proj_js(&self) -> Vec<f32> {
        let cols = self.view_proj();
        let mut flat = Vec::with_capacity(16);
        for col in cols.iter() {
            flat.extend_from_slice(col);
        }
        flat
    }

    #[wasm_bindgen(js_name = "position")]
    #[lsp_doc("docs/api/scene/camera/position.md")]
    pub fn position_js(&self) -> Vec<f32> {
        self.position().to_vec()
    }
}

// Bridge so the unified Scene.add / Pass.add dispatchers can detect a Light
// inside a JsValue without exporting the type name in every file.
crate::impl_js_bridge!(Light, crate::PassError);

#[wasm_bindgen]
impl Light {
    #[wasm_bindgen(js_name = "directional")]
    #[lsp_doc("docs/api/scene/light/directional.md")]
    pub fn directional_js(direction: Vec<f32>, color: Vec<f32>) -> Result<Light, JsError> {
        let direction = vec3(&direction, "Light.directional direction")?;
        let color = vec3(&color, "Light.directional color")?;
        Ok(Light::directional(direction, color))
    }

    #[wasm_bindgen(js_name = "point")]
    #[lsp_doc("docs/api/scene/light/point.md")]
    pub fn point_js(position: Vec<f32>, color: Vec<f32>) -> Result<Light, JsError> {
        let position = vec3(&position, "Light.point position")?;
        let color = vec3(&color, "Light.point color")?;
        Ok(Light::point(position, color))
    }

    #[wasm_bindgen(js_name = "spot")]
    #[lsp_doc("docs/api/scene/light/spot.md")]
    pub fn spot_js(
        position: Vec<f32>,
        direction: Vec<f32>,
        color: Vec<f32>,
    ) -> Result<Light, JsError> {
        let position = vec3(&position, "Light.spot position")?;
        let direction = vec3(&direction, "Light.spot direction")?;
        let color = vec3(&color, "Light.spot color")?;
        Ok(Light::spot(position, direction, color))
    }

    #[wasm_bindgen(js_name = "kind")]
    #[lsp_doc("docs/api/scene/light/kind.md")]
    pub fn kind_js(&self) -> String {
        match self.kind() {
            LightKind::Directional => "directional".to_string(),
            LightKind::Point => "point".to_string(),
            LightKind::Spot => "spot".to_string(),
        }
    }

    #[wasm_bindgen(js_name = "color")]
    #[lsp_doc("docs/api/scene/light/color.md")]
    pub fn color_js(&self) -> Vec<f32> {
        self.color().to_vec()
    }

    #[wasm_bindgen(js_name = "intensity")]
    #[lsp_doc("docs/api/scene/light/intensity.md")]
    pub fn intensity_js(&self) -> f32 {
        self.intensity()
    }

    #[wasm_bindgen(js_name = "position")]
    #[lsp_doc("docs/api/scene/light/position.md")]
    pub fn position_js(&self) -> Option<Vec<f32>> {
        self.position().map(|p| p.to_vec())
    }

    #[wasm_bindgen(js_name = "direction")]
    #[lsp_doc("docs/api/scene/light/direction.md")]
    pub fn direction_js(&self) -> Option<Vec<f32>> {
        self.direction().map(|d| d.to_vec())
    }

    #[wasm_bindgen(js_name = "range")]
    #[lsp_doc("docs/api/scene/light/range.md")]
    pub fn range_js(&self) -> Option<f32> {
        self.range()
    }

    #[wasm_bindgen(js_name = "innerConeAngle")]
    #[lsp_doc("docs/api/scene/light/inner_cone_angle.md")]
    pub fn inner_cone_angle_js(&self) -> Option<f32> {
        self.inner_cone_angle()
    }

    #[wasm_bindgen(js_name = "outerConeAngle")]
    #[lsp_doc("docs/api/scene/light/outer_cone_angle.md")]
    pub fn outer_cone_angle_js(&self) -> Option<f32> {
        self.outer_cone_angle()
    }

    #[wasm_bindgen(js_name = "setColor")]
    #[lsp_doc("docs/api/scene/light/set_color.md")]
    pub fn set_color_js(&self, color: Vec<f32>) -> Result<Light, JsError> {
        let color = vec3(&color, "Light.setColor")?;
        Ok(self.set_color(color))
    }

    #[wasm_bindgen(js_name = "setIntensity")]
    #[lsp_doc("docs/api/scene/light/set_intensity.md")]
    pub fn set_intensity_js(&self, value: f32) -> Light {
        self.set_intensity(value)
    }

    #[wasm_bindgen(js_name = "setPosition")]
    #[lsp_doc("docs/api/scene/light/set_position.md")]
    pub fn set_position_js(&self, position: Vec<f32>) -> Result<Light, JsError> {
        let position = vec3(&position, "Light.setPosition")?;
        self.set_position(position)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "setDirection")]
    #[lsp_doc("docs/api/scene/light/set_direction.md")]
    pub fn set_direction_js(&self, direction: Vec<f32>) -> Result<Light, JsError> {
        let direction = vec3(&direction, "Light.setDirection")?;
        self.set_direction(direction)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "setRange")]
    #[lsp_doc("docs/api/scene/light/set_range.md")]
    pub fn set_range_js(&self, value: f32) -> Result<Light, JsError> {
        self.set_range(value)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "setConeAngles")]
    #[lsp_doc("docs/api/scene/light/set_cone_angles.md")]
    pub fn set_cone_angles_js(
        &self,
        inner_radians: f32,
        outer_radians: f32,
    ) -> Result<Light, JsError> {
        self.set_cone_angles(inner_radians, outer_radians)
            .map_err(|e| JsError::new(&e.to_string()))
    }
}

#[wasm_bindgen]
impl Scene {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/scene/scene/new.md")]
    pub fn new_js() -> Self {
        Scene::new()
    }

    /// Load a scene file. Pass a string for a path / URL-like locator, or a
    /// `Uint8Array` for an in-memory `.glb` payload.
    #[wasm_bindgen(js_name = "load")]
    #[lsp_doc("docs/api/scene/scene/load.md")]
    pub fn load_js(source: &JsValue) -> Result<Scene, JsError> {
        let scene_source = if let Some(s) = source.as_string() {
            crate::scene::SceneSource::gltf(s)
        } else if let Ok(arr) = source.dyn_ref::<js_sys::Uint8Array>().ok_or(()) {
            crate::scene::SceneSource::gltf(arr.to_vec())
        } else {
            return Err(JsError::new(
                "Scene.load: expected a path string or a Uint8Array",
            ));
        };
        Scene::load(scene_source).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Unified `Scene.add` — branches on the runtime JS type. Adding a new
    /// `SceneObject` Rust-side means adding one extra try-cast arm here.
    #[wasm_bindgen(js_name = "add")]
    #[lsp_doc("docs/api/scene/scene/add.md")]
    pub fn add_js(&self, object: &JsValue) -> Result<(), JsError> {
        if let Ok(model) = Model::try_from(object) {
            return self.add(&model).map(|_| ()).map_err(|e| e.into());
        }
        if let Ok(camera) = Camera::try_from(object) {
            return self.add(&camera).map(|_| ()).map_err(|e| e.into());
        }
        if let Ok(light) = Light::try_from(object) {
            return self.add(&light).map(|_| ()).map_err(|e| e.into());
        }
        Err(JsError::new(
            "Scene.add: expected a Model, Camera, or Light",
        ))
    }

    #[wasm_bindgen(js_name = "addPass")]
    #[lsp_doc("docs/api/scene/scene/add_pass.md")]
    pub fn add_pass_js(&self, pass: &Pass) {
        self.add_pass(pass);
    }

    #[wasm_bindgen(js_name = "removePass")]
    #[lsp_doc("docs/api/scene/scene/remove_pass.md")]
    pub fn remove_pass_js(&self, pass: &Pass) -> bool {
        self.remove_pass(pass)
    }

    #[wasm_bindgen(js_name = "getPass")]
    #[lsp_doc("docs/api/scene/scene/get_pass.md")]
    pub fn get_pass_js(&self, index: usize) -> Option<Pass> {
        self.get_pass(index)
    }

    #[wasm_bindgen(js_name = "listPasses")]
    #[lsp_doc("docs/api/scene/scene/list_passes.md")]
    pub fn list_passes_js(&self) -> Vec<Pass> {
        self.list_passes()
    }

    #[wasm_bindgen(js_name = "setPasses")]
    #[lsp_doc("docs/api/scene/scene/set_passes.md")]
    pub fn set_passes_js(&self, passes: Vec<Pass>) {
        self.set_passes(passes);
    }

    #[wasm_bindgen(js_name = "noDefaults")]
    #[lsp_doc("docs/api/scene/scene/no_defaults.md")]
    pub fn no_defaults_js(&self) {
        self.no_defaults();
    }

    #[wasm_bindgen(js_name = "noDefaultCamera")]
    #[lsp_doc("docs/api/scene/scene/no_default_camera.md")]
    pub fn no_default_camera_js(&self) {
        self.no_default_camera();
    }

    #[wasm_bindgen(js_name = "noDefaultLight")]
    #[lsp_doc("docs/api/scene/scene/no_default_light.md")]
    pub fn no_default_light_js(&self) {
        self.no_default_light();
    }

    #[wasm_bindgen(js_name = "setDefaultCamera")]
    #[lsp_doc("docs/api/scene/scene/set_default_camera.md")]
    pub fn set_default_camera_js(&self, camera: &Camera) {
        self.set_default_camera(camera);
    }

    #[wasm_bindgen(js_name = "setDefaultLight")]
    #[lsp_doc("docs/api/scene/scene/set_default_light.md")]
    pub fn set_default_light_js(&self, light: &Light) {
        self.set_default_light(light);
    }

    #[wasm_bindgen(js_name = "ambient")]
    #[lsp_doc("docs/api/scene/scene/ambient.md")]
    pub fn ambient_js(&self, color: Vec<f32>) -> Result<(), JsError> {
        let c = vec3(&color, "Scene.ambient")?;
        self.ambient(c);
        Ok(())
    }

    #[wasm_bindgen(js_name = "models")]
    #[lsp_doc("docs/api/scene/scene/models.md")]
    pub fn models_js(&self) -> Vec<Model> {
        self.models()
    }

    #[wasm_bindgen(js_name = "cameras")]
    #[lsp_doc("docs/api/scene/scene/cameras.md")]
    pub fn cameras_js(&self) -> Vec<Camera> {
        self.cameras()
    }

    #[wasm_bindgen(js_name = "lights")]
    #[lsp_doc("docs/api/scene/scene/lights.md")]
    pub fn lights_js(&self) -> Vec<Light> {
        self.lights()
    }
}

// JsValue -> Scene bridge so the Renderer's `render` dispatch can detect a
// Scene the same way it detects a Pass / Shader / Mesh.
crate::impl_js_bridge!(Scene, crate::PassError);
crate::impl_js_bridge!(Model, crate::PassError);
crate::impl_js_bridge!(Camera, crate::PassError);

fn material_share(material: &Material) -> Material {
    // Share the Material's shader (Arc-clone) so the JS handle and the new
    // Material observe the same uniform state. Material::clone is itself a
    // shallow Arc-share — the same shape, just routed through the derive.
    Material::custom(material.shader.clone())
}

fn vec3(v: &[f32], field: &str) -> Result<[f32; 3], JsError> {
    if v.len() != 3 {
        return Err(JsError::new(&format!(
            "{field}: expected an array of length 3"
        )));
    }
    Ok([v[0], v[1], v[2]])
}
