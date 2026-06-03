//! Light — single type covering the three glTF `KHR_lights_punctual` kinds
//! (directional, point, spot). The kind is set once at construction via the
//! [`Light::directional`] / [`Light::point`] / [`Light::spot`] constructors;
//! kind-specific setters (`set_position`, `set_direction`, `set_range`,
//! `set_cone_angles`) return a typed [`LightError::FieldNotApplicable`] when
//! called on the wrong kind, and the matching getters return `None`.
//!
//! Lights hold Arc-shared state, so a single handle can be absorbed by
//! multiple Passes with `pass.add(&light)`; later mutators propagate to every
//! shader the Light has been wired into. The renderer's shader-binding code
//! is uniform — every Light writes into the same WGSL `lights.lights[..]`
//! array, branching on the `kind` discriminant.

use glam::Vec3;
use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::sync::{Arc, Weak};

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::Shader;
use crate::scene::SceneObject;
use crate::shader::ShaderObject;

/// Discriminant for the three light kinds. Wire-compatible with the WGSL
/// `light.kind` field in `pbr_main.wgsl`: directional=0, point=1, spot=2.
#[cfg_attr(python, pyo3::pyclass(eq, eq_int))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum LightKind {
    Directional = 0,
    Point = 1,
    Spot = 2,
}

/// Errors returned by kind-specific [`Light`] setters when the field does
/// not apply to the constructed kind.
#[derive(Debug, thiserror::Error)]
pub enum LightError {
    #[error("Light::{field} does not apply to a {kind:?} light")]
    FieldNotApplicable {
        kind: LightKind,
        field: &'static str,
    },
    #[error("Light::set_range expects a non-negative value, got {0}")]
    NegativeRange(f32),
}

/// Hard cap on simultaneously-active lights per Shader. Must match the
/// `PBR_MAX_LIGHTS` const + `array<Light, N>` literal in
/// `src/material/pbr_main.wgsl`; raise both together if you need more.
pub(crate) const PBR_MAX_LIGHTS: u32 = 32;

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

impl LightObject {
    fn new(state: LightState) -> Arc<Self> {
        Arc::new(Self {
            state: RwLock::new(state),
            attached: RwLock::new(Vec::new()),
        })
    }

    fn write_to_shader_slot(&self, shader: &Shader, slot: u32) {
        let s = *self.state.read();
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
        let mut attached = self.attached.write();
        attached.retain(|(weak, slot)| {
            if let Some(shader) = weak.upgrade() {
                self.write_to_shader_slot(&Shader::from(shader), *slot);
                true
            } else {
                false
            }
        });
    }

    fn attach_to_pass(self: &Arc<Self>, pass: &crate::Pass) -> Result<(), crate::PassError> {
        let shaders: Vec<Arc<ShaderObject>> = pass.object.shaders.read().iter().cloned().collect();
        // Pre-check the cap on every shader. A Light already attached to a
        // shader rides on its existing slot (no new allocation), so the
        // pre-check mirrors the dedup logic in `apply_to_shader` rather
        // than refusing reattachment of an existing Light.
        for s in &shaders {
            let already_attached = self.attached.read().iter().any(|(weak, _)| {
                weak.upgrade()
                    .map(|sh| Arc::ptr_eq(&sh, s))
                    .unwrap_or(false)
            });
            if !already_attached && *s.user_lights_attached.read() >= PBR_MAX_LIGHTS {
                return Err(crate::PassError::LightCapReached {
                    cap: PBR_MAX_LIGHTS,
                });
            }
        }
        for s in shaders {
            self.apply_to_shader(&Shader::from(s));
        }
        Ok(())
    }

