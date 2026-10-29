//! Light — single type covering the three glTF `KHR_lights_punctual` kinds
//! (directional, point, spot). The kind is set once at construction via the
//! [`Light::directional`] / [`Light::point`] / [`Light::spot`] constructors;
//! kind-specific setters (`set_position`, `set_direction`, `set_range`,
//! `set_cone_angles`) return a typed [`LightError::FieldNotApplicable`] when
//! called on the wrong kind, and the matching getters return `None`.
//!
//! A Light holds Arc-shared state. Each shader that declares the light roots
//! keeps a weak list of the lights that reach it, and the renderer packs their
//! current state at each render. The passes and scenes that took a light hold
//! it, so it lights while one of them, or a handle, lives.

use glam::Vec3;
use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::Shader;
use crate::scene::SceneObject;
use crate::shader::ShaderObject;

/// Discriminant for the three light kinds. Wire-compatible with the WGSL
/// `Light.kind` field in `src/material/light.wgsl`: directional=0, point=1, spot=2.
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass(eq, eq_int, from_py_object))]
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
    #[error("Light::set_range expects a non-negative number, got {0}")]
    NegativeRange(f32),
}

/// Moves on every change to the lights a shader packs:
/// a light setter, a light list or ambient change, and a light drop.
static LIGHT_EPOCH: AtomicU64 = AtomicU64::new(1);

/// Marks every packed light set stale.
pub(crate) fn bump_epoch() {
    LIGHT_EPOCH.fetch_add(1, Ordering::Relaxed);
}

/// The current light epoch.
pub(crate) fn current_epoch() -> u64 {
    LIGHT_EPOCH.load(Ordering::Relaxed)
}

/// One light as a shader reads it, in the 64-byte WGSL `Light` layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GpuLight {
    pub(crate) kind: u32,
    pub(crate) intensity: f32,
    pub(crate) range: f32,
    pub(crate) inner_cone_cos: f32,
    pub(crate) position: [f32; 3],
    pub(crate) outer_cone_cos: f32,
    pub(crate) direction: [f32; 3],
    pub(crate) color: [f32; 3],
}

impl GpuLight {
    /// The size of one packed light.
    pub(crate) const SIZE: usize = 64;

    /// The white light `Material::pbr` draws with while no Light reaches it.
    pub(crate) fn placeholder() -> Self {
        Self {
            kind: LightKind::Directional as u32,
            intensity: 1.0,
            range: 0.0,
            inner_cone_cos: 1.0,
            position: [0.0; 3],
            outer_cone_cos: std::f32::consts::FRAC_PI_4.cos(),
            direction: [0.0, -1.0, 0.0],
            color: [1.0; 3],
        }
    }

    /// Appends the 64 bytes of this light to `out`.
    pub(crate) fn write(&self, out: &mut Vec<u8>) {
        let words: [u32; 16] = [
            self.kind,
            self.intensity.to_bits(),
            self.range.to_bits(),
            self.inner_cone_cos.to_bits(),
            self.position[0].to_bits(),
            self.position[1].to_bits(),
            self.position[2].to_bits(),
            self.outer_cone_cos.to_bits(),
            self.direction[0].to_bits(),
            self.direction[1].to_bits(),
            self.direction[2].to_bits(),
            0,
            self.color[0].to_bits(),
            self.color[1].to_bits(),
            self.color[2].to_bits(),
            0,
        ];
        for word in words {
            out.extend_from_slice(&word.to_le_bytes());
        }
    }
}

#[derive(Debug)]
pub(crate) struct LightObject {
    /// Unique in the process, so a shader's list dedupes by it.
    pub(crate) id: u64,
    state: RwLock<LightState>,
}

