#![cfg(mobile)]

use lsp_doc::lsp_doc;
use std::sync::Arc;

use crate::renderer::platform::mobile::FragmentColorError;
use crate::scene::{Camera, Light, Model, Scene};
use crate::{Material, Mesh, Pass};

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
    pub fn translate_mobile(
        self: Arc<Self>,
        offset: Vec<f32>,
    ) -> Result<(), FragmentColorError> {
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
    pub fn scale_mobile(
        self: Arc<Self>,
        factor: Vec<f32>,
    ) -> Result<(), FragmentColorError> {
        let arr = take_vec3(&factor, "Model.scale")?;
        self.scale(arr);
        Ok(())
    }
}

#[uniffi::export]
impl Camera {
    #[uniffi::constructor(name = "perspective")]
    #[lsp_doc("docs/api/scene/camera/perspective.md")]
    pub fn perspective_mobile(
        fovy_radians: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Arc<Self> {
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
        eye: Vec<f32>,
        target: Vec<f32>,
        up: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let eye = take_vec3(&eye, "Camera.lookAt eye")?;
        let target = take_vec3(&target, "Camera.lookAt target")?;
        let up = take_vec3(&up, "Camera.lookAt up")?;
        Ok(Arc::new((*self).clone().look_at(eye, target, up)))
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

    #[uniffi::method(name = "direction")]
    #[lsp_doc("docs/api/scene/light/direction.md")]
    pub fn direction_mobile(self: Arc<Self>) -> Vec<f32> {
        self.direction().to_vec()
    }

    #[uniffi::method(name = "position")]
    #[lsp_doc("docs/api/scene/light/position.md")]
    pub fn position_mobile(self: Arc<Self>) -> Vec<f32> {
        self.position().to_vec()
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

    #[uniffi::method(name = "range")]
    #[lsp_doc("docs/api/scene/light/range.md")]
    pub fn range_mobile(self: Arc<Self>) -> f32 {
        self.range()
    }

    #[uniffi::method(name = "innerConeAngle")]
    #[lsp_doc("docs/api/scene/light/inner_cone_angle.md")]
    pub fn inner_cone_angle_mobile(self: Arc<Self>) -> f32 {
        self.inner_cone_angle()
    }

    #[uniffi::method(name = "outerConeAngle")]
    #[lsp_doc("docs/api/scene/light/outer_cone_angle.md")]
    pub fn outer_cone_angle_mobile(self: Arc<Self>) -> f32 {
        self.outer_cone_angle()
    }

    #[uniffi::method(name = "setDirection")]
    #[lsp_doc("docs/api/scene/light/set_direction.md")]
    pub fn set_direction_mobile(
        self: Arc<Self>,
        direction: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let direction = take_vec3(&direction, "Light.setDirection")?;
        Ok(Arc::new(self.set_direction(direction)))
    }

    #[uniffi::method(name = "setPosition")]
    #[lsp_doc("docs/api/scene/light/set_position.md")]
    pub fn set_position_mobile(
        self: Arc<Self>,
        position: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let position = take_vec3(&position, "Light.setPosition")?;
        Ok(Arc::new(self.set_position(position)))
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

    #[uniffi::method(name = "setRange")]
    #[lsp_doc("docs/api/scene/light/set_range.md")]
    pub fn set_range_mobile(self: Arc<Self>, value: f32) -> Arc<Self> {
        Arc::new(self.set_range(value))
    }

    #[uniffi::method(name = "setConeAngles")]
    #[lsp_doc("docs/api/scene/light/set_cone_angles.md")]
    pub fn set_cone_angles_mobile(
        self: Arc<Self>,
        inner_radians: f32,
        outer_radians: f32,
    ) -> Arc<Self> {
        Arc::new(self.set_cone_angles(inner_radians, outer_radians))
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

    #[uniffi::method(name = "addModel")]
    #[lsp_doc("docs/api/scene/scene/add.md")]
    pub fn add_model_mobile(
        self: Arc<Self>,
        model: Arc<Model>,
    ) -> Result<(), FragmentColorError> {
        self.add(model.as_ref())
            .map(|_| ())
            .map_err(|e| FragmentColorError::Render(e.to_string()))
    }

    #[uniffi::method(name = "addCamera")]
    #[lsp_doc("docs/api/scene/scene/add.md")]
    pub fn add_camera_mobile(
        self: Arc<Self>,
        camera: Arc<Camera>,
    ) -> Result<(), FragmentColorError> {
        self.add(camera.as_ref())
            .map(|_| ())
            .map_err(|e| FragmentColorError::Render(e.to_string()))
    }

    #[uniffi::method(name = "addLight")]
    #[lsp_doc("docs/api/scene/scene/add.md")]
    pub fn add_light_mobile(
        self: Arc<Self>,
        light: Arc<Light>,
    ) -> Result<(), FragmentColorError> {
        self.add(light.as_ref())
            .map(|_| ())
            .map_err(|e| FragmentColorError::Render(e.to_string()))
    }

    #[uniffi::method(name = "addPass")]
    #[lsp_doc("docs/api/scene/scene/add_pass.md")]
    pub fn add_pass_mobile(self: Arc<Self>, pass: Arc<Pass>) {
        self.add_pass(pass.as_ref());
    }

    #[uniffi::method(name = "ambient")]
    #[lsp_doc("docs/api/scene/scene/ambient.md")]
    pub fn ambient_mobile(
        self: Arc<Self>,
        color: Vec<f32>,
    ) -> Result<(), FragmentColorError> {
        let c = take_vec3(&color, "Scene.ambient")?;
        self.ambient(c);
        Ok(())
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