    fn apply_to_shader(&self, shader: &Shader) {
        // Dedup-by-shader: a Light already absorbed by this Shader keeps
        // its existing slot, just rewrites its values. Without this, the
        // Pass's replay-on-shader-join path would consume a new slot for
        // every Model that joins after the Light, eventually capping out.
        let shader_ptr = Arc::as_ptr(&shader.object);
        let existing = self.attached.read().iter().find_map(|(weak, slot)| {
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
        self.attached
            .write()
            .push((Arc::downgrade(&shader.object), slot));
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
    /// Build a directional light — parallel rays in `direction`, no position.
    /// Fit for sun / sky / fill — anything where every shaded surface
    /// receives light from the same world-space direction.
    #[lsp_doc("docs/api/scene/light/directional.md")]
    pub fn directional(direction: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            object: LightObject::new(LightState::defaults(
                LightKind::Directional,
                Vec3::from(direction),
                Vec3::ZERO,
                Vec3::from(color),
            )),
        }
    }

    /// Build a point light — radiates from `position` with inverse-square
    /// distance falloff. `set_range` caps the influence radius (default 0 =
    /// unlimited, matching glTF `KHR_lights_punctual`).
    #[lsp_doc("docs/api/scene/light/point.md")]
    pub fn point(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            object: LightObject::new(LightState::defaults(
                LightKind::Point,
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::from(position),
                Vec3::from(color),
            )),
        }
    }

    /// Build a spot light — point light constrained to a cone. `position`
    /// is the apex, `direction` is the cone axis, cone half-angles default
    /// to `(0, π/4)` (full centre, 45° outer); tune via `set_cone_angles`.
    #[lsp_doc("docs/api/scene/light/spot.md")]
    pub fn spot(position: [f32; 3], direction: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            object: LightObject::new(LightState::defaults(
                LightKind::Spot,
                Vec3::from(direction),
                Vec3::from(position),
                Vec3::from(color),
            )),
        }
    }

    /// Read which kind this Light was constructed as.
    #[lsp_doc("docs/api/scene/light/kind.md")]
    pub fn kind(&self) -> LightKind {
        self.object.state.read().kind
    }

    // ------------------------------------------------------------------
    // Universal getters — defined for every kind.
    // ------------------------------------------------------------------

    /// Read the linear-RGB color (defined for every kind).
    #[lsp_doc("docs/api/scene/light/color.md")]
    pub fn color(&self) -> [f32; 3] {
        self.object.state.read().color.to_array()
    }

    /// Read the scalar intensity multiplier (defined for every kind).
    #[lsp_doc("docs/api/scene/light/intensity.md")]
    pub fn intensity(&self) -> f32 {
        self.object.state.read().intensity
    }

    // ------------------------------------------------------------------
    // Universal setters — defined for every kind.
    // ------------------------------------------------------------------

    /// Update the linear-RGB color and propagate to every shader the Light
    /// has been wired into. Defined for every kind.
    #[lsp_doc("docs/api/scene/light/set_color.md")]
    pub fn set_color(&self, color: [f32; 3]) -> Self {
        self.object.state.write().color = Vec3::from(color);
        self.object.propagate();
        self.clone()
    }

    /// Update the scalar intensity multiplier and propagate to every shader
    /// the Light has been wired into. Defined for every kind.
    #[lsp_doc("docs/api/scene/light/set_intensity.md")]
    pub fn set_intensity(&self, value: f32) -> Self {
        self.object.state.write().intensity = value;
        self.object.propagate();
        self.clone()
    }

    // ------------------------------------------------------------------
    // Kind-specific getters — return `None` on the kind that doesn't apply.
    // ------------------------------------------------------------------

    /// Read the world-space position. `None` for a directional light.
    #[lsp_doc("docs/api/scene/light/position.md")]
    pub fn position(&self) -> Option<[f32; 3]> {
        let s = self.object.state.read();
        match s.kind {
            LightKind::Point | LightKind::Spot => Some(s.position.to_array()),
            LightKind::Directional => None,
        }
    }

    /// Read the world-space direction. `None` for a point light.
    #[lsp_doc("docs/api/scene/light/direction.md")]
    pub fn direction(&self) -> Option<[f32; 3]> {
        let s = self.object.state.read();
        match s.kind {
            LightKind::Directional | LightKind::Spot => Some(s.direction.to_array()),
            LightKind::Point => None,
        }
    }

    /// Read the influence-radius cap. `None` for a directional light.
    #[lsp_doc("docs/api/scene/light/range.md")]
    pub fn range(&self) -> Option<f32> {
        let s = self.object.state.read();
        match s.kind {
            LightKind::Point | LightKind::Spot => Some(s.range),
            LightKind::Directional => None,
        }
    }

