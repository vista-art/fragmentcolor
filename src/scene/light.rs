//! Light — single type covering the three glTF `KHR_lights_punctual` kinds
//! (directional, point, spot). The kind is set once at construction via the
//! [`Light::directional`] / [`Light::point`] / [`Light::spot`] constructors;
//! kind-specific setters (`set_position`, `set_direction`, `set_range`,
//! `set_cone_angles`) return a typed [`LightError::FieldNotApplicable`] when
//! called on the wrong kind, and the matching getters return `None`.
//! Every constructor and setter returns [`LightError::NotFinite`] for NaN or
//! an infinity, and a refused setter leaves the light as it was.
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
use crate::core::share_light;
use crate::scene::{NotFinite, SceneObject, check_finite};

/// Discriminant for the three light kinds. Wire-compatible with the WGSL
/// `Light.kind` field in `src/scene/material/light.wgsl`: directional=0, point=1, spot=2.
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass(eq, eq_int, from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum LightKind {
    Directional = 0,
    Point = 1,
    Spot = 2,
}

/// Errors from [`Light`] constructors and setters: a field that doesn't
/// apply to the light's kind, a negative range, or a number that isn't finite.
#[derive(Debug, thiserror::Error)]
pub enum LightError {
    #[error("Light::{field} does not apply to a {kind:?} light")]
    FieldNotApplicable {
        kind: LightKind,
        field: &'static str,
    },
    #[error("Light::set_range expects a non-negative number, got {0}")]
    NegativeRange(f32),
    /// A number that isn't finite, such as NaN or an infinity, which `label` names with its call.
    #[error("{label} must be finite, got {value}")]
    NotFinite { label: String, value: f32 },
}

impl From<NotFinite> for LightError {
    fn from(NotFinite { label, value }: NotFinite) -> Self {
        LightError::NotFinite { label, value }
    }
}

/// Moves on every change to the lights a shader packs:
/// a light setter, a light list or ambient change, and a light drop.
static LIGHT_EPOCH: AtomicU64 = AtomicU64::new(1);

/// Marks every packed light set stale.
pub(crate) fn bump_epoch() {
    LIGHT_EPOCH.fetch_add(1, Ordering::Relaxed);
}

/// The current light epoch.
pub(crate) fn read_epoch() -> u64 {
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
    pub(crate) fn to_gpu(&self) -> GpuLight {
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

    /// Applies `change` when the light is one of `kinds`, and otherwise refuses the setter `field`.
    fn update_kinds(
        &self,
        kinds: &[LightKind],
        field: &'static str,
        change: impl FnOnce(&mut LightState),
    ) -> Result<(), LightError> {
        let kind = self.state.read().kind;
        if !kinds.contains(&kind) {
            return Err(LightError::FieldNotApplicable { kind, field });
        }
        self.update(change);
        Ok(())
    }

    /// Tints the light `color`, refusing a channel that isn't finite first.
    fn set_color(&self, color: [f32; 3]) -> Result<(), LightError> {
        check_finite("Light::set_color", None, &color)?;
        self.update(|state| state.color = Vec3::from(color));
        Ok(())
    }

    /// Scales the light's color by `value`, refusing one that isn't finite first.
    fn set_intensity(&self, value: f32) -> Result<(), LightError> {
        check_finite("Light::set_intensity", Some("value"), &[value])?;
        self.update(|state| state.intensity = value);
        Ok(())
    }

    /// Moves a point or spot light to `position`, refusing an item that isn't finite first.
    fn set_position(&self, position: [f32; 3]) -> Result<(), LightError> {
        check_finite("Light::set_position", None, &position)?;
        let kinds = [LightKind::Point, LightKind::Spot];
        let position = Vec3::from(position);
        self.update_kinds(&kinds, "set_position", |state| state.position = position)
    }

    /// Points a directional or spot light along `direction`, refusing an item that isn't finite first.
    fn set_direction(&self, direction: [f32; 3]) -> Result<(), LightError> {
        check_finite("Light::set_direction", None, &direction)?;
        let kinds = [LightKind::Directional, LightKind::Spot];
        let direction = Vec3::from(direction);
        self.update_kinds(&kinds, "set_direction", |state| state.direction = direction)
    }

    /// Caps how far a point or spot light reaches, refusing a negative or NaN `range` first.
    fn set_range(&self, range: f32) -> Result<(), LightError> {
        if range.is_nan() || range < 0.0 {
            return Err(LightError::NegativeRange(range));
        }
        let kinds = [LightKind::Point, LightKind::Spot];
        self.update_kinds(&kinds, "set_range", |state| state.range = range)
    }

    /// Sets a spot light's cone half-angles, refusing one that isn't finite first.
    fn set_cone_angles(&self, inner: f32, outer: f32) -> Result<(), LightError> {
        check_finite("Light::set_cone_angles", Some("inner"), &[inner])?;
        check_finite("Light::set_cone_angles", Some("outer"), &[outer])?;
        self.update_kinds(&[LightKind::Spot], "set_cone_angles", |state| {
            state.inner_cone_angle = inner;
            state.outer_cone_angle = outer;
        })
    }

    /// Hands this light to every lit shader on `pass` and to the pass, or to none when
    /// a shader is full. It holds the pass's shaders, then its lights, the order a joining
    /// shader takes them, so that shader either is among the ones lit here or finds the
    /// light on the pass.
    fn attach_to_pass(
        self: &Arc<Self>,
        pass: &crate::Pass,
        light: &Light,
    ) -> Result<(), crate::PassError> {
        let shaders = pass.object.shaders.read();
        let mut lights = pass.object.lights.write();
        share_light(&shaders, self)?;
        // The pass holds the light, so it lights after the caller drops its handle.
        lights.push(light);
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
    pub fn directional(direction: [f32; 3], color: [f32; 3]) -> Result<Self, LightError> {
        let parts = LightParts::check("Light::directional", None, Some(direction), color)?;
        Ok(Self::create(LightKind::Directional, parts))
    }

    /// Build a point light — radiates from `position` with inverse-square
    /// distance falloff. `set_range` caps the influence radius (default 0 =
    /// unlimited, matching glTF `KHR_lights_punctual`).
    #[lsp_doc("docs/api/scene/light/point.md")]
    pub fn point(position: [f32; 3], color: [f32; 3]) -> Result<Self, LightError> {
        let parts = LightParts::check("Light::point", Some(position), None, color)?;
        Ok(Self::create(LightKind::Point, parts))
    }

    /// Build a spot light — point light constrained to a cone. `position`
    /// is the apex, `direction` is the cone axis, cone half-angles default
    /// to `(0, π/4)` (full center, 45° outer); tune via `set_cone_angles`.
    #[lsp_doc("docs/api/scene/light/spot.md")]
    pub fn spot(
        position: [f32; 3],
        direction: [f32; 3],
        color: [f32; 3],
    ) -> Result<Self, LightError> {
        let parts = LightParts::check("Light::spot", Some(position), Some(direction), color)?;
        Ok(Self::create(LightKind::Spot, parts))
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
    pub fn set_color(&self, color: [f32; 3]) -> Result<Self, LightError> {
        self.object.set_color(color)?;
        Ok(self.clone())
    }

    /// Update the scalar intensity multiplier. Every shader the Light
    /// reaches draws with it at the next render. Defined for every kind.
    #[lsp_doc("docs/api/scene/light/set_intensity.md")]
    pub fn set_intensity(&self, value: f32) -> Result<Self, LightError> {
        self.object.set_intensity(value)?;
        Ok(self.clone())
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
        self.object.set_position(position)?;
        Ok(self.clone())
    }

    /// Update the world-space direction. Errors on a point light.
    #[lsp_doc("docs/api/scene/light/set_direction.md")]
    pub fn set_direction(&self, direction: [f32; 3]) -> Result<Self, LightError> {
        self.object.set_direction(direction)?;
        Ok(self.clone())
    }

    /// Update the influence-radius cap. Errors on a directional light and
    /// on a negative or NaN value regardless of kind.
    #[lsp_doc("docs/api/scene/light/set_range.md")]
    pub fn set_range(&self, range: f32) -> Result<Self, LightError> {
        self.object.set_range(range)?;
        Ok(self.clone())
    }

    /// Update the cone half-angles (radians). Errors on a non-spot light.
    #[lsp_doc("docs/api/scene/light/set_cone_angles.md")]
    pub fn set_cone_angles(&self, inner: f32, outer: f32) -> Result<Self, LightError> {
        self.object.set_cone_angles(inner, outer)?;
        Ok(self.clone())
    }
}

impl Light {
    /// A light of `kind` from `parts`, which a constructor has checked.
    fn create(kind: LightKind, parts: LightParts) -> Self {
        let LightParts {
            position,
            direction,
            color,
        } = parts;
        let state = LightState::defaults(kind, direction, position, color);
        Self {
            object: LightObject::new(state),
        }
    }

    /// The light a Scene adds when it has none: white, from above and in front.
    /// Its constants are finite, so it skips the checks.
    pub(crate) fn create_stock() -> Self {
        let parts = LightParts {
            position: Vec3::ZERO,
            direction: Vec3::new(0.0, -0.3, -1.0),
            color: Vec3::ONE,
        };
        Self::create(LightKind::Directional, parts)
    }
}

/// The vectors a light starts from. A directional light sits at the origin
/// and a point light points straight down, though neither reads that field.
struct LightParts {
    position: Vec3,
    direction: Vec3,
    color: Vec3,
}

impl LightParts {
    /// The parts `call` took, each item checked to be finite, in the order the call takes them.
    fn check(
        call: &str,
        position: Option<[f32; 3]>,
        direction: Option<[f32; 3]>,
        color: [f32; 3],
    ) -> Result<Self, LightError> {
        if let Some(position) = position {
            check_finite(call, Some("position"), &position)?;
        }
        if let Some(direction) = direction {
            check_finite(call, Some("direction"), &direction)?;
        }
        check_finite(call, Some("color"), &color)?;
        Ok(Self {
            position: position.map_or(Vec3::ZERO, Vec3::from),
            direction: direction.map_or(Vec3::NEG_Y, Vec3::from),
            color: Vec3::from(color),
        })
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
        self.object.attach_to_pass(pass, self)
    }

    fn apply_to_shader(&self, shader: &Shader) {
        shader.object.add_light(&self.object);
    }
}

/// A number that isn't finite is an invalid input, which the bindings refuse at the call.
/// The other variants are the light's own refusals.
#[cfg(any(python, wasm, test))]
impl crate::errors::Classify for LightError {
    fn classify(&self) -> crate::errors::ErrorClass {
        use crate::errors::ErrorClass;
        match self {
            LightError::NotFinite { .. } => ErrorClass::InvalidInput,
            LightError::FieldNotApplicable { .. } | LightError::NegativeRange(_) => {
                ErrorClass::Light
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Material;

    fn pbr_triangle_mesh() -> crate::Mesh {
        let mesh = crate::Mesh::new();
        mesh.add_vertices(
            [[0.0, 0.5, 0.0], [-0.5, -0.5, 0.0], [0.5, -0.5, 0.0]].map(crate::Vertex::pbr),
        );
        mesh
    }

    // Story: a light added while a Model's shader joins the same pass reaches that
    // shader, whichever comes first. The join waits for the light, then finds it on the pass.
    #[test]
    fn a_light_reaches_a_shader_that_joins_meanwhile() {
        let pass = crate::Pass::new("racing");
        let model = crate::Model::new(pbr_triangle_mesh(), Material::pbr());
        let shader = model.material.shader.object.clone();
        let joiner = std::rc::Rc::new(std::cell::RefCell::new(None));
        let handle = joiner.clone();
        let (racing, added) = (pass.clone(), model.clone());
        crate::testing::race_hooks::once("light checked", move || {
            let (sender, receiver) = std::sync::mpsc::channel();
            let thread = std::thread::spawn(move || {
                racing.add(&added).expect("add the model");
                let _ = sender.send(());
            });
            let _ = receiver.recv_timeout(std::time::Duration::from_millis(200));
            *handle.borrow_mut() = Some(thread);
        });

        let lamp = Light::point([0.0, 1.0, 0.0], [1.0; 3]).expect("light");
        pass.add(&lamp).expect("add the light");
        if let Some(thread) = joiner.borrow_mut().take() {
            thread.join().expect("the model thread");
        }

        assert_eq!(
            shader.summarize_lights().count,
            1,
            "the joined shader holds the light"
        );
    }

    #[test]
    fn constructors_report_their_kind() {
        let d = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).expect("light");
        let p = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).expect("light");
        let s = Light::spot([0.0, 5.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).expect("light");
        assert_eq!(d.kind(), LightKind::Directional);
        assert_eq!(p.kind(), LightKind::Point);
        assert_eq!(s.kind(), LightKind::Spot);
    }

    #[test]
    fn universal_setters_round_trip_for_every_kind() {
        for light in [
            Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).expect("light"),
            Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).expect("light"),
            Light::spot([0.0, 5.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).expect("light"),
        ] {
            light
                .set_color([0.2, 0.4, 0.6])
                .expect("color")
                .set_intensity(2.5)
                .expect("intensity");
            assert_eq!(light.color(), [0.2, 0.4, 0.6]);
            assert!((light.intensity() - 2.5).abs() < 1.0e-6);
        }
    }

    #[test]
    fn kind_specific_setters_err_on_wrong_kind() {
        let directional = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).expect("light");
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

        let point = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).expect("light");
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
        let point = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).expect("light");
        assert!(matches!(
            point.set_range(f32::NAN),
            Err(LightError::NegativeRange(_))
        ));
        assert_eq!(point.range(), Some(0.0), "the old range stays");
    }

    // Story: every constructor and setter refuses NaN and an infinity with an error that
    // names the call and the item, and a refused setter leaves the light as it was.
    #[test]
    fn numbers_that_are_not_finite_are_refused() {
        let built = [
            Light::directional([0.0, f32::NAN, 0.0], [1.0; 3]).err(),
            Light::point([0.0; 3], [1.0, f32::INFINITY, 1.0]).err(),
            Light::spot([0.0; 3], [0.0, 0.0, f32::NEG_INFINITY], [1.0; 3]).err(),
        ]
        .map(|error| error.map(|error| error.to_string()).unwrap_or_default());
        let spot = Light::spot([0.0, 2.0, 0.0], [0.0, -1.0, 0.0], [0.5; 3])
            .expect("spot")
            .set_intensity(2.0)
            .expect("intensity")
            .set_cone_angles(0.1, 0.5)
            .expect("cones");
        let before = spot.object.to_gpu();

        let refused = [
            spot.set_color([f32::NAN, 0.0, 0.0]).err(),
            spot.set_intensity(f32::INFINITY).err(),
            spot.set_position([0.0, 0.0, f32::NAN]).err(),
            spot.set_direction([f32::NEG_INFINITY, 0.0, 0.0]).err(),
            spot.set_cone_angles(f32::NAN, 0.5).err(),
            spot.set_cone_angles(0.1, f32::INFINITY).err(),
        ]
        .map(|error| error.map(|error| error.to_string()).unwrap_or_default());

        assert_eq!(
            built,
            [
                "Light::directional: direction: item 1 must be finite, got NaN",
                "Light::point: color: item 1 must be finite, got inf",
                "Light::spot: direction: item 2 must be finite, got -inf",
            ]
        );
        assert_eq!(
            refused,
            [
                "Light::set_color: item 0 must be finite, got NaN",
                "Light::set_intensity: value must be finite, got inf",
                "Light::set_position: item 2 must be finite, got NaN",
                "Light::set_direction: item 0 must be finite, got -inf",
                "Light::set_cone_angles: inner must be finite, got NaN",
                "Light::set_cone_angles: outer must be finite, got inf",
            ]
        );
        assert_eq!(
            spot.object.to_gpu(),
            before,
            "a refused setter keeps the light"
        );
    }

    #[test]
    fn kind_specific_setters_ok_and_round_trip_on_right_kind() {
        let point = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).expect("light");
        point
            .set_position([3.0, 1.5, -2.0])
            .expect("position on point");
        point.set_range(8.0).expect("range on point");
        assert_eq!(point.position(), Some([3.0, 1.5, -2.0]));
        assert_eq!(point.range(), Some(8.0));
        assert!(point.direction().is_none(), "point has no direction");
        assert!(point.inner_cone_angle().is_none());
        assert!(point.outer_cone_angle().is_none());

        let spot = Light::spot([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).expect("light");
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
        let sun = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).expect("light");
        let lamp = Light::point([2.0, 1.0, 0.0], [1.0, 0.5, 0.5])
            .expect("light")
            .set_intensity(3.0)
            .expect("intensity");
        let torch = Light::spot([0.0, 3.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 0.8])
            .expect("light")
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
            .collect_lights()
            .iter()
            .map(|light| light.kind)
            .collect();
        assert_eq!(kinds, [0, 1, 2]);

        // A setter reaches the shader with no further `add`.
        sun.set_color([0.5, 0.25, 0.125]).expect("color");
        let color = material.shader().object.collect_lights()[0].color;
        assert_eq!(color, [0.5, 0.25, 0.125]);
    }

    #[test]
    fn replay_keeps_one_entry_per_light() {
        // A shader joining the pass replays every light on it. A light the
        // shader already holds keeps its one entry.
        let light = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).expect("light");
        let material = Material::pbr();
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        let pass = crate::Pass::new("scene");
        pass.add(&light).expect("light");
        pass.add(&model).expect("model");
        let other = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        pass.add(&other).expect("other");

        assert_eq!(material.shader().object.collect_lights().len(), 1);
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

        let sun = Light::directional([0.0, -1.0, 0.0], [1.0; 3]).expect("light");
        let lamp = Light::point([0.0, 1.0, 0.0], [1.0; 3]).expect("light");
        pass_a.add(&sun).expect("sun on pass_a");
        pass_b.add(&lamp).expect("lamp on pass_b");

        let kinds: Vec<u32> = material
            .shader()
            .object
            .collect_lights()
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
        pass.add(&Light::point([0.0, 1.0, 0.0], [1.0; 3]).expect("light"))
            .expect("light");
        assert_eq!(material.shader().object.collect_lights()[0].kind, 1);

        let scene_material = Material::pbr();
        let scene = crate::Scene::new();
        scene
            .add(&crate::scene::Model::new(
                pbr_triangle_mesh(),
                scene_material.clone(),
            ))
            .expect("model");
        scene
            .add(&Light::point([0.0, 1.0, 0.0], [1.0; 3]).expect("light"))
            .expect("light");
        assert_eq!(scene_material.shader().object.collect_lights()[0].kind, 1);

        // Once nothing holds the light, the shader falls back to its placeholder.
        drop(pass);
        assert_eq!(
            material.shader().object.collect_lights(),
            [GpuLight::placeholder()]
        );
    }

    #[test]
    fn dropped_lights_leave_room_under_the_cap() {
        // The list keeps dropped lights until it prunes, and the cap counts live ones.
        crate::testing::light_cap::lower(4);
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
                .add(&Light::point([index as f32, 0.0, 0.0], [1.0; 3]).expect("light"))
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
                .add(&Light::point([index as f32, 1.0, 0.0], [1.0; 3]).expect("light"))
                .expect("the dropped lights leave room");
        }
        let fifth = second.add(&Light::point([0.0, 2.0, 0.0], [1.0; 3]).expect("light"));
        assert!(matches!(
            fifth,
            Err(crate::PassError::LightCapReached { cap: 4 })
        ));
        assert_eq!(material.shader().object.collect_lights().len(), 4);
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
