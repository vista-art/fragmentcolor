#![cfg(wasm)]

use crate::error::{ErrorClass, WebError};
use lsp_doc::lsp_doc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::bindings::web::{
    read_bytes, read_count, read_flag, read_float, read_text, read_unsigned, read_vector,
    read_vector_items,
};
use crate::scene::{Camera, Light, LightKind, Model, Scene, SceneSource};
use crate::{Material, Mesh, Pass};

#[wasm_bindgen]
impl Model {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/scene/model/new.md")]
    pub fn new_js(mesh: &Mesh, material: &Material) -> Self {
        // Materials and Meshes are wrapped around Arc internally, so cloning
        // here is an Arc::clone — cheap, share semantics. The JS caller keeps
        // their original handle live.
        Model::new(mesh.clone(), material.clone())
    }

    #[wasm_bindgen(js_name = "mesh")]
    #[lsp_doc("docs/api/scene/model/mesh.md")]
    pub fn mesh_js(&self) -> Mesh {
        self.mesh.clone()
    }

    #[wasm_bindgen(js_name = "material")]
    #[lsp_doc("docs/api/scene/model/material.md")]
    pub fn material_js(&self) -> Material {
        self.material.clone()
    }

    #[wasm_bindgen(js_name = "transform")]
    #[lsp_doc("docs/api/scene/model/transform.md")]
    pub fn transform_js(&self) -> Vec<js_sys::Float32Array> {
        list_columns(self.transform())
    }

    #[wasm_bindgen(js_name = "setTransform")]
    #[lsp_doc("docs/api/scene/model/set_transform.md")]
    pub fn set_transform_js(
        &self,
        #[wasm_bindgen(
            unchecked_param_type = "number[] | Float32Array | (number[] | Float32Array)[]"
        )]
        matrix: &JsValue,
    ) -> Result<(), WebError> {
        self.set_transform(read_matrix(matrix)?);
        Ok(())
    }

    #[wasm_bindgen(js_name = "translate")]
    #[lsp_doc("docs/api/scene/model/translate.md")]
    pub fn translate_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] offset: &JsValue,
    ) -> Result<(), WebError> {
        let arr = read_vector::<3>(offset, "Model.translate")?;
        self.translate(arr);
        Ok(())
    }

    #[wasm_bindgen(js_name = "rotate")]
    #[lsp_doc("docs/api/scene/model/rotate.md")]
    pub fn rotate_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] axis: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] radians: &JsValue,
    ) -> Result<(), WebError> {
        let arr = read_vector::<3>(axis, "Model.rotate: axis")?;
        self.rotate(arr, read_float(radians, "Model.rotate: radians")?);
        Ok(())
    }

    #[wasm_bindgen(js_name = "scale")]
    #[lsp_doc("docs/api/scene/model/scale.md")]
    pub fn scale_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] factor: &JsValue,
    ) -> Result<(), WebError> {
        let arr = read_vector::<3>(factor, "Model.scale")?;
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
    pub fn set_visible_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "boolean")] visible: &JsValue,
    ) -> Result<(), WebError> {
        self.set_visible(read_flag(visible, "Model.setVisible: visible")?);
        Ok(())
    }
}

#[wasm_bindgen]
impl Camera {
    #[wasm_bindgen(js_name = "perspective")]
    #[lsp_doc("docs/api/scene/camera/perspective.md")]
    pub fn perspective_js(
        #[wasm_bindgen(unchecked_param_type = "number")] fovy_radians: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] aspect: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] near: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] far: &JsValue,
    ) -> Result<Camera, WebError> {
        Ok(Camera::perspective(
            read_float(fovy_radians, "Camera.perspective: fovy_radians")?,
            read_float(aspect, "Camera.perspective: aspect")?,
            read_float(near, "Camera.perspective: near")?,
            read_float(far, "Camera.perspective: far")?,
        )?)
    }

