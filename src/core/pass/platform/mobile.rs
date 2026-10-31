//! Mobile (Swift / Kotlin) uniffi bindings for `Pass`.
//!
//! Uniffi cannot marshal generic parameters (`impl Into<T>`, `<R: Renderable>`,
//! `<T: TryInto<...>>`) over the FFI boundary, so each method gets a concrete
//! mobile entry point. The naming convention mirrors the other platform modules:
//! mobile-specific methods carry a `_mobile` suffix on the Rust side so the
//! build-time doc scanner keeps them separate from the Rust-only API, and every
//! uniffi export carries an explicit `name = "..."` attribute to expose
//! idiomatic camelCase names in Swift and Kotlin.
//!
//! Hidden per-language docs under `docs/api/core/pass/hidden/<method>_mobile.md`
//! satisfy the build-time documentation validator.

use std::sync::Arc;

use lsp_doc::lsp_doc;

use crate::Pass;
use crate::PassError;
use crate::bindings::numbers::read_color;
use crate::error::platform::mobile::FragmentColorError;
use crate::{Mesh, ScreenRegion, Shader, Texture};
use crate::{RenderableHandle, SceneObjectHandle, TargetHandle};

// -----------------------------------------------------------------
// Pass (uniffi bindings)
// -----------------------------------------------------------------

#[uniffi::export]
impl Pass {
    /// Create a new render `Pass` with the given name.
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/core/pass/new.md")]
    pub fn new_mobile(name: String) -> Arc<Self> {
        Arc::new(Self::new(&name))
    }

    /// Create a new compute `Pass` with the given name.
    #[uniffi::constructor(name = "compute")]
    #[lsp_doc("docs/api/core/pass/compute.md")]
    pub fn compute_mobile(name: String) -> Arc<Self> {
        Arc::new(Self::compute(&name))
    }

    /// Create a `Pass` pre-configured for the given `Shader`.
    #[uniffi::constructor(name = "fromShader")]
    #[lsp_doc("docs/api/core/pass/hidden/from_shader_mobile.md")]
    pub fn from_shader_mobile(name: String, shader: Arc<Shader>) -> Arc<Self> {
        Arc::new(Self::from_shader(&name, &shader))
    }

    /// Attach a shader to this pass.
    #[uniffi::method(name = "addShader")]
    #[lsp_doc("docs/api/core/pass/add_shader.md")]
    pub fn add_shader_mobile(&self, shader: Arc<Shader>) -> Result<(), FragmentColorError> {
        Ok(self.add_shader(&shader)?)
    }

    /// Add a mesh to the last compatible shader in this pass.
    #[uniffi::method(name = "addMesh")]
    #[lsp_doc("docs/api/core/pass/add_mesh.md")]
    pub fn add_mesh_mobile(&self, mesh: Arc<Mesh>) -> Result<(), FragmentColorError> {
        self.add_mesh(&mesh)
            .map_err(|e: PassError| FragmentColorError::Render(e.to_string()))
    }

    /// Unified `Pass.add` — branches on the runtime mobile handle. Adding
    /// a new `SceneObject` Rust-side means adding one extra variant to
    /// [`SceneObjectHandle`](crate::SceneObjectHandle) and one arm here.
    /// Returns the pass, as Rust's `add` does, so calls chain.
    #[uniffi::method(name = "add")]
    #[lsp_doc("docs/api/core/pass/add.md")]
    pub fn add_mobile(
        self: Arc<Self>,
        object: SceneObjectHandle,
    ) -> Result<Arc<Self>, FragmentColorError> {
        match &object {
            SceneObjectHandle::Model(model) => self.add(model.as_ref())?,
            SceneObjectHandle::Camera(camera) => self.add(camera.as_ref())?,
            SceneObjectHandle::Light(light) => self.add(light.as_ref())?,
        };
        Ok(self)
    }

    /// Set the render target (color attachment) for this pass.
    ///
    /// The `target` must be a `TargetHandle::Texture` variant; window targets
    /// cannot be used as a render-to-texture color target.
    #[uniffi::method(name = "setTarget")]
    #[lsp_doc("docs/api/core/pass/hidden/set_target_mobile.md")]
    pub fn set_target_mobile(&self, target: TargetHandle) -> Result<(), FragmentColorError> {
        match target {
            TargetHandle::Texture(mt) => {
                let tex_target = mt.texture_target();
                self.set_target(&tex_target)
                    .map_err(|e: PassError| FragmentColorError::Render(e.to_string()))
            }
            TargetHandle::Window(_) => Err(FragmentColorError::Render(
                "setTarget: WindowTarget cannot be used as a Pass color target; use TextureTarget instead".into(),
            )),
        }
    }