impl Drop for LightObject {
    fn drop(&mut self) {
        bump_epoch();
    }
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
    /// Stored as radians; converted to cosine when packed.
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
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Arc::new(Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            state: RwLock::new(state),
        })
    }

    /// The light as the shaders read it.
    pub(crate) fn gpu(&self) -> GpuLight {
        let state = *self.state.read();
        GpuLight {
            kind: state.kind as u32,
            intensity: state.intensity,
            range: state.range,
            inner_cone_cos: state.inner_cone_angle.cos(),
            position: state.position.to_array(),
            outer_cone_cos: state.outer_cone_angle.cos(),
            direction: state.direction.to_array(),
            color: state.color.to_array(),
        }
    }

    /// Applies `change` to the state and marks the packed lights stale.
    fn update(&self, change: impl FnOnce(&mut LightState)) {
        change(&mut self.state.write());
        bump_epoch();
    }

    /// Hands this light to every lit shader on `pass`, or to none when one is full.
    fn attach_to_pass(self: &Arc<Self>, pass: &crate::Pass) -> Result<(), crate::PassError> {
        let shaders: Vec<Arc<ShaderObject>> = pass.object.shaders.read().iter().cloned().collect();
        for shader in &shaders {
            shader.check_light_room(&[self.id])?;
        }
        for shader in &shaders {
            shader.add_light(self);
        }
        Ok(())
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
    /// to `(0, π/4)` (full center, 45° outer); tune via `set_cone_angles`.
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

    /// Update the linear-RGB color. Every shader the Light reaches draws
    /// with it at the next render. Defined for every kind.
    #[lsp_doc("docs/api/scene/light/set_color.md")]
    pub fn set_color(&self, color: [f32; 3]) -> Self {
        self.object.update(|state| state.color = Vec3::from(color));
        self.clone()
    }

    /// Update the scalar intensity multiplier. Every shader the Light
    /// reaches draws with it at the next render. Defined for every kind.
    #[lsp_doc("docs/api/scene/light/set_intensity.md")]
    pub fn set_intensity(&self, value: f32) -> Self {
        self.object.update(|state| state.intensity = value);
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
                self.object
                    .update(|state| state.position = Vec3::from(position));
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
                self.object
                    .update(|state| state.direction = Vec3::from(direction));
                Ok(self.clone())
            }
            LightKind::Point => Err(LightError::FieldNotApplicable {
                kind,
                field: "set_direction",
            }),
        }
    }

    /// Update the influence-radius cap. Errors on a directional light and
    /// on a negative or NaN value regardless of kind.
    #[lsp_doc("docs/api/scene/light/set_range.md")]
    pub fn set_range(&self, range: f32) -> Result<Self, LightError> {
        if range.is_nan() || range < 0.0 {
            return Err(LightError::NegativeRange(range));
        }
        let kind = self.object.state.read().kind;
        match kind {
            LightKind::Point | LightKind::Spot => {
                self.object.update(|state| state.range = range);
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
                self.object.update(|state| {
                    state.inner_cone_angle = inner;
                    state.outer_cone_angle = outer;
                });
                Ok(self.clone())
            }
            _ => Err(LightError::FieldNotApplicable {
                kind,
                field: "set_cone_angles",
            }),
        }
    }
}

/// The lights a pass holds, in add order, each once.
#[derive(Debug, Default)]
pub(crate) struct PassLights {
    list: Vec<Light>,
    ids: HashSet<u64>,
}

impl PassLights {
    /// Holds `light`, unless the pass already does.
    pub(crate) fn push(&mut self, light: &Light) {
        if self.ids.insert(light.object.id) {
            self.list.push(light.clone());
        }
    }

    /// The lights in add order.
    pub(crate) fn iter(&self) -> std::slice::Iter<'_, Light> {
        self.list.iter()
    }
}

impl SceneObject for Light {
    fn attach(&self, pass: &crate::Pass) -> Result<(), crate::PassError> {
        self.object.attach_to_pass(pass)?;
        // The pass holds the light, so it lights after the caller drops its handle.
        pass.object.lights.write().push(self);
        Ok(())
    }

    fn apply_to_shader(&self, shader: &Shader) {
        shader.object.add_light(&self.object);
    }
}

