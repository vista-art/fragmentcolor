//! Light — directional, point, and spot variants following glTF
//! `KHR_lights_punctual`.
//!
//! A Light holds Arc-shared state, so the same handle can be absorbed by
//! multiple Passes with `pass.add(&light)`; later mutators (`set_direction`,
//! `set_position`, `set_color`, `set_intensity`, `set_range`,
//! `set_cone_angles`) propagate to every shader the Light has been wired
//! into.
//!
//! Construction picks the variant up-front:
//!
//! - [`Light::directional`] — parallel rays, only `direction` matters
//! - [`Light::point`] — radiates from `position`, inverse-square falloff
//! - [`Light::spot`] — point light constrained to a cone aligned with
//!   `-direction`, with smooth falloff between the inner and outer angle
//!
//! Mutators are shared across variants; fields that don't apply to a given
//! kind (e.g. `set_position` on a directional) are stored but unused by the
//! shader.

use glam::Vec3;
use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::sync::{Arc, Weak};

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::scene::SceneObject;
use crate::shader::ShaderObject;
use crate::Shader;

/// Numeric tag matched against the WGSL `light.kind` field. Values stay in
/// lockstep with the branch indices in `pbr_main.wgsl`'s fs_main:
/// directional, point, spot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum LightKind {
    Directional = 0,
    Point = 1,
    Spot = 2,
}

#[derive(Debug)]
pub(crate) struct LightObject {
    state: RwLock<LightState>,
    attached: RwLock<Vec<Weak<ShaderObject>>>,
}

#[derive(Debug, Clone, Copy)]
struct LightState {
    kind: LightKind,
    direction: Vec3,
    position: Vec3,
    color: Vec3,
    intensity: f32,
    /// 0.0 means unlimited (matches glTF KHR_lights_punctual default).
    range: f32,
    /// Stored as radians; converted to cosine when written to the shader.
    inner_cone_angle: f32,
    outer_cone_angle: f32,
}

impl LightState {
    fn defaults(kind: LightKind, direction: Vec3, position: Vec3, color: Vec3) -> Self {
        Self {
            kind,
            direction,
            position,
            color,
            intensity: 1.0,
            range: 0.0,
            inner_cone_angle: 0.0,
            outer_cone_angle: std::f32::consts::FRAC_PI_4,
        }
    }
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/scene/light/light.md")]
pub struct Light {
    pub(crate) object: Arc<LightObject>,
}

crate::impl_fc_kind!(Light, "Light");

impl Light {
    fn from_state(state: LightState) -> Self {
        Self {
            object: Arc::new(LightObject {
                state: RwLock::new(state),
                attached: RwLock::new(Vec::new()),
            }),
        }
    }

    #[lsp_doc("docs/api/scene/light/directional.md")]
    pub fn directional(direction: [f32; 3], color: [f32; 3]) -> Self {
        Self::from_state(LightState::defaults(
            LightKind::Directional,
            Vec3::from(direction),
            Vec3::ZERO,
            Vec3::from(color),
        ))
    }

    #[lsp_doc("docs/api/scene/light/point.md")]
    pub fn point(position: [f32; 3], color: [f32; 3]) -> Self {
        Self::from_state(LightState::defaults(
            LightKind::Point,
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::from(position),
            Vec3::from(color),
        ))
    }

    #[lsp_doc("docs/api/scene/light/spot.md")]
    pub fn spot(position: [f32; 3], direction: [f32; 3], color: [f32; 3]) -> Self {
        Self::from_state(LightState::defaults(
            LightKind::Spot,
            Vec3::from(direction),
            Vec3::from(position),
            Vec3::from(color),
        ))
    }

    #[lsp_doc("docs/api/scene/light/direction.md")]
    pub fn direction(&self) -> [f32; 3] {
        self.object.state.read().direction.to_array()
    }

    #[lsp_doc("docs/api/scene/light/position.md")]
    pub fn position(&self) -> [f32; 3] {
        self.object.state.read().position.to_array()
    }

    #[lsp_doc("docs/api/scene/light/color.md")]
    pub fn color(&self) -> [f32; 3] {
        self.object.state.read().color.to_array()
    }

    #[lsp_doc("docs/api/scene/light/intensity.md")]
    pub fn intensity(&self) -> f32 {
        self.object.state.read().intensity
    }

