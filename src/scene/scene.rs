//! Scene — top-level container for [`Model`](crate::Model),
//! [`Camera`](crate::Camera), [`Light`](crate::Light), and any user-defined
//! [`SceneObject`](crate::scene::SceneObject). Owns one or more
//! [`Pass`](crate::Pass) entries underneath and implements
//! [`Renderable`](crate::Renderable), so the whole scene goes to the
//! [`Renderer`](crate::Renderer) in a single call.
//!
//! `Scene::new()` is sync — no `Renderer` argument, no async, nothing to
//! await. The first time a SceneObject is added the Scene allocates a
//! default Pass to absorb it; the first time the Scene is rendered the
//! underlying GPU resources initialise on demand. Same lazy-init pattern
//! the rest of FragmentColor follows.
//!
//! When the user has added Models but no Camera or Light, the Scene injects
//! sensible defaults at render time so the "hello world" path renders
//! something recognisable. As soon as you add your own Camera / Light, the
//! defaults stop firing.

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
    /// Lazy default Pass — created on the first `Scene::add` so an empty
    /// Scene allocates no GPU bookkeeping at all.
    pub(crate) default_pass: RwLock<Option<Pass>>,
    /// Pre-passes added via `Scene::add_pass`. Rendered in insertion order
    /// *before* the default Pass.
    pub(crate) extra_passes: RwLock<Vec<Pass>>,
    /// Sticky once-set flags so the default-Camera / default-Light injection
    /// at `passes()` time only fires once and only when the user hasn't
    /// supplied their own.
    pub(crate) has_camera: RwLock<bool>,
    pub(crate) has_light: RwLock<bool>,
    /// Scene-wide ambient color (`lights.ambient` in the PBR shader). Set
    /// via `Scene::ambient`; cached so Models added afterwards inherit it.
    pub(crate) ambient: RwLock<Option<[f32; 3]>>,
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
                default_pass: RwLock::new(None),
                extra_passes: RwLock::new(Vec::new()),
                has_camera: RwLock::new(false),
                has_light: RwLock::new(false),
                ambient: RwLock::new(None),
            }),
        }
    }

    #[lsp_doc("docs/api/scene/scene/add.md")]
    pub fn add<O: SceneObject + 'static>(&self, object: &O) -> Result<&Self, PassError> {
        let pass = self.ensure_default_pass();
        pass.add(object)?;
        // Track Camera / Light presence so the render-time default-fallback
        // path knows whether to inject. TypeId equality is exact — wrapping
        // a Camera in a user newtype counts as "user-supplied", which is the
        // right behaviour: anything custom takes ownership of the role.
        let tid = std::any::TypeId::of::<O>();
        if tid == std::any::TypeId::of::<Camera>() {
            *self.inner.has_camera.write() = true;
        } else if tid == std::any::TypeId::of::<Light>() {
            *self.inner.has_light.write() = true;
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

    #[lsp_doc("docs/api/scene/scene/ambient.md")]
    pub fn ambient(&self, color: [f32; 3]) -> &Self {
        *self.inner.ambient.write() = Some(color);
        // Stamp onto every shader currently in the scene. Future Models
        // added via `Scene::add` pick the value up from the stash.
        for pass in self.inner.extra_passes.read().iter() {
            for shader in pass.object.shaders.read().iter() {
                let _ = shader.set("lights.ambient", color);
            }
        }
        if let Some(pass) = self.inner.default_pass.read().as_ref() {
            for shader in pass.object.shaders.read().iter() {
                let _ = shader.set("lights.ambient", color);
            }
        }
        self
    }

    #[lsp_doc("docs/api/scene/scene/add_pass.md")]
    pub fn add_pass(&self, pass: &Pass) -> &Self {
        self.inner.extra_passes.write().push(pass.clone());
        self
    }

    /// Lazily build the default Pass on first `add`. The Pass is named so it
    /// shows up identifiably in graphics debuggers (RenderDoc, Xcode GPU
    /// frame capture).
    fn ensure_default_pass(&self) -> Pass {
        if let Some(p) = self.inner.default_pass.read().clone() {
            return p;
        }
        let mut slot = self.inner.default_pass.write();
        // Re-check under the write lock in case a concurrent caller raced us.
        if let Some(p) = slot.clone() {
            return p;
        }
        let pass = Pass::new("Scene Default Pass");
        *slot = Some(pass.clone());
        pass
    }

    /// Inject default Camera / Light into the default Pass when the user
    /// hasn't supplied their own. Idempotent — the sticky `has_camera` /
    /// `has_light` flags flip on first injection, so subsequent `passes()`
    /// calls are no-ops on this front.
    fn ensure_render_defaults(&self) {
        let Some(pass) = self.inner.default_pass.read().clone() else {
            return;
        };
        let needs_camera = !*self.inner.has_camera.read();
        let needs_light = !*self.inner.has_light.read();
        if needs_camera {
            // 60° vertical FOV, square aspect, a comfortable [0.1, 100] depth
            // range. The eye sits five units back from the origin looking at
            // it with conventional +Y up. Fine for offscreen test targets and
            // for someone trying the API for the first time; users with a
            // non-square target supply their own Camera.
            let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0)
                .look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
            // Camera::attach always succeeds — discard the never-error result.
            let _ = pass.add(&camera);
            *self.inner.has_camera.write() = true;
        }
        if needs_light {
            // Slightly off-axis white directional light — gives diffuse
            // shading some visible falloff instead of perfectly flat.
            let light = Light::directional([0.3, -1.0, -0.4], [1.0, 1.0, 1.0]);
            let _ = pass.add(&light);
            *self.inner.has_light.write() = true;
        }
    }
}

