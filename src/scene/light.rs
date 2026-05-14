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

/// Hard cap on simultaneously-active lights per Shader. Must match the
/// `PBR_MAX_LIGHTS` const + `array<Light, N>` literal in
/// `src/material/pbr_main.wgsl`; raise both together if you need more.
pub(crate) const PBR_MAX_LIGHTS: u32 = 8;

#[derive(Debug)]
pub(crate) struct LightObject {
    state: RwLock<LightState>,
    /// `(shader, slot)` pairs — each entry remembers which slot in the
    /// shader's `lights.lights[..]` array this Light owns, so live
    /// `set_*` mutations can write back to the right index.
    attached: RwLock<Vec<(Weak<ShaderObject>, u32)>>,
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

    fn write_to_shader_slot(&self, shader: &Shader, slot: u32) {
        let s = *self.object.state.read();
        let prefix = format!("lights.lights[{slot}]");
        let _ = shader.set(&format!("{prefix}.kind"), s.kind as u32);
        let _ = shader.set(&format!("{prefix}.direction"), s.direction.to_array());
        let _ = shader.set(&format!("{prefix}.position"), s.position.to_array());
        let _ = shader.set(&format!("{prefix}.color"), s.color.to_array());
        let _ = shader.set(&format!("{prefix}.intensity"), s.intensity);
        let _ = shader.set(&format!("{prefix}.range"), s.range);
        let _ = shader.set(
            &format!("{prefix}.inner_cone_cos"),
            s.inner_cone_angle.cos(),
        );
        let _ = shader.set(
            &format!("{prefix}.outer_cone_cos"),
            s.outer_cone_angle.cos(),
        );
    }

    fn propagate(&self) {
        let mut attached = self.object.attached.write();
        attached.retain(|(weak, slot)| {
            if let Some(shader) = weak.upgrade() {
                self.write_to_shader_slot(&Shader::from(shader), *slot);
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
        // Dedup-by-shader: a Light already absorbed by this Shader keeps
        // its existing slot, just rewrites its values. Without this, the
        // Pass's replay-on-shader-join path would consume a new slot for
        // every Model that joins after the Light, eventually capping out.
        let shader_ptr = Arc::as_ptr(&shader.object);
        let existing = self
            .object
            .attached
            .read()
            .iter()
            .find_map(|(weak, slot)| {
                weak.upgrade().and_then(|sh| {
                    if Arc::as_ptr(&sh) == shader_ptr {
                        Some(*slot)
                    } else {
                        None
                    }
                })
            });
        if let Some(slot) = existing {
            self.write_to_shader_slot(shader, slot);
            return;
        }

        // Allocate the next slot. Material::pbr seeds slot 0 with a dim
        // placeholder + `lights.count = 1`; the first user-attached Light
        // overwrites that slot rather than living at slot 1, so adding
        // a single Light produces one lit scene (not "Light + placeholder").
        let slot = {
            let mut n = shader.object.user_lights_attached.write();
            let i = *n;
            if i >= PBR_MAX_LIGHTS {
                log::warn!(
                    "Shader has reached the {}-light cap; ignoring additional Light attach",
                    PBR_MAX_LIGHTS
                );
                return;
            }
            *n = i + 1;
            i
        };
        self.write_to_shader_slot(shader, slot);
        // Publish the new active count to the WGSL `lights.count` field.
        let _ = shader.set("lights.count", slot + 1);
        self.object
            .attached
            .write()
            .push((Arc::downgrade(&shader.object), slot));
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
            crate::mesh::Vertex::pbr([0.0, 0.5, 0.0]).set(crate::mesh::Vertex::UV0, [0.5, 1.0]),
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
            .get("lights.lights[0].direction")
            .expect("lights.lights[0].direction");
        assert_eq!(dir, [0.3, -1.0, -0.4]);

        let col: [f32; 3] = material.shader().get("lights.lights[0].color").expect("lights.lights[0].color");
        assert_eq!(col, [1.0, 0.95, 0.9]);

        let kind: u32 = material.shader().get("lights.lights[0].kind").expect("lights.lights[0].kind");
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

        let kind: u32 = material.shader().get("lights.lights[0].kind").unwrap();
        assert_eq!(kind, 1);
        let pos: [f32; 3] = material.shader().get("lights.lights[0].position").unwrap();
        assert_eq!(pos, [2.0, 1.0, 0.5]);
        let intensity: f32 = material.shader().get("lights.lights[0].intensity").unwrap();
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

        let kind: u32 = material.shader().get("lights.lights[0].kind").unwrap();
        assert_eq!(kind, 2);
        let inner: f32 = material.shader().get("lights.lights[0].inner_cone_cos").unwrap();
        let outer: f32 = material.shader().get("lights.lights[0].outer_cone_cos").unwrap();
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
        let dir: [f32; 3] = material.shader().get("lights.lights[0].direction").unwrap();
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
        let pos: [f32; 3] = material.shader().get("lights.lights[0].position").unwrap();
        assert_eq!(pos, [3.0, 1.5, -2.0]);
    }

    #[test]
    fn two_lights_take_distinct_slots() {
        // First light claims slot 0 (overwrites the Material's default
        // single-light seed); second takes slot 1. `lights.count` reaches 2.
        let sun = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
        let lamp = Light::point([2.0, 1.0, 0.0], [1.0, 0.5, 0.5]).set_intensity(3.0);
        let material = Material::pbr().expect("pbr");
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        let pass = crate::Pass::new("scene");
        pass.add(&model).expect("add_model");
        pass.add(&sun).expect("sun");
        pass.add(&lamp).expect("lamp");

        let count: u32 = material.shader().get("lights.count").unwrap();
        assert_eq!(count, 2);

        let sun_dir: [f32; 3] = material.shader().get("lights.lights[0].direction").unwrap();
        assert_eq!(sun_dir, [0.0, -1.0, 0.0]);
        let lamp_kind: u32 = material.shader().get("lights.lights[1].kind").unwrap();
        assert_eq!(lamp_kind, 1, "second slot should hold the point light");
        let lamp_intensity: f32 = material
            .shader()
            .get("lights.lights[1].intensity")
            .unwrap();
        assert!((lamp_intensity - 3.0).abs() < 1.0e-6);
    }

    #[test]
    fn light_attached_twice_keeps_one_slot() {
        // The Pass replay mechanism re-runs `apply_to_shader` for every
        // scene_object whenever a new Shader joins. Without the dedup-by-
        // shader check in `apply_to_shader`, each replay would claim a
        // fresh slot and saturate the cap after a few Model adds. The
        // dedup keeps the Light at its original slot.
        let light = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
        let material = Material::pbr().expect("pbr");
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        let pass = crate::Pass::new("scene");
        pass.add(&light).expect("light");
        pass.add(&model).expect("model");
        // Adding another model that shares the same material → same shader →
        // triggers replay of the existing Light. count must stay 1.
        let other = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        pass.add(&other).expect("other");

        let count: u32 = material.shader().get("lights.count").unwrap();
        assert_eq!(count, 1, "dedup must keep replay from claiming new slots");
    }

    #[test]
    fn ambient_default_seeds_to_dim_grey() {
        // Material::apply_defaults seeds `lights.ambient = [0.03; 3]` so
        // a fresh material's shader has a sensible non-zero ambient
        // (matches the hardcoded `* 0.03` value the prior shader had).
        let material = Material::pbr().expect("pbr");
        let amb: [f32; 3] = material.shader().get("lights.ambient").unwrap();
        assert_eq!(amb, [0.03, 0.03, 0.03]);
    }
}