    #[lsp_doc("docs/api/scene/light/range.md")]
    pub fn range(&self) -> f32 {
        self.object.state.read().range
    }

    #[lsp_doc("docs/api/scene/light/inner_cone_angle.md")]
    pub fn inner_cone_angle(&self) -> f32 {
        self.object.state.read().inner_cone_angle
    }

    #[lsp_doc("docs/api/scene/light/outer_cone_angle.md")]
    pub fn outer_cone_angle(&self) -> f32 {
        self.object.state.read().outer_cone_angle
    }

    #[lsp_doc("docs/api/scene/light/set_direction.md")]
    pub fn set_direction(&self, direction: [f32; 3]) -> Self {
        self.object.state.write().direction = Vec3::from(direction);
        self.propagate();
        self.clone()
    }

    #[lsp_doc("docs/api/scene/light/set_position.md")]
    pub fn set_position(&self, position: [f32; 3]) -> Self {
        self.object.state.write().position = Vec3::from(position);
        self.propagate();
        self.clone()
    }

    #[lsp_doc("docs/api/scene/light/set_color.md")]
    pub fn set_color(&self, color: [f32; 3]) -> Self {
        self.object.state.write().color = Vec3::from(color);
        self.propagate();
        self.clone()
    }

    #[lsp_doc("docs/api/scene/light/set_intensity.md")]
    pub fn set_intensity(&self, value: f32) -> Self {
        self.object.state.write().intensity = value;
        self.propagate();
        self.clone()
    }

    #[lsp_doc("docs/api/scene/light/set_range.md")]
    pub fn set_range(&self, value: f32) -> Self {
        self.object.state.write().range = value.max(0.0);
        self.propagate();
        self.clone()
    }

    #[lsp_doc("docs/api/scene/light/set_cone_angles.md")]
    pub fn set_cone_angles(&self, inner_radians: f32, outer_radians: f32) -> Self {
        let mut s = self.object.state.write();
        s.inner_cone_angle = inner_radians;
        s.outer_cone_angle = outer_radians;
        drop(s);
        self.propagate();
        self.clone()
    }

    fn write_to_shader(&self, shader: &Shader) {
        let s = *self.object.state.read();
        let _ = shader.set("light.kind", s.kind as u32);
        let _ = shader.set("light.direction", s.direction.to_array());
        let _ = shader.set("light.position", s.position.to_array());
        let _ = shader.set("light.color", s.color.to_array());
        let _ = shader.set("light.intensity", s.intensity);
        let _ = shader.set("light.range", s.range);
        let _ = shader.set("light.inner_cone_cos", s.inner_cone_angle.cos());
        let _ = shader.set("light.outer_cone_cos", s.outer_cone_angle.cos());
    }

    fn propagate(&self) {
        let mut attached = self.object.attached.write();
        attached.retain(|weak| {
            if let Some(shader) = weak.upgrade() {
                self.write_to_shader(&Shader::from(shader));
                true
            } else {
                false
            }
        });
    }
}

impl SceneObject for Light {
    fn attach(&self, pass: &crate::Pass) -> Result<(), crate::PassError> {
        let shaders: Vec<Arc<ShaderObject>> =
            pass.object.shaders.read().iter().cloned().collect();
        for s in shaders {
            self.apply_to_shader(&Shader::from(s));
        }
        pass.object.scene_objects.write().push(Box::new(self.clone()));
        Ok(())
    }

    fn apply_to_shader(&self, shader: &Shader) {
        self.write_to_shader(shader);
        self.object
            .attached
            .write()
            .push(Arc::downgrade(&shader.object));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Material;

    #[test]
    fn directional_round_trips_direction_and_color() {
        let light = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);
        assert_eq!(light.direction(), [0.3, -1.0, -0.4]);
        assert_eq!(light.color(), [1.0, 0.95, 0.9]);
        assert_eq!(light.intensity(), 1.0);
        assert_eq!(light.range(), 0.0);
    }

    #[test]
    fn point_round_trips_position_and_color() {
        let light = Light::point([5.0, 2.0, -1.0], [0.8, 0.9, 1.0]);
        assert_eq!(light.position(), [5.0, 2.0, -1.0]);
        assert_eq!(light.color(), [0.8, 0.9, 1.0]);
    }

