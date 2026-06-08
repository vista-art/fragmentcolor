//! Scene — top-level container for [`Model`](crate::Model),
//! [`Camera`](crate::Camera), [`Light`](crate::Light) (directional / point
//! / spot, all one type), and any user-defined
//! [`SceneObject`](crate::scene::SceneObject). Owns one or more
//! [`Pass`](crate::Pass) entries underneath and implements
//! [`Renderable`](crate::Renderable), so the whole scene goes to the
//! [`Renderer`](crate::Renderer) in a single call.
//!
//! `Scene::new()` is sync — no `Renderer` argument, no async, nothing to
//! await. The first time a SceneObject is added the Scene allocates a Pass
//! to absorb it; the first time the Scene is rendered the underlying GPU
//! resources initialise on demand. Same lazy-init pattern the rest of
//! FragmentColor follows.
//!
//! A Scene owns one ordered `Vec<Pass>`. Loaders, builders, and user code
//! all append into the same vec, and `Renderable for Scene` iterates it in
//! order. There's no privileged "default" pass: the pass that absorbs
//! `Scene::add` objects is an ordinary member of the vec, and the CRUD
//! surface (`add_pass`, `remove_pass`, `get_pass`, `list_passes`,
//! `set_passes`) lets the caller read, append, reorder, or replace the
//! whole graph — the same composability every layer below `Scene` already
//! has.
//!
//! When the user has added Models but no Camera or Light, the Scene injects
//! sensible defaults at render time so the "hello world" path renders
//! something recognisable. As soon as you add your own Camera / Light, the
//! defaults stop firing. Composition callers that drive every uniform
//! themselves can turn the injection off with `no_defaults` (or the
//! per-kind `no_default_camera` / `no_default_light`), or replace the stock
//! values with `set_default_camera` / `set_default_light`.

use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::sync::Arc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::scene::{Camera, Light, SceneObject};
use crate::{Pass, PassError, PassObject, Renderable};

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/scene/scene/scene.md")]
pub struct Scene {
    pub(crate) inner: Arc<SceneInner>,
}

#[derive(Debug)]
pub(crate) struct SceneInner {
    /// The Scene's ordered pass graph. Loaders, builders, `add_pass`, and
    /// the lazily-created absorb Pass all push into this one vec;
    /// `Renderable for Scene` iterates it in order. Empty until the first
    /// `add` / `add_pass`, so a fresh Scene allocates no GPU bookkeeping.
    pub(crate) passes: RwLock<Vec<Pass>>,
    /// Handle to the Pass inside `passes` that absorbs `Scene::add` objects
    /// (Models / Cameras / Lights). Created lazily on the first `add` and
    /// appended to `passes` at that point, so it keeps its insertion-order
    /// position relative to any `add_pass` calls. `None` until then, and
    /// cleared again if the caller removes it from the graph via
    /// `remove_pass` / `set_passes`.
    pub(crate) absorb_pass: RwLock<Option<Pass>>,
    /// Sticky once-set flags so the default-Camera / default-Light injection
    /// at `passes()` time only fires once and only when the user hasn't
    /// supplied their own.
    pub(crate) has_camera: RwLock<bool>,
    pub(crate) has_light: RwLock<bool>,
    /// User-facing injection toggles (both default `true`). Cleared by
    /// `no_defaults` / `no_default_camera` / `no_default_light` so a
    /// composition caller that overrides every uniform never gets FC's
    /// stock Camera / Light injected on top.
    pub(crate) inject_camera: RwLock<bool>,
    pub(crate) inject_light: RwLock<bool>,
    /// Optional caller-supplied replacements for the stock default Camera /
    /// Light. Set via `set_default_camera` / `set_default_light`; consumed
    /// by the injection path at first render in place of FC's stock values.
    pub(crate) default_camera: RwLock<Option<Camera>>,
    pub(crate) default_light: RwLock<Option<Light>>,
    /// Scene-wide ambient color (`lights.ambient` in the PBR shader). Set
    /// via `Scene::ambient`; cached so Models added afterwards inherit it.
    pub(crate) ambient: RwLock<Option<[f32; 3]>>,
    /// Typed handles to every SceneObject the user (or the loader) added.
    /// Each handle is an Arc-shared `Clone` of the original, so mutating
    /// one of these from `scene.cameras()[i].look_at(...)` propagates to
    /// every shader the Camera was wired into — same live-handle semantics
    /// the `Pass::add` path already gives.
    pub(crate) models: RwLock<Vec<crate::scene::Model>>,
    pub(crate) cameras: RwLock<Vec<Camera>>,
    pub(crate) lights: RwLock<Vec<Light>>,
}