#[cfg(any(python, wasm, test))]
impl crate::error::Classify for LightError {
    fn classify(&self) -> crate::error::ErrorClass {
        crate::error::ErrorClass::Light
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
    fn set_range_refuses_nan() {
        let point = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(matches!(
            point.set_range(f32::NAN),
            Err(LightError::NegativeRange(_))
        ));
        assert_eq!(point.range(), Some(0.0), "the old range stays");
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
    fn lights_reach_the_shader_across_kinds() {
        // Three kinds in one pass, through the polymorphic `Pass::add`. The
        // shader packs each in attach order with its `kind` discriminant.
        let sun = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
        let lamp = Light::point([2.0, 1.0, 0.0], [1.0, 0.5, 0.5]).set_intensity(3.0);
        let torch = Light::spot([0.0, 3.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 0.8])
            .set_cone_angles(0.2, 0.6)
            .expect("spot cones");
        let material = Material::pbr();
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

        let kinds: Vec<u32> = material
            .shader()
            .object
            .gpu_lights()
            .iter()
            .map(|light| light.kind)
            .collect();
        assert_eq!(kinds, [0, 1, 2]);

        // A setter reaches the shader with no further `add`.
        sun.set_color([0.5, 0.25, 0.125]);
        let color = material.shader().object.gpu_lights()[0].color;
        assert_eq!(color, [0.5, 0.25, 0.125]);
    }

    #[test]
    fn replay_keeps_one_entry_per_light() {
        // A shader joining the pass replays every light on it. A light the
        // shader already holds keeps its one entry.
        let light = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
        let material = Material::pbr();
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        let pass = crate::Pass::new("scene");
        pass.add(&light).expect("light");
        pass.add(&model).expect("model");
        let other = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        pass.add(&other).expect("other");

        assert_eq!(material.shader().object.gpu_lights().len(), 1);
    }

    #[test]
    fn lights_follow_the_shader_across_passes() {
        // Two passes that share a Material share its shader, and so its light
        // list: a light added to either pass lights both.
        let material = Material::pbr();
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

        let kinds: Vec<u32> = material
            .shader()
            .object
            .gpu_lights()
            .iter()
            .map(|light| light.kind)
            .collect();
        assert_eq!(kinds, [0, 1], "both lights reach the shared shader");
    }

    #[test]
    fn a_light_lives_while_a_pass_or_scene_holds_it() {
        let material = Material::pbr();
        let pass = crate::Pass::new("held by a pass");
        pass.add(&crate::scene::Model::new(
            pbr_triangle_mesh(),
            material.clone(),
        ))
        .expect("model");
        pass.add(&Light::point([0.0, 1.0, 0.0], [1.0; 3]))
            .expect("light");
        assert_eq!(material.shader().object.gpu_lights()[0].kind, 1);

        let scene_material = Material::pbr();
        let scene = crate::Scene::new();
        scene
            .add(&crate::scene::Model::new(
                pbr_triangle_mesh(),
                scene_material.clone(),
            ))
            .expect("model");
        scene
            .add(&Light::point([0.0, 1.0, 0.0], [1.0; 3]))
            .expect("light");
        assert_eq!(scene_material.shader().object.gpu_lights()[0].kind, 1);

        // Once nothing holds the light, the shader falls back to its placeholder.
        drop(pass);
        assert_eq!(
            material.shader().object.gpu_lights(),
            [GpuLight::placeholder()]
        );
    }

    #[test]
    fn dropped_lights_leave_room_under_the_cap() {
        // The list keeps dropped lights until it prunes, and the cap counts live ones.
        crate::shader::lower_test_cap(4);
        let material = Material::pbr();
        let first = crate::Pass::new("first");
        first
            .add(&crate::scene::Model::new(
                pbr_triangle_mesh(),
                material.clone(),
            ))
            .expect("model");
        for index in 0..4 {
            first
                .add(&Light::point([index as f32, 0.0, 0.0], [1.0; 3]))
                .expect("under the cap");
        }
        drop(first);
        let second = crate::Pass::new("second");
        second
            .add(&crate::scene::Model::new(
                pbr_triangle_mesh(),
                material.clone(),
            ))
            .expect("model");
        for index in 0..4 {
            second
                .add(&Light::point([index as f32, 1.0, 0.0], [1.0; 3]))
                .expect("the dropped lights leave room");
        }
        let fifth = second.add(&Light::point([0.0, 2.0, 0.0], [1.0; 3]));
        assert!(matches!(
            fifth,
            Err(crate::PassError::LightCapReached { cap: 4 })
        ));
        assert_eq!(material.shader().object.gpu_lights().len(), 4);
    }

    #[test]
    fn ambient_default_seeds_to_dim_gray() {
        // Material::pbr seeds `lights.ambient = [0.03; 3]` so a fresh
        // material's shader has a sensible non-zero ambient.
        let material = Material::pbr();
        let amb: [f32; 3] = material.shader().get("lights.ambient").unwrap();
        assert_eq!(amb, [0.03, 0.03, 0.03]);
    }
}