    /// Read the inner cone half-angle in radians. `Some` only on spot lights.
    #[lsp_doc("docs/api/scene/light/inner_cone_angle.md")]
    pub fn inner_cone_angle(&self) -> Option<f32> {
        let s = self.object.state.read();
        match s.kind {
            LightKind::Spot => Some(s.inner_cone_angle),
            _ => None,
        }
    }

    /// Read the outer cone half-angle in radians. `Some` only on spot lights.
    #[lsp_doc("docs/api/scene/light/outer_cone_angle.md")]
    pub fn outer_cone_angle(&self) -> Option<f32> {
        let s = self.object.state.read();
        match s.kind {
            LightKind::Spot => Some(s.outer_cone_angle),
            _ => None,
        }
    }

    // ------------------------------------------------------------------
    // Kind-specific setters — return Err on the kind that doesn't apply.
    // ------------------------------------------------------------------

    /// Update the world-space position. Errors on a directional light.
    #[lsp_doc("docs/api/scene/light/set_position.md")]
    pub fn set_position(&self, position: [f32; 3]) -> Result<Self, LightError> {
        let kind = self.object.state.read().kind;
        match kind {
            LightKind::Point | LightKind::Spot => {
                self.object.state.write().position = Vec3::from(position);
                self.object.propagate();
                Ok(self.clone())
            }
            LightKind::Directional => Err(LightError::FieldNotApplicable {
                kind,
                field: "set_position",
            }),
        }
    }

    /// Update the world-space direction. Errors on a point light.
    #[lsp_doc("docs/api/scene/light/set_direction.md")]
    pub fn set_direction(&self, direction: [f32; 3]) -> Result<Self, LightError> {
        let kind = self.object.state.read().kind;
        match kind {
            LightKind::Directional | LightKind::Spot => {
                self.object.state.write().direction = Vec3::from(direction);
                self.object.propagate();
                Ok(self.clone())
            }
            LightKind::Point => Err(LightError::FieldNotApplicable {
                kind,
                field: "set_direction",
            }),
        }
    }

    /// Update the influence-radius cap. Errors on a directional light and
    /// on any negative value regardless of kind.
    #[lsp_doc("docs/api/scene/light/set_range.md")]
    pub fn set_range(&self, range: f32) -> Result<Self, LightError> {
        if range < 0.0 {
            return Err(LightError::NegativeRange(range));
        }
        let kind = self.object.state.read().kind;
        match kind {
            LightKind::Point | LightKind::Spot => {
                self.object.state.write().range = range;
                self.object.propagate();
                Ok(self.clone())
            }
            LightKind::Directional => Err(LightError::FieldNotApplicable {
                kind,
                field: "set_range",
            }),
        }
    }

    /// Update the cone half-angles (radians). Errors on a non-spot light.
    #[lsp_doc("docs/api/scene/light/set_cone_angles.md")]
    pub fn set_cone_angles(&self, inner: f32, outer: f32) -> Result<Self, LightError> {
        let kind = self.object.state.read().kind;
        match kind {
            LightKind::Spot => {
                let mut s = self.object.state.write();
                s.inner_cone_angle = inner;
                s.outer_cone_angle = outer;
                drop(s);
                self.object.propagate();
                Ok(self.clone())
            }
            _ => Err(LightError::FieldNotApplicable {
                kind,
                field: "set_cone_angles",
            }),
        }
    }
}

impl SceneObject for Light {
    fn attach(&self, pass: &crate::Pass) -> Result<(), crate::PassError> {
        self.object.attach_to_pass(pass)?;
        pass.object
            .scene_objects
            .write()
            .push(Box::new(self.clone()));
        Ok(())
    }