crate::impl_fc_kind!(Scene, "Scene");

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    #[lsp_doc("docs/api/scene/scene/load.md")]
    pub fn load(
        source: impl Into<crate::scene::SceneSource>,
    ) -> Result<Self, crate::scene::SceneLoadError> {
        crate::scene::loader::load(source.into())
    }

    #[lsp_doc("docs/api/scene/scene/new.md")]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SceneInner {
                passes: RwLock::new(Vec::new()),
                absorb_pass: RwLock::new(None),
                has_camera: RwLock::new(false),
                has_light: RwLock::new(false),
                inject_camera: RwLock::new(true),
                inject_light: RwLock::new(true),
                default_camera: RwLock::new(None),
                default_light: RwLock::new(None),
                ambient: RwLock::new(None),
                models: RwLock::new(Vec::new()),
                cameras: RwLock::new(Vec::new()),
                lights: RwLock::new(Vec::new()),
            }),
        }
    }

    #[lsp_doc("docs/api/scene/scene/add.md")]
    pub fn add<O: SceneObject + 'static>(&self, object: &O) -> Result<&Self, PassError> {
        let pass = self.ensure_absorb_pass();
        pass.add(object)?;
        // Stash a typed Arc-clone of the object alongside the pass attach
        // so `scene.cameras()` / `lights()` / `models()` can hand back live
        // handles. TypeId equality is exact — wrapping a Camera in a user
        // newtype counts as "user-supplied" and skips the typed lane (it
        // still rides the pass as a `SceneObject`); custom types just
        // wouldn't show up in the typed-getter slot, which matches what
        // the user asked for by reaching for a custom type.
        let any = object as &dyn std::any::Any;
        if let Some(camera) = any.downcast_ref::<Camera>() {
            self.inner.cameras.write().push(camera.clone());
            *self.inner.has_camera.write() = true;
        } else if let Some(light) = any.downcast_ref::<Light>() {
            self.inner.lights.write().push(light.clone());
            *self.inner.has_light.write() = true;
        } else if let Some(model) = any.downcast_ref::<crate::scene::Model>() {
            self.inner.models.write().push(model.clone());
        }
        // Re-stamp the stashed ambient onto whatever shaders just joined
        // so callers can `scene.ambient(...)` before any models are added
        // and still see the value carry through. Empty / no-shaders adds
        // (Camera, Light) skip the write silently — the next Model add
        // will pick it up.
        if let Some(amb) = *self.inner.ambient.read() {
            for shader in pass.object.shaders.read().iter() {
                let _ = shader.set("lights.ambient", amb);
            }
        }
        Ok(self)
    }

    /// Snapshot of every [`Model`](crate::Model) added to this Scene via
    /// [`Scene::add`] (including Models the loader created from glTF
    /// `mesh` nodes). Each entry is an Arc-shared clone of the original
    /// handle — mutating a returned `Model` (`set_visible`, `translate`,
    /// …) propagates live to every shader the Model was wired into, with
    /// no re-attach needed.
    #[lsp_doc("docs/api/scene/scene/models.md")]
    pub fn models(&self) -> Vec<crate::scene::Model> {
        self.inner.models.read().clone()
    }

    /// Snapshot of every [`Camera`](crate::Camera) added to this Scene via
    /// [`Scene::add`] (including Cameras the loader created from glTF
    /// `camera` nodes, unless you skipped them via the loader's camera
    /// filter). Each entry is an Arc-shared clone — `camera.look_at(...)`
    /// on a returned handle drives every shader the Camera is wired into.
    #[lsp_doc("docs/api/scene/scene/cameras.md")]
    pub fn cameras(&self) -> Vec<Camera> {
        self.inner.cameras.read().clone()
    }

    /// Snapshot of every [`Light`](crate::Light) added to this Scene via
    /// [`Scene::add`] (including Lights the loader created from glTF
    /// `KHR_lights_punctual` nodes, unless you skipped them via the loader's
    /// light filter). Each entry is an Arc-shared clone — `light.set_color(...)`
    /// on a returned handle drives every shader the Light occupies a slot in.
    #[lsp_doc("docs/api/scene/scene/lights.md")]
    pub fn lights(&self) -> Vec<Light> {
        self.inner.lights.read().clone()
    }

    #[lsp_doc("docs/api/scene/scene/ambient.md")]
    pub fn ambient(&self, color: [f32; 3]) -> &Self {
        *self.inner.ambient.write() = Some(color);
        // Stamp onto every shader currently in the scene. Future Models
        // added via `Scene::add` pick the value up from the stash.
        for pass in self.inner.passes.read().iter() {
            for shader in pass.object.shaders.read().iter() {
                let _ = shader.set("lights.ambient", color);
            }
        }
        self
    }

    #[lsp_doc("docs/api/scene/scene/add_pass.md")]
    pub fn add_pass(&self, pass: &Pass) -> &Self {
        self.inner.passes.write().push(pass.clone());
        self
    }

    #[lsp_doc("docs/api/scene/scene/remove_pass.md")]
    pub fn remove_pass(&self, pass: &Pass) -> bool {
        // Lock order: `passes` first, `absorb_pass` second (see
        // `ensure_absorb_pass`).
        let mut passes = self.inner.passes.write();
        let Some(idx) = passes
            .iter()
            .position(|p| Arc::ptr_eq(&p.object, &pass.object))
        else {
            return false;
        };
        passes.remove(idx);
        // Forget the absorb handle if it pointed at the pass we just removed,
        // so the next `add` rebuilds one instead of attaching to a Pass the
        // graph no longer renders.
        let mut slot = self.inner.absorb_pass.write();
        let was_absorb = slot
            .as_ref()
            .is_some_and(|absorb| Arc::ptr_eq(&absorb.object, &pass.object));
        if was_absorb {
            *slot = None;
        }
        true
    }

    #[lsp_doc("docs/api/scene/scene/get_pass.md")]
    pub fn get_pass(&self, index: usize) -> Option<Pass> {
        self.inner.passes.read().get(index).cloned()
    }

    #[lsp_doc("docs/api/scene/scene/list_passes.md")]
    pub fn list_passes(&self) -> Vec<Pass> {
        self.inner.passes.read().clone()
    }

    #[lsp_doc("docs/api/scene/scene/set_passes.md")]
    pub fn set_passes(&self, passes: Vec<Pass>) {
        // Lock order: `passes` first, `absorb_pass` second (see
        // `ensure_absorb_pass`).
        let mut current = self.inner.passes.write();
        *current = passes;
        // Drop the absorb handle if the replacement no longer contains it.
        let mut slot = self.inner.absorb_pass.write();
        let dropped = slot.as_ref().is_some_and(|absorb| {
            !current
                .iter()
                .any(|p| Arc::ptr_eq(&p.object, &absorb.object))
        });
        if dropped {
            *slot = None;
        }
    }

    #[lsp_doc("docs/api/scene/scene/no_defaults.md")]
    pub fn no_defaults(&self) -> &Self {
        *self.inner.inject_camera.write() = false;
        *self.inner.inject_light.write() = false;
        self
    }

    #[lsp_doc("docs/api/scene/scene/no_default_camera.md")]
    pub fn no_default_camera(&self) -> &Self {
        *self.inner.inject_camera.write() = false;
        self
    }

    #[lsp_doc("docs/api/scene/scene/no_default_light.md")]
    pub fn no_default_light(&self) -> &Self {
        *self.inner.inject_light.write() = false;
        self
    }

    #[lsp_doc("docs/api/scene/scene/set_default_camera.md")]
    pub fn set_default_camera(&self, camera: &Camera) -> &Self {
        *self.inner.default_camera.write() = Some(camera.clone());
        // Naming a default camera is an explicit request to inject it, so
        // re-arm injection even if `no_default_camera` ran earlier.
        *self.inner.inject_camera.write() = true;
        self
    }

    #[lsp_doc("docs/api/scene/scene/set_default_light.md")]
    pub fn set_default_light(&self, light: &Light) -> &Self {
        *self.inner.default_light.write() = Some(light.clone());
        *self.inner.inject_light.write() = true;
        self
    }

    /// Lazily build the absorb Pass on first `add` and append it to the
    /// pass graph. The Pass is named so it shows up identifiably in graphics
    /// debuggers (RenderDoc, Xcode GPU frame capture). If the caller dropped
    /// a previously-built absorb Pass from the graph (via `remove_pass` /
    /// `set_passes`), a fresh one is appended at the current end.
    ///
    /// Lock order: `passes` is taken first, `absorb_pass` second — the same
    /// order [`Scene::remove_pass`] and [`Scene::set_passes`] use, so the
    /// three can't deadlock against each other under concurrent mutation.
    fn ensure_absorb_pass(&self) -> Pass {
        let mut passes = self.inner.passes.write();
        // Reuse the existing absorb Pass only if it's still in the graph.
        let existing = self.inner.absorb_pass.read().clone();
        if let Some(p) = existing
            && passes.iter().any(|x| Arc::ptr_eq(&x.object, &p.object))
        {
            return p;
        }
        let pass = Pass::new("Scene Default Pass");
        passes.push(pass.clone());
        *self.inner.absorb_pass.write() = Some(pass.clone());
        pass
    }

    /// Inject default Camera / Light into the absorb Pass when injection is
    /// enabled and the user hasn't supplied their own. Idempotent — the
    /// sticky `has_camera` / `has_light` flags flip on first injection, so
    /// subsequent `passes()` calls are no-ops on this front. Defaults are
    /// also stashed into the Scene's typed lanes so `scene.cameras()` /
    /// `scene.lights()` surface them — anyone fishing the default camera
    /// back out to drive it per frame should hit the getter and find it
    /// there. With no absorb Pass (a pure `add_pass` composition) there's
    /// nothing to attach to, so injection is skipped.
    fn ensure_render_defaults(&self) {
        let needs_camera = *self.inner.inject_camera.read() && !*self.inner.has_camera.read();
        let needs_light = *self.inner.inject_light.read() && !*self.inner.has_light.read();
        if !needs_camera && !needs_light {
            return;
        }
        let Some(pass) = self.inner.absorb_pass.read().clone() else {
            return;
        };
        if needs_camera {
            // A caller-supplied default (set_default_camera) wins; otherwise
            // fall back to the stock one: 60° vertical FOV, square aspect, a
            // comfortable [0.1, 100] depth range, eye five units back from
            // the origin looking at it with conventional +Y up. Fine for
            // offscreen test targets and for someone trying the API for the
            // first time; users with a non-square target supply their own
            // Camera.
            let camera = self.inner.default_camera.read().clone().unwrap_or_else(|| {
                Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).look_at(
                    [0.0, 0.0, 5.0],
                    [0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                )
            });
            // Camera::attach always succeeds — discard the never-error result.
            let _ = pass.add(&camera);
            self.inner.cameras.write().push(camera);
            *self.inner.has_camera.write() = true;
        }
        if needs_light {
            // A caller-supplied default (set_default_light) wins; otherwise
            // fall back to a white directional light aimed roughly toward the
            // default Camera's view direction (-Z), with a small Y tilt so a
            // front-facing quad gets visible shading without becoming
            // perfectly flat. The -Z hit is what matters: a glTF mesh
            // imported with no lights and rendered through the default
            // Scene+Camera setup must read as lit, not silhouetted.
            let light = self
                .inner
                .default_light
                .read()
                .clone()
                .unwrap_or_else(|| Light::directional([0.0, -0.3, -1.0], [1.0, 1.0, 1.0]));
            let _ = pass.add(&light);
            self.inner.lights.write().push(light);
            *self.inner.has_light.write() = true;
        }
    }
}