    #[test]
    fn spot_carries_both_position_and_direction() {
        let light = Light::spot([0.0, 5.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
        assert_eq!(light.position(), [0.0, 5.0, 0.0]);
        assert_eq!(light.direction(), [0.0, -1.0, 0.0]);
        // Default cone defaults to (0, π/4) — full center, 45° outer.
        assert_eq!(light.inner_cone_angle(), 0.0);
        assert!((light.outer_cone_angle() - std::f32::consts::FRAC_PI_4).abs() < 1.0e-6);
    }

    #[test]
    fn set_range_clamps_to_non_negative() {
        let light = Light::point([0.0; 3], [1.0; 3]);
        light.set_range(-5.0);
        assert_eq!(light.range(), 0.0);
    }

    fn pbr_triangle_mesh() -> crate::Mesh {
        let mesh = crate::Mesh::new();
        mesh.add_vertex(
            crate::mesh::Vertex::new([0.0, 0.5, 0.0])
                .set(crate::mesh::Vertex::NORMAL, [0.0, 0.0, 1.0])
                .set(crate::mesh::Vertex::UV0, [0.5, 1.0]),
        );
        mesh
    }

    #[test]
    fn pass_add_seeds_shader_uniforms() {
        let light = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);
        let material = Material::pbr().expect("pbr");
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());

        let pass = crate::Pass::new("scene");
        pass.add(&model).expect("add_model");
        pass.add(&light).expect("add light");

        let dir: [f32; 3] = material
            .shader()
            .get("light.direction")
            .expect("light.direction");
        assert_eq!(dir, [0.3, -1.0, -0.4]);

        let col: [f32; 3] = material.shader().get("light.color").expect("light.color");
        assert_eq!(col, [1.0, 0.95, 0.9]);

        let kind: u32 = material.shader().get("light.kind").expect("light.kind");
        assert_eq!(kind, 0);
    }

    #[test]
    fn point_attach_writes_kind_and_position() {
        let light = Light::point([2.0, 1.0, 0.5], [1.0, 1.0, 1.0]).set_intensity(2.5);
        let material = Material::pbr().expect("pbr");
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        let pass = crate::Pass::new("scene");
        pass.add(&model).expect("add_model");
        pass.add(&light).expect("add light");

        let kind: u32 = material.shader().get("light.kind").unwrap();
        assert_eq!(kind, 1);
        let pos: [f32; 3] = material.shader().get("light.position").unwrap();
        assert_eq!(pos, [2.0, 1.0, 0.5]);
        let intensity: f32 = material.shader().get("light.intensity").unwrap();
        assert!((intensity - 2.5).abs() < 1.0e-6);
    }

    #[test]
    fn spot_attach_writes_kind_and_cone_cosines() {
        let light = Light::spot([0.0, 5.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
            .set_cone_angles(0.2, 0.6);
        let material = Material::pbr().expect("pbr");
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        let pass = crate::Pass::new("scene");
        pass.add(&model).expect("add_model");
        pass.add(&light).expect("add light");

        let kind: u32 = material.shader().get("light.kind").unwrap();
        assert_eq!(kind, 2);
        let inner: f32 = material.shader().get("light.inner_cone_cos").unwrap();
        let outer: f32 = material.shader().get("light.outer_cone_cos").unwrap();
        assert!((inner - 0.2_f32.cos()).abs() < 1.0e-6);
        assert!((outer - 0.6_f32.cos()).abs() < 1.0e-6);
    }

    #[test]
    fn set_direction_propagates_to_all_pass_shaders() {
        let light = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
        let material = Material::pbr().expect("pbr");
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());

        let pass = crate::Pass::new("scene");
        pass.add(&model).expect("add_model");
        pass.add(&light).expect("add light");

        light.set_direction([0.5, -0.5, 0.0]);
        let dir: [f32; 3] = material.shader().get("light.direction").unwrap();
        assert_eq!(dir, [0.5, -0.5, 0.0]);
    }

    #[test]
    fn set_position_propagates_to_all_pass_shaders() {
        let light = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let material = Material::pbr().expect("pbr");
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());

        let pass = crate::Pass::new("scene");
        pass.add(&model).expect("add_model");
        pass.add(&light).expect("add light");

        light.set_position([3.0, 1.5, -2.0]);
        let pos: [f32; 3] = material.shader().get("light.position").unwrap();
        assert_eq!(pos, [3.0, 1.5, -2.0]);
    }
}