    fn apply_to_shader(&self, shader: &Shader) {
        self.object.apply_to_shader(shader);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Material;

    fn pbr_triangle_mesh() -> crate::Mesh {
        let mesh = crate::Mesh::new();
        mesh.add_vertex(
            crate::mesh::Vertex::pbr([0.0, 0.5, 0.0]).set(crate::mesh::Vertex::UV0, [0.5, 1.0]),
        );
        mesh
    }

    #[test]
    fn constructors_report_their_kind() {
        let d = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
        let p = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let s = Light::spot([0.0, 5.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
        assert_eq!(d.kind(), LightKind::Directional);
        assert_eq!(p.kind(), LightKind::Point);
        assert_eq!(s.kind(), LightKind::Spot);
    }

    #[test]
    fn universal_setters_round_trip_for_every_kind() {
        for light in [
            Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]),
            Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
            Light::spot([0.0, 5.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]),
        ] {
            light.set_color([0.2, 0.4, 0.6]).set_intensity(2.5);
            assert_eq!(light.color(), [0.2, 0.4, 0.6]);
            assert!((light.intensity() - 2.5).abs() < 1.0e-6);
        }
    }

    #[test]
    fn kind_specific_setters_err_on_wrong_kind() {
        let directional = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(matches!(
            directional.set_position([1.0, 0.0, 0.0]),
            Err(LightError::FieldNotApplicable {
                kind: LightKind::Directional,
                field: "set_position"
            }),
        ));
        assert!(matches!(
            directional.set_range(5.0),
            Err(LightError::FieldNotApplicable {
                kind: LightKind::Directional,
                field: "set_range"
            }),
        ));
        assert!(matches!(
            directional.set_cone_angles(0.2, 0.6),
            Err(LightError::FieldNotApplicable {
                kind: LightKind::Directional,
                field: "set_cone_angles"
            }),
        ));

        let point = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(matches!(
            point.set_direction([0.0, -1.0, 0.0]),
            Err(LightError::FieldNotApplicable {
                kind: LightKind::Point,
                field: "set_direction"
            }),
        ));
        assert!(matches!(
            point.set_cone_angles(0.2, 0.6),
            Err(LightError::FieldNotApplicable {
                kind: LightKind::Point,
                field: "set_cone_angles"
            }),
        ));

        // set_range on negative values errors regardless of kind.
        assert!(matches!(
            point.set_range(-1.0),
            Err(LightError::NegativeRange(_))
        ));
    }

    #[test]
    fn kind_specific_setters_ok_and_round_trip_on_right_kind() {
        let point = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        point
            .set_position([3.0, 1.5, -2.0])
            .expect("position on point");
        point.set_range(8.0).expect("range on point");
        assert_eq!(point.position(), Some([3.0, 1.5, -2.0]));
        assert_eq!(point.range(), Some(8.0));
        assert!(point.direction().is_none(), "point has no direction");
        assert!(point.inner_cone_angle().is_none());
        assert!(point.outer_cone_angle().is_none());

        let spot = Light::spot([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
        spot.set_position([0.0, 2.0, 0.0])
            .expect("position on spot");
        spot.set_direction([0.0, 0.0, -1.0])
            .expect("direction on spot");
        spot.set_cone_angles(0.2, 0.6).expect("cone angles on spot");
        assert_eq!(spot.position(), Some([0.0, 2.0, 0.0]));
        assert_eq!(spot.direction(), Some([0.0, 0.0, -1.0]));
        assert_eq!(
            spot.inner_cone_angle().map(|a| (a - 0.2).abs() < 1.0e-6),
            Some(true)
        );
        assert_eq!(
            spot.outer_cone_angle().map(|a| (a - 0.6).abs() < 1.0e-6),
            Some(true)
        );
    }

    #[test]
    fn shader_uniforms_propagate_across_kinds() {
        // Three kinds in one pass — exercises the polymorphic add path
        // (Pass::add<O: SceneObject>) and confirms each kind writes the
        // correct discriminator into `lights.lights[i].kind`.
        let sun = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
        let lamp = Light::point([2.0, 1.0, 0.0], [1.0, 0.5, 0.5]).set_intensity(3.0);
        let torch = Light::spot([0.0, 3.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 0.8])
            .set_cone_angles(0.2, 0.6)
            .expect("spot cones");
        let material = Material::pbr().expect("pbr");
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        let pass = crate::Pass::new("scene");
        pass.add(&model)
            .expect("model")
            .add(&sun)
            .expect("sun")
            .add(&lamp)
            .expect("lamp")
            .add(&torch)
            .expect("torch");

        let count: u32 = material.shader().get("lights.count").unwrap();
        assert_eq!(count, 3);
        let k0: u32 = material.shader().get("lights.lights[0].kind").unwrap();
        let k1: u32 = material.shader().get("lights.lights[1].kind").unwrap();
        let k2: u32 = material.shader().get("lights.lights[2].kind").unwrap();
        assert_eq!(k0, 0);
        assert_eq!(k1, 1);
        assert_eq!(k2, 2);

        // Universal set_color propagates live.
        sun.set_color([0.5, 0.25, 0.125]);
        let c: [f32; 3] = material.shader().get("lights.lights[0].color").unwrap();
        assert_eq!(c, [0.5, 0.25, 0.125]);
    }

    #[test]
    fn replay_keeps_one_slot_per_light() {
        // The Pass replay mechanism re-runs `apply_to_shader` for every
        // scene_object whenever a new Shader joins. Without the dedup-by-
        // shader check in `apply_to_shader`, each replay would claim a
        // fresh slot and saturate the cap after a few Model adds.
        let light = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
        let material = Material::pbr().expect("pbr");
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        let pass = crate::Pass::new("scene");
        pass.add(&light).expect("light");
        pass.add(&model).expect("model");
        let other = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        pass.add(&other).expect("other");

        let count: u32 = material.shader().get("lights.count").unwrap();
        assert_eq!(count, 1, "dedup must keep replay from claiming new slots");
    }

    #[test]
    fn cap_rejects_lights_past_the_limit() {
        // Add `PBR_MAX_LIGHTS + 1` distinct lights; the last one should fail
        // with the typed cap error and `cap` should report the configured
        // ceiling.
        let material = Material::pbr().expect("pbr");
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        let pass = crate::Pass::new("scene");
        pass.add(&model).expect("model");
        for _ in 0..PBR_MAX_LIGHTS {
            pass.add(&Light::directional([0.0, -1.0, 0.0], [1.0; 3]))
                .expect("light under cap");
        }
        let extra = Light::directional([0.0, -1.0, 0.0], [1.0; 3]);
        let err = pass.add(&extra).expect_err("33rd light must fail");
        match err {
            crate::PassError::LightCapReached { cap } => assert_eq!(cap, PBR_MAX_LIGHTS),
            other => panic!("expected LightCapReached, got {other:?}"),
        }
    }

    #[test]
    fn cap_is_per_shader_not_per_pass() {
        // Two Passes that share a Material share the underlying ShaderObject
        // and therefore its 32-light slots. A Light added to Pass A appears
        // in Pass B's render because both reach into the same shader. The
        // doc on `Light` warns about this; this test pins the contract so
        // any future "per-Pass cap" refactor flags it explicitly.
        let material = Material::pbr().expect("pbr");
        let model_a = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        let model_b = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());

        let pass_a = crate::Pass::new("scene_a");
        let pass_b = crate::Pass::new("scene_b");
        pass_a.add(&model_a).expect("model_a attaches");
        pass_b.add(&model_b).expect("model_b attaches");

        let sun = Light::directional([0.0, -1.0, 0.0], [1.0; 3]);
        let lamp = Light::point([0.0, 1.0, 0.0], [1.0; 3]);
        pass_a.add(&sun).expect("sun on pass_a");
        pass_b.add(&lamp).expect("lamp on pass_b");

        // Both lights occupy slots on the shared shader; lights.count is 2.
        let count: u32 = material.shader().get("lights.count").unwrap();
        assert_eq!(
            count, 2,
            "lights from both Passes pack into the shared shader's slots"
        );
    }

    #[test]
    fn ambient_default_seeds_to_dim_grey() {
        // Material::apply_defaults seeds `lights.ambient = [0.03; 3]` so
        // a fresh material's shader has a sensible non-zero ambient.
        let material = Material::pbr().expect("pbr");
        let amb: [f32; 3] = material.shader().get("lights.ambient").unwrap();
        assert_eq!(amb, [0.03, 0.03, 0.03]);
    }
}