impl Renderable for Scene {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        self.ensure_render_defaults();
        let mut all: Vec<Arc<PassObject>> = Vec::new();
        for pass in self.inner.passes.read().iter() {
            all.extend(pass.passes().iter().cloned());
        }
        all.into()
    }

    fn roots(&self) -> Arc<[Arc<PassObject>]> {
        self.inner
            .passes
            .read()
            .iter()
            .map(|pass| pass.object.clone())
            .collect::<Vec<_>>()
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::Vertex;
    use crate::{Material, Mesh, Model, Renderer, Target};

    fn pbr_triangle_mesh() -> Mesh {
        let mesh = Mesh::new();
        for (p, uv) in [
            ([0.0, 0.5, 0.0], [0.5, 1.0]),
            ([-0.5, -0.5, 0.0], [0.0, 0.0]),
            ([0.5, -0.5, 0.0], [1.0, 0.0]),
        ] {
            mesh.add_vertex(Vertex::pbr(p).set(Vertex::UV0, uv));
        }
        mesh
    }

    #[test]
    fn new_starts_empty() {
        let scene = Scene::new();
        // No passes exist until something gets added — stays cheap when the
        // user wires up the scene from a config and only later attaches
        // anything.
        assert!(scene.inner.passes.read().is_empty());
        assert!(scene.inner.absorb_pass.read().is_none());
        // Nothing rendered, no defaults injected — passes() on an empty scene
        // returns an empty list.
        assert!(scene.passes().is_empty());
    }

    #[test]
    fn add_creates_absorb_pass_lazily() {
        let scene = Scene::new();
        let model = Model::new(pbr_triangle_mesh(), Material::pbr());
        scene.add(&model).expect("add");
        assert!(scene.inner.absorb_pass.read().is_some());
        assert_eq!(scene.inner.passes.read().len(), 1);
    }

    #[test]
    fn add_pass_appends_to_graph() {
        let scene = Scene::new();
        let backdrop = Pass::new("backdrop");
        scene.add_pass(&backdrop);
        scene.add_pass(&Pass::new("shadow"));
        assert_eq!(scene.inner.passes.read().len(), 2);
    }

    #[test]
    fn passes_render_in_insertion_order() {
        let scene = Scene::new();
        // add_pass first, then a Model — the absorb Pass is appended after
        // the backdrop, so the render order follows insertion order.
        let backdrop = Pass::new("backdrop");
        scene.add_pass(&backdrop);
        let model = Model::new(pbr_triangle_mesh(), Material::pbr());
        scene.add(&model).expect("add");

        let list = scene.passes();
        assert!(list.len() >= 2, "got {} passes", list.len());
        // Backdrop was inserted first, so it renders first.
        assert_eq!(list[0].name.as_ref(), "backdrop");
        // The absorb Pass is named "Scene Default Pass".
        assert!(
            list.iter().any(|p| p.name.as_ref() == "Scene Default Pass"),
            "expected absorb pass in {:?}",
            list.iter().map(|p| p.name.as_ref()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn add_after_add_pass_follows_call_order() {
        // The absorb Pass is no longer privileged: a Pass added after the
        // geometry renders after it, unlike the old "extras always first"
        // behaviour.
        let scene = Scene::new();
        let model = Model::new(pbr_triangle_mesh(), Material::pbr());
        scene.add(&model).expect("add"); // absorb pass first
        let overlay = Pass::new("overlay");
        scene.add_pass(&overlay); // overlay second

        let list = scene.passes();
        assert_eq!(
            list.last().map(|p| p.name.as_ref()),
            Some("overlay"),
            "overlay added last should render last, in {:?}",
            list.iter().map(|p| p.name.as_ref()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn defaults_inject_camera_and_light_when_missing() {
        let scene = Scene::new();
        let material = Material::pbr();
        let model = Model::new(pbr_triangle_mesh(), material.clone());
        scene.add(&model).expect("add");

        // Trigger the default-fallback path — `passes()` is what the renderer
        // calls.
        let _ = scene.passes();

        // The default Camera + Light should have written their state into the
        // Material's shader; the camera position is the canonical [0, 0, 5].
        let pos: [f32; 3] = material
            .shader()
            .get("camera.position")
            .expect("camera.position");
        assert_eq!(pos, [0.0, 0.0, 5.0]);
        let light_color: [f32; 3] = material
            .shader()
            .get("lights.lights[0].color")
            .expect("lights.lights[0].color");
        assert_eq!(light_color, [1.0, 1.0, 1.0]);
        // Sticky flags now true — second pass through is a no-op.
        assert!(*scene.inner.has_camera.read());
        assert!(*scene.inner.has_light.read());
    }

    #[test]
    fn user_camera_skips_default_injection() {
        let scene = Scene::new();
        let material = Material::pbr();
        let model = Model::new(pbr_triangle_mesh(), material.clone());
        scene.add(&model).expect("add");
        // User-supplied camera at [3, 0, 0]. After this `add`, has_camera is
        // sticky-true and the default-Camera at [0, 0, 5] should never appear.
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).look_at(
            [3.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        scene.add(&camera).expect("camera");

        let _ = scene.passes();
        let pos: [f32; 3] = material.shader().get("camera.position").unwrap();
        assert_eq!(pos, [3.0, 0.0, 0.0]);
    }

    #[test]
    fn user_light_skips_default_injection() {
        let scene = Scene::new();
        let material = Material::pbr();
        let model = Model::new(pbr_triangle_mesh(), material.clone());
        scene.add(&model).expect("add");
        let light = Light::directional([0.0, -1.0, 0.0], [0.5, 0.0, 0.0]);
        scene.add(&light).expect("light");

        let _ = scene.passes();
        let color: [f32; 3] = material.shader().get("lights.lights[0].color").unwrap();
        assert_eq!(color, [0.5, 0.0, 0.0]);
    }

    #[test]
    fn clone_shares_state() {
        // Scene is shallow-Clone (Arc-share). Mutations through the cloned
        // handle are visible through the original.
        let scene = Scene::new();
        let alias = scene.clone();
        alias
            .add(&Model::new(pbr_triangle_mesh(), Material::pbr()))
            .expect("add via alias");
        assert!(scene.inner.absorb_pass.read().is_some());
    }

    #[test]
    fn typed_getters_surface_added_objects() {
        // Each `Scene::add` call routes through the typed-lane bookkeeping;
        // the three getters return Arc-shared clones of what was added.
        let scene = Scene::new();
        let model = Model::new(pbr_triangle_mesh(), Material::pbr());
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).look_at(
            [1.0, 2.0, 3.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        let light = Light::directional([0.0, -1.0, 0.0], [1.0, 0.95, 0.9]);

        scene.add(&model).expect("model");
        scene.add(&camera).expect("camera");
        scene.add(&light).expect("light");

        assert_eq!(scene.models().len(), 1);
        assert_eq!(scene.cameras().len(), 1);
        assert_eq!(scene.lights().len(), 1);
        // Live-handle check: mutating the returned Camera propagates to
        // the Model's shader, same as mutating the original handle would.
        let cam_back = scene.cameras().into_iter().next().unwrap();
        cam_back.look_at([9.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let p: [f32; 3] = model
            .material()
            .shader()
            .get("camera.position")
            .expect("camera.position");
        assert_eq!(p, [9.0, 0.0, 0.0]);
    }

    #[test]
    fn typed_getters_capture_defaults_after_render() {
        // The default-injection path bypasses `Scene::add` and goes
        // straight to `pass.add(...)` — the typed lanes must still pick
        // those up so consumers can grab the default camera/light back.
        let scene = Scene::new();
        let model = Model::new(pbr_triangle_mesh(), Material::pbr());
        scene.add(&model).expect("model");
        assert!(scene.cameras().is_empty(), "no camera before passes()");
        assert!(scene.lights().is_empty(), "no light before passes()");
        let _ = scene.passes();
        assert_eq!(scene.cameras().len(), 1, "default camera surfaces");
        assert_eq!(scene.lights().len(), 1, "default light surfaces");
        // Defaults are real handles — drive the default camera per frame:
        let default_camera = scene.cameras().into_iter().next().unwrap();
        default_camera.look_at([7.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let p: [f32; 3] = model.material().shader().get("camera.position").unwrap();
        assert_eq!(p, [7.0, 0.0, 0.0]);
    }

    #[test]
    fn typed_getters_preserve_insertion_order() {
        // Order matters for callers iterating "the second camera" or
        // pairing models with their loader-order glTF nodes.
        let scene = Scene::new();
        let first = Light::directional([0.0, -1.0, 0.0], [1.0, 0.0, 0.0]);
        let second = Light::point([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        scene.add(&first).expect("first");
        scene.add(&second).expect("second");
        let lights = scene.lights();
        assert_eq!(lights[0].color(), [1.0, 0.0, 0.0]);
        assert_eq!(lights[1].color(), [0.0, 1.0, 0.0]);
    }

    #[test]
    fn renders_through_renderer_end_to_end() {
        // Smoke test: the canonical "scene with model + default fallbacks"
        // round-trips through the full render path and produces an RGBA image
        // matching the target dimensions.
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([64u32, 64u32])
                .await
                .expect("texture target");
            let model = Model::new(
                pbr_triangle_mesh(),
                Material::pbr().base_color([0.6, 0.2, 0.8, 1.0]),
            );
            let scene = Scene::new();
            scene.add(&model).expect("add model");
            renderer.render(&scene, &target).expect("render scene");
            let image = target.get_image().await;
            assert_eq!(image.len(), 64 * 64 * 4);
        });
    }

    #[test]
    fn list_and_get_pass_expose_the_graph() {
        let scene = Scene::new();
        let a = Pass::new("a");
        let b = Pass::new("b");
        scene.add_pass(&a).add_pass(&b);

        let list = scene.list_passes();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].object.name.as_ref(), "a");
        assert_eq!(list[1].object.name.as_ref(), "b");

        assert_eq!(
            scene.get_pass(1).map(|p| p.object.name.to_string()),
            Some("b".to_string())
        );
        assert!(scene.get_pass(2).is_none(), "out-of-range returns None");
    }

    #[test]
    fn remove_pass_takes_a_pass_out_of_the_graph() {
        let scene = Scene::new();
        let keep = Pass::new("keep");
        let drop = Pass::new("drop");
        scene.add_pass(&keep).add_pass(&drop);

        assert!(
            scene.remove_pass(&drop),
            "removing a present pass returns true"
        );
        let names: Vec<String> = scene
            .list_passes()
            .iter()
            .map(|p| p.object.name.to_string())
            .collect();
        assert_eq!(names, vec!["keep".to_string()]);

        // Removing one that's already gone returns false.
        assert!(!scene.remove_pass(&drop));
    }

    #[test]
    fn remove_pass_forgets_the_absorb_handle() {
        // After removing the absorb Pass, the next `add` must rebuild one
        // instead of attaching to a Pass the graph no longer holds.
        let scene = Scene::new();
        scene
            .add(&Model::new(pbr_triangle_mesh(), Material::pbr()))
            .expect("add");
        let absorb = scene.list_passes()[0].clone();
        assert!(scene.remove_pass(&absorb));
        assert!(scene.inner.absorb_pass.read().is_none());

        scene
            .add(&Model::new(pbr_triangle_mesh(), Material::pbr()))
            .expect("add again");
        assert_eq!(scene.list_passes().len(), 1, "a fresh absorb pass appears");
    }

    #[test]
    fn set_passes_replaces_the_whole_graph() {
        let scene = Scene::new();
        scene.add_pass(&Pass::new("old"));
        scene.set_passes(vec![Pass::new("x"), Pass::new("y"), Pass::new("z")]);
        let names: Vec<String> = scene
            .list_passes()
            .iter()
            .map(|p| p.object.name.to_string())
            .collect();
        assert_eq!(
            names,
            vec!["x".to_string(), "y".to_string(), "z".to_string()]
        );
    }

    #[test]
    fn set_passes_drops_a_stale_absorb_handle() {
        let scene = Scene::new();
        scene
            .add(&Model::new(pbr_triangle_mesh(), Material::pbr()))
            .expect("add");
        assert!(scene.inner.absorb_pass.read().is_some());
        // Replace the graph with passes that don't include the absorb pass.
        scene.set_passes(vec![Pass::new("fresh")]);
        assert!(scene.inner.absorb_pass.read().is_none());
    }

    #[test]
    fn set_passes_keeps_a_surviving_absorb_handle() {
        let scene = Scene::new();
        scene
            .add(&Model::new(pbr_triangle_mesh(), Material::pbr()))
            .expect("add");
        let absorb = scene.list_passes()[0].clone();
        // Reorder, keeping the absorb pass in the graph.
        scene.set_passes(vec![Pass::new("before"), absorb]);
        assert!(
            scene.inner.absorb_pass.read().is_some(),
            "absorb handle survives a reorder that keeps it"
        );
    }

    #[test]
    fn no_defaults_skips_camera_and_light_injection() {
        let scene = Scene::new();
        let material = Material::pbr();
        let model = Model::new(pbr_triangle_mesh(), material.clone());
        scene.add(&model).expect("add");
        scene.no_defaults();

        let _ = scene.passes();
        // Neither default was injected, so the typed lanes stay empty and the
        // shader never received a camera position.
        assert!(scene.cameras().is_empty());
        assert!(scene.lights().is_empty());
        assert!(
            material
                .shader()
                .get::<[f32; 3]>("camera.position")
                .is_err(),
            "no default camera should have written camera.position"
        );
    }

    #[test]
    fn no_default_camera_keeps_default_light() {
        let scene = Scene::new();
        let model = Model::new(pbr_triangle_mesh(), Material::pbr());
        scene.add(&model).expect("add");
        scene.no_default_camera();

        let _ = scene.passes();
        assert!(scene.cameras().is_empty(), "camera injection is off");
        assert_eq!(scene.lights().len(), 1, "light injection still fires");
    }

    #[test]
    fn no_default_light_keeps_default_camera() {
        let scene = Scene::new();
        let model = Model::new(pbr_triangle_mesh(), Material::pbr());
        scene.add(&model).expect("add");
        scene.no_default_light();

        let _ = scene.passes();
        assert_eq!(scene.cameras().len(), 1, "camera injection still fires");
        assert!(scene.lights().is_empty(), "light injection is off");
    }

    #[test]
    fn set_default_camera_injects_the_override() {
        let scene = Scene::new();
        let material = Material::pbr();
        let model = Model::new(pbr_triangle_mesh(), material.clone());
        scene.add(&model).expect("add");

        let custom = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).look_at(
            [4.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        scene.set_default_camera(&custom);

        let _ = scene.passes();
        let pos: [f32; 3] = material
            .shader()
            .get("camera.position")
            .expect("camera.position");
        assert_eq!(pos, [4.0, 0.0, 0.0], "the override camera was injected");
    }

    #[test]
    fn set_default_light_injects_the_override() {
        let scene = Scene::new();
        let material = Material::pbr();
        let model = Model::new(pbr_triangle_mesh(), material.clone());
        scene.add(&model).expect("add");

        let custom = Light::directional([0.0, -1.0, 0.0], [0.2, 0.4, 0.6]);
        scene.set_default_light(&custom);

        let _ = scene.passes();
        let color: [f32; 3] = material
            .shader()
            .get("lights.lights[0].color")
            .expect("light color");
        assert_eq!(color, [0.2, 0.4, 0.6], "the override light was injected");
    }

    #[test]
    fn set_default_camera_rearms_after_no_default_camera() {
        // no_default_camera turns injection off; a later set_default_camera
        // is an explicit request, so it re-arms injection.
        let scene = Scene::new();
        let material = Material::pbr();
        let model = Model::new(pbr_triangle_mesh(), material.clone());
        scene.add(&model).expect("add");

        scene.no_default_camera();
        let custom = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).look_at(
            [0.0, 6.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, -1.0],
        );
        scene.set_default_camera(&custom);

        let _ = scene.passes();
        let pos: [f32; 3] = material.shader().get("camera.position").unwrap();
        assert_eq!(pos, [0.0, 6.0, 0.0]);
    }
}