    #[wasm_bindgen(js_name = "orthographic")]
    #[lsp_doc("docs/api/scene/camera/orthographic.md")]
    pub fn orthographic_js(
        #[wasm_bindgen(unchecked_param_type = "number")] left: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] right: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] bottom: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] top: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] near: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] far: &JsValue,
    ) -> Result<Camera, WebError> {
        Ok(Camera::orthographic(
            read_float(left, "Camera.orthographic: left")?,
            read_float(right, "Camera.orthographic: right")?,
            read_float(bottom, "Camera.orthographic: bottom")?,
            read_float(top, "Camera.orthographic: top")?,
            read_float(near, "Camera.orthographic: near")?,
            read_float(far, "Camera.orthographic: far")?,
        )?)
    }

    #[wasm_bindgen(js_name = "lookAt")]
    #[lsp_doc("docs/api/scene/camera/look_at.md")]
    pub fn look_at_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] position: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] target: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] up: &JsValue,
    ) -> Result<Camera, WebError> {
        let pos = read_vector::<3>(position, "Camera.lookAt: position")?;
        let target = read_vector::<3>(target, "Camera.lookAt: target")?;
        let up = read_vector::<3>(up, "Camera.lookAt: up")?;
        Ok(self.look_at(pos, target, up)?)
    }

    #[wasm_bindgen(js_name = "setAspect")]
    #[lsp_doc("docs/api/scene/camera/set_aspect.md")]
    pub fn set_aspect_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] aspect: &JsValue,
    ) -> Result<Camera, WebError> {
        Ok(self.set_aspect(read_float(aspect, "Camera.setAspect: aspect")?)?)
    }

    #[wasm_bindgen(js_name = "viewProj")]
    #[lsp_doc("docs/api/scene/camera/view_proj.md")]
    pub fn view_proj_js(&self) -> Vec<js_sys::Float32Array> {
        list_columns(self.view_proj())
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
    pub fn directional_js(
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] direction: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] color: &JsValue,
    ) -> Result<Light, WebError> {
        let direction = read_vector::<3>(direction, "Light.directional: direction")?;
        let color = read_vector::<3>(color, "Light.directional: color")?;
        Ok(Light::directional(direction, color))
    }

    #[wasm_bindgen(js_name = "point")]
    #[lsp_doc("docs/api/scene/light/point.md")]
    pub fn point_js(
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] position: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] color: &JsValue,
    ) -> Result<Light, WebError> {
        let position = read_vector::<3>(position, "Light.point: position")?;
        let color = read_vector::<3>(color, "Light.point: color")?;
        Ok(Light::point(position, color))
    }

    #[wasm_bindgen(js_name = "spot")]
    #[lsp_doc("docs/api/scene/light/spot.md")]
    pub fn spot_js(
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] position: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] direction: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] color: &JsValue,
    ) -> Result<Light, WebError> {
        let position = read_vector::<3>(position, "Light.spot: position")?;
        let direction = read_vector::<3>(direction, "Light.spot: direction")?;
        let color = read_vector::<3>(color, "Light.spot: color")?;
        Ok(Light::spot(position, direction, color))
    }

    #[wasm_bindgen(js_name = "kind")]
    #[lsp_doc("docs/api/scene/light/kind.md")]
    pub fn kind_js(&self) -> LightKind {
        self.kind()
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
    pub fn set_color_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] color: &JsValue,
    ) -> Result<Light, WebError> {
        let color = read_vector::<3>(color, "Light.setColor")?;
        Ok(self.set_color(color))
    }

    #[wasm_bindgen(js_name = "setIntensity")]
    #[lsp_doc("docs/api/scene/light/set_intensity.md")]
    pub fn set_intensity_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] value: &JsValue,
    ) -> Result<Light, WebError> {
        Ok(self.set_intensity(read_float(value, "Light.setIntensity: value")?))
    }

    #[wasm_bindgen(js_name = "setPosition")]
    #[lsp_doc("docs/api/scene/light/set_position.md")]
    pub fn set_position_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] position: &JsValue,
    ) -> Result<Light, WebError> {
        let position = read_vector::<3>(position, "Light.setPosition")?;
        Ok(self.set_position(position)?)
    }

    #[wasm_bindgen(js_name = "setDirection")]
    #[lsp_doc("docs/api/scene/light/set_direction.md")]
    pub fn set_direction_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] direction: &JsValue,
    ) -> Result<Light, WebError> {
        let direction = read_vector::<3>(direction, "Light.setDirection")?;
        Ok(self.set_direction(direction)?)
    }

    #[wasm_bindgen(js_name = "setRange")]
    #[lsp_doc("docs/api/scene/light/set_range.md")]
    pub fn set_range_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] value: &JsValue,
    ) -> Result<Light, WebError> {
        Ok(self.set_range(read_float(value, "Light.setRange: value")?)?)
    }

    #[wasm_bindgen(js_name = "setConeAngles")]
    #[lsp_doc("docs/api/scene/light/set_cone_angles.md")]
    pub fn set_cone_angles_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] inner_radians: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] outer_radians: &JsValue,
    ) -> Result<Light, WebError> {
        let inner = read_float(inner_radians, "Light.setConeAngles: inner_radians")?;
        let outer = read_float(outer_radians, "Light.setConeAngles: outer_radians")?;
        Ok(self.set_cone_angles(inner, outer)?)
    }
}