    /// Set a `Texture` as the color attachment.
    /// Swift and Kotlin expose it as `setTarget(texture)`.
    #[uniffi::method(name = "setTargetTexture")]
    #[lsp_doc("docs/api/core/pass/hidden/set_target_texture_mobile.md")]
    pub fn set_target_texture_mobile(
        &self,
        texture: Arc<Texture>,
    ) -> Result<(), FragmentColorError> {
        Ok(self.set_target(texture.as_ref())?)
    }

    /// Set the depth attachment for this pass.
    ///
    /// The `target` must be a `TargetHandle::Texture` variant wrapping a
    /// depth-format texture (e.g. `Depth32Float`).
    #[uniffi::method(name = "setDepthTarget")]
    #[lsp_doc("docs/api/core/pass/hidden/set_depth_target_mobile.md")]
    pub fn set_depth_target_mobile(&self, target: TargetHandle) -> Result<(), FragmentColorError> {
        match target {
            TargetHandle::Texture(mt) => {
                let tex_target = mt.texture_target();
                self.set_depth_target(&tex_target)
                    .map_err(|e: PassError| FragmentColorError::Render(e.to_string()))
            }
            TargetHandle::Window(_) => Err(FragmentColorError::Render(
                "setDepthTarget: WindowTarget cannot be used as a depth attachment; use a depth Texture instead".into(),
            )),
        }
    }

    /// Set a depth `Texture` from `Renderer.createDepthTexture` as the
    /// depth attachment. Swift and Kotlin expose it as `setDepthTarget(texture)`.
    #[uniffi::method(name = "setDepthTexture")]
    #[lsp_doc("docs/api/core/pass/hidden/set_depth_texture_mobile.md")]
    pub fn set_depth_texture_mobile(
        &self,
        texture: Arc<Texture>,
    ) -> Result<(), FragmentColorError> {
        self.set_depth_target(texture.as_ref())
            .map_err(|e: PassError| FragmentColorError::Render(e.to_string()))
    }

    /// Set the clear color for this pass as linear `[r, g, b]` or `[r, g, b, a]`.
    ///
    /// Switching from load to clear mode: after calling this the pass will
    /// clear its color attachment to `color` at the start of each frame.
    #[uniffi::method(name = "setClearColor")]
    #[lsp_doc("docs/api/core/pass/hidden/set_clear_color_mobile.md")]
    pub fn set_clear_color_mobile(&self, color: Vec<f32>) -> Result<(), FragmentColorError> {
        Ok(self.set_clear_color(read_color(color, "Pass.setClearColor")?)?)
    }

    /// Set the viewport (scissor + render region) for this pass.
    #[uniffi::method(name = "setViewport")]
    #[lsp_doc("docs/api/core/pass/set_viewport.md")]
    pub fn set_viewport_mobile(&self, region: ScreenRegion) -> Result<(), FragmentColorError> {
        Ok(self.set_viewport(region)?)
    }

    /// Set the compute dispatch dimensions for this pass.
    /// A 0 in any dimension dispatches nothing.
    #[uniffi::method(name = "setComputeDispatch")]
    #[lsp_doc("docs/api/core/pass/set_compute_dispatch.md")]
    pub fn set_compute_dispatch_mobile(&self, x: u32, y: u32, z: u32) {
        self.set_compute_dispatch(x, y, z);
    }

    /// Declare DAG dependencies for this pass.
    ///
    /// `deps` is a list of `RenderableHandle` values (Shader, Pass, or Mesh
    /// variants). The pass will be executed only after all listed dependencies
    /// have completed. Returns an error if a self-reference or cycle is
    /// detected, or if the same dependency appears twice.
    #[uniffi::method(name = "require")]
    #[lsp_doc("docs/api/core/pass/hidden/require_mobile.md")]
    pub fn require_mobile(&self, deps: Vec<RenderableHandle>) -> Result<(), FragmentColorError> {
        for dep in deps {
            self.require(&dep)
                .map_err(|e: PassError| FragmentColorError::Render(e.to_string()))?;
        }
        Ok(())
    }

    /// Return `true` if this is a compute pass; `false` for a render pass.
    #[uniffi::method(name = "isCompute")]
    #[lsp_doc("docs/api/core/pass/is_compute.md")]
    pub fn is_compute_mobile(&self) -> bool {
        self.is_compute()
    }

    /// Switch this pass to load mode: the previous frame's contents are
    /// preserved rather than cleared at the start of the frame.
    #[uniffi::method(name = "loadPrevious")]
    #[lsp_doc("docs/api/core/pass/load_previous.md")]
    pub fn load_previous_mobile(&self) {
        self.load_previous();
    }

    #[uniffi::method(name = "name")]
    #[lsp_doc("docs/api/core/pass/name.md")]
    pub fn name_mobile(&self) -> String {
        self.name()
    }
}