impl Renderable for Scene {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        self.ensure_render_defaults();
        let mut all: Vec<Arc<PassObject>> = Vec::new();
        for pass in self.inner.extra_passes.read().iter() {
            all.extend(pass.passes().iter().cloned());
        }
        if let Some(pass) = self.inner.default_pass.read().as_ref() {
            all.extend(pass.passes().iter().cloned());
        }
        all.into()
    }

    fn roots(&self) -> Arc<[Arc<PassObject>]> {
        let mut roots: Vec<Arc<PassObject>> = Vec::new();
        for pass in self.inner.extra_passes.read().iter() {
            roots.push(pass.object.clone());
        }
        if let Some(pass) = self.inner.default_pass.read().as_ref() {
            roots.push(pass.object.clone());
        }
        roots.into()
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
            mesh.add_vertex(
                Vertex::new(p)
                    .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
                    .set(Vertex::UV0, uv).set(Vertex::COLOR0, [1.0, 1.0, 1.0, 1.0]).set(Vertex::UV1, [0.0, 0.0]).set(Vertex::TANGENT, [1.0, 0.0, 0.0, 1.0]),
            );
        }
        mesh
    }

    #[test]
    fn new_starts_empty() {
        let scene = Scene::new();
        // No default pass exists until something gets added — stays cheap when
        // the user wires up the scene from a config and only later attaches
        // anything.
        assert!(scene.inner.default_pass.read().is_none());
        assert!(scene.inner.extra_passes.read().is_empty());
        // Nothing rendered, no defaults injected — passes() on an empty scene
        // returns an empty list.
        assert!(scene.passes().is_empty());
    }

    #[test]
    fn add_creates_default_pass_lazily() {
        let scene = Scene::new();
        let model = Model::new(pbr_triangle_mesh(), Material::pbr().expect("pbr"));
        scene.add(&model).expect("add");
        assert!(scene.inner.default_pass.read().is_some());
    }

    #[test]
    fn add_pass_appends_to_extras() {
        let scene = Scene::new();
        let backdrop = Pass::new("backdrop");
        scene.add_pass(&backdrop);
        scene.add_pass(&Pass::new("shadow"));
        assert_eq!(scene.inner.extra_passes.read().len(), 2);
    }

    #[test]
    fn passes_lists_extras_then_default() {
        let scene = Scene::new();
        let backdrop = Pass::new("backdrop");
        scene.add_pass(&backdrop);
        let model = Model::new(pbr_triangle_mesh(), Material::pbr().expect("pbr"));
        scene.add(&model).expect("add");

        let list = scene.passes();
        assert!(list.len() >= 2, "got {} passes", list.len());
        // Backdrop should come before the default.
        assert_eq!(list[0].name.as_ref(), "backdrop");
        // Default Pass is named "Scene Default Pass".
        assert!(
            list.iter()
                .any(|p| p.name.as_ref() == "Scene Default Pass"),
            "expected default scene pass in {:?}",
            list.iter().map(|p| p.name.as_ref()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn defaults_inject_camera_and_light_when_missing() {
        let scene = Scene::new();
        let material = Material::pbr().expect("pbr");
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
        let material = Material::pbr().expect("pbr");
        let model = Model::new(pbr_triangle_mesh(), material.clone());
        scene.add(&model).expect("add");
        // User-supplied camera at [3, 0, 0]. After this `add`, has_camera is
        // sticky-true and the default-Camera at [0, 0, 5] should never appear.
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0)
            .look_at([3.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        scene.add(&camera).expect("camera");

        let _ = scene.passes();
        let pos: [f32; 3] = material.shader().get("camera.position").unwrap();
        assert_eq!(pos, [3.0, 0.0, 0.0]);
    }

    #[test]
    fn user_light_skips_default_injection() {
        let scene = Scene::new();
        let material = Material::pbr().expect("pbr");
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
            .add(&Model::new(pbr_triangle_mesh(), Material::pbr().expect("pbr")))
            .expect("add via alias");
        assert!(scene.inner.default_pass.read().is_some());
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
                Material::pbr()
                    .expect("pbr")
                    .base_color([0.6, 0.2, 0.8, 1.0]),
            );
            let scene = Scene::new();
            scene.add(&model).expect("add model");
            renderer.render(&scene, &target).expect("render scene");
            let image = target.get_image().await;
            assert_eq!(image.len(), 64 * 64 * 4);
        });
    }
}