#[wasm_bindgen]
impl Scene {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/scene/scene/new.md")]
    pub fn new_js() -> Self {
        Scene::new()
    }

    /// Load a scene from the bytes of a `.glb` file, as an `ArrayBuffer`, a
    /// `Uint8Array` or any other view. The web build can't read a path, so
    /// fetch the file first. `options` can skip the file's cameras and lights.
    #[wasm_bindgen(js_name = "load")]
    #[lsp_doc("docs/api/scene/scene/load.md")]
    pub fn load_js(
        #[wasm_bindgen(unchecked_param_type = "ArrayBuffer | ArrayBufferView")] source: &JsValue,
        #[wasm_bindgen(
            unchecked_optional_param_type = "{ cameras?: boolean; lights?: boolean } | null"
        )]
        options: Option<JsValue>,
    ) -> Result<Scene, WebError> {
        let source = if let Some(bytes) = read_bytes(source) {
            SceneSource::gltf(bytes)
        } else if let Some(path) = source.as_string() {
            SceneSource::gltf(path)
        } else {
            return Err(WebError::type_error(
                "Scene.load: expected the bytes of a .glb file, as an ArrayBuffer or a typed array",
            ));
        };
        let cameras = read_option(options.as_ref(), "cameras")?;
        let lights = read_option(options.as_ref(), "lights")?;
        Ok(Scene::load(source.cameras(cameras).lights(lights))?)
    }

    /// Unified `Scene.add`, which branches on the runtime JS type. Adding a new
    /// `SceneObject` on the Rust side means adding one try-cast arm here.
    #[wasm_bindgen(js_name = "add")]
    #[lsp_doc("docs/api/scene/scene/add.md")]
    pub fn add_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "Model | Camera | Light")] object: &JsValue,
    ) -> Result<Scene, WebError> {
        if let Ok(model) = Model::try_from(object) {
            self.add(&model)?;
        } else if let Ok(camera) = Camera::try_from(object) {
            self.add(&camera)?;
        } else if let Ok(light) = Light::try_from(object) {
            self.add(&light)?;
        } else {
            return Err(WebError::type_error(
                "Scene.add: expected a Model, Camera, or Light",
            ));
        }
        Ok(self.clone())
    }

    #[wasm_bindgen(js_name = "addPass")]
    #[lsp_doc("docs/api/scene/scene/add_pass.md")]
    pub fn add_pass_js(&self, pass: &Pass) -> Scene {
        self.add_pass(pass).clone()
    }

    #[wasm_bindgen(js_name = "removePass")]
    #[lsp_doc("docs/api/scene/scene/remove_pass.md")]
    pub fn remove_pass_js(&self, pass: &Pass) -> bool {
        self.remove_pass(pass)
    }

    #[wasm_bindgen(js_name = "getPass")]
    #[lsp_doc("docs/api/scene/scene/get_pass.md")]
    pub fn get_pass_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] index: &JsValue,
    ) -> Result<Option<Pass>, WebError> {
        let index = read_count(index, "Scene.getPass: index")?;
        Ok(self.get_pass(index as usize))
    }

    #[wasm_bindgen(js_name = "findPass")]
    #[lsp_doc("docs/api/scene/scene/find_pass.md")]
    pub fn find_pass_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "string")] name: &JsValue,
    ) -> Result<Option<Pass>, WebError> {
        Ok(self.find_pass(&read_text(name, "Scene.findPass: name")?))
    }

    #[wasm_bindgen(js_name = "listPasses")]
    #[lsp_doc("docs/api/scene/scene/list_passes.md")]
    pub fn list_passes_js(&self) -> Vec<Pass> {
        self.list_passes()
    }

    /// Add a SceneObject to a specific Pass, addressed by a numeric index or
    /// a string name. Branches on the runtime JS type like `Scene.add`.
    #[wasm_bindgen(js_name = "addTo")]
    #[lsp_doc("docs/api/scene/scene/add_to.md")]
    pub fn add_to_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number | string")] target: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "Model | Camera | Light")] object: &JsValue,
    ) -> Result<Scene, WebError> {
        let target = if let Some(index) = target.as_f64() {
            crate::scene::PassRef::Index(read_unsigned(index, "Scene.addTo: index")? as usize)
        } else if let Some(name) = target.as_string() {
            crate::scene::PassRef::Name(name)
        } else {
            return Err(WebError::type_error(
                "Scene.addTo: target must be an index or a name",
            ));
        };
        if let Ok(model) = Model::try_from(object) {
            self.add_to(target, &model)?;
        } else if let Ok(camera) = Camera::try_from(object) {
            self.add_to(target, &camera)?;
        } else if let Ok(light) = Light::try_from(object) {
            self.add_to(target, &light)?;
        } else {
            return Err(WebError::type_error(
                "Scene.addTo: expected a Model, Camera, or Light",
            ));
        }
        Ok(self.clone())
    }

    #[wasm_bindgen(js_name = "setPasses")]
    #[lsp_doc("docs/api/scene/scene/set_passes.md")]
    pub fn set_passes_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "Pass[]")] passes: &JsValue,
    ) -> Result<(), WebError> {
        self.set_passes(read_passes(passes)?);
        Ok(())
    }

    #[wasm_bindgen(js_name = "noDefaults")]
    #[lsp_doc("docs/api/scene/scene/no_defaults.md")]
    pub fn no_defaults_js(&self) -> Scene {
        self.no_defaults().clone()
    }

    #[wasm_bindgen(js_name = "noDefaultCamera")]
    #[lsp_doc("docs/api/scene/scene/no_default_camera.md")]
    pub fn no_default_camera_js(&self) -> Scene {
        self.no_default_camera().clone()
    }

    #[wasm_bindgen(js_name = "noDefaultLight")]
    #[lsp_doc("docs/api/scene/scene/no_default_light.md")]
    pub fn no_default_light_js(&self) -> Scene {
        self.no_default_light().clone()
    }

    #[wasm_bindgen(js_name = "setDefaultCamera")]
    #[lsp_doc("docs/api/scene/scene/set_default_camera.md")]
    pub fn set_default_camera_js(&self, camera: &Camera) -> Scene {
        self.set_default_camera(camera).clone()
    }

    #[wasm_bindgen(js_name = "setDefaultLight")]
    #[lsp_doc("docs/api/scene/scene/set_default_light.md")]
    pub fn set_default_light_js(&self, light: &Light) -> Scene {
        self.set_default_light(light).clone()
    }

    #[wasm_bindgen(js_name = "ambient")]
    #[lsp_doc("docs/api/scene/scene/ambient.md")]
    pub fn ambient_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] color: &JsValue,
    ) -> Result<Scene, WebError> {
        let color = read_vector::<3>(color, "Scene.ambient")?;
        Ok(self.ambient(color).clone())
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

/// Reads a column-major 4x4 matrix: 16 numbers, or four columns of four, each list an array
/// or a `Float32Array` as a float vector is. Any other list, or another count, throws naming the call.
fn read_matrix(value: &JsValue) -> Result<[[f32; 4]; 4], WebError> {
    const LABEL: &str = "Model.setTransform";
    let columns = match value.dyn_ref::<js_sys::Array>() {
        Some(array) if array.length() > 0 && array.iter().all(|column| column.is_object()) => {
            array.to_vec()
        }
        _ => return read_flat_matrix(value, LABEL),
    };
    if columns.len() != 4 {
        return Err(WebError::new(
            ErrorClass::InvalidInput,
            &format!(
                "{LABEL} takes 4 columns of 4 numbers, got {} columns",
                columns.len()
            ),
        ));
    }
    let mut matrix = [[0.0; 4]; 4];
    for (index, (column, slot)) in columns.iter().zip(matrix.iter_mut()).enumerate() {
        let label = format!("{LABEL}: column {index}");
        let numbers = read_vector_items(column, &label)?;
        *slot = match <[f64; 4]>::try_from(numbers.as_slice()) {
            Ok(column) => column.map(|number| number as f32),
            Err(_) => {
                return Err(WebError::new(
                    ErrorClass::InvalidInput,
                    &format!("{label} takes 4 numbers, got {}", numbers.len()),
                ));
            }
        };
    }
    Ok(matrix)
}

/// A 4x4 matrix as its four columns, each a `Float32Array` of four, as Rust's `[[f32; 4]; 4]` holds it.
fn list_columns(matrix: [[f32; 4]; 4]) -> Vec<js_sys::Float32Array> {
    matrix
        .iter()
        .map(|column| js_sys::Float32Array::from(column.as_slice()))
        .collect()
}

/// A matrix of 16 numbers, column after column.
fn read_flat_matrix(value: &JsValue, label: &str) -> Result<[[f32; 4]; 4], WebError> {
    let numbers = read_vector_items(value, label)?;
    match <[f64; 16]>::try_from(numbers.as_slice()) {
        Ok(flat) => Ok(std::array::from_fn(|column| {
            std::array::from_fn(|row| flat[column * 4 + row] as f32)
        })),
        Err(_) => Err(WebError::new(
            ErrorClass::InvalidInput,
            &format!(
                "{label} takes 16 numbers or 4 columns of 4 numbers, got {} numbers",
                numbers.len()
            ),
        )),
    }
}

/// Reads one boolean flag of the `Scene.load` options. A missing flag is `true`.
fn read_option(options: Option<&JsValue>, name: &str) -> Result<bool, WebError> {
    let Some(options) = options.filter(|options| !options.is_undefined() && !options.is_null())
    else {
        return Ok(true);
    };
    if !options.is_object() {
        return Err(WebError::type_error(
            "Scene.load: options must be an object",
        ));
    }
    match js_sys::Reflect::get(options, &name.into()) {
        Ok(value) if value.is_undefined() => Ok(true),
        Ok(value) => value.as_bool().ok_or_else(|| {
            WebError::type_error(&format!("Scene.load: options.{name} must be a boolean"))
        }),
        Err(_) => Ok(true),
    }
}

/// Reads an array of passes. Each item is cloned through its handle,
/// so the caller's `Pass` objects stay usable afterwards.
fn read_passes(value: &JsValue) -> Result<Vec<Pass>, WebError> {
    let Some(array) = value.dyn_ref::<js_sys::Array>() else {
        return Err(WebError::type_error(
            "Scene.setPasses: expected an array of Pass",
        ));
    };
    let read = |(index, item): (usize, JsValue)| {
        Pass::try_from(&item).map_err(|_| {
            WebError::type_error(&format!("Scene.setPasses: item {index} is not a Pass"))
        })
    };
    array.iter().enumerate().map(read).collect()
}
