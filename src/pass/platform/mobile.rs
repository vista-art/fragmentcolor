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

use crate::pass::error::PassError;
use crate::pass::{Pass, PassInput};
use crate::renderer::platform::mobile::FragmentColorError;
use crate::renderer::renderable::{RenderableHandle, SceneObjectHandle, TargetHandle};
use crate::{Mesh, ScreenRegion, Shader};

// -----------------------------------------------------------------
// Mobile-only record types
// -----------------------------------------------------------------

/// Mobile-serializable mirror of `PassInput`.
///
/// `PassInput` stores a `Color(u32)` tuple-newtype which uniffi's `Record`
/// derive does not support. `MobilePassInput` exposes the same information
/// as named fields that uniffi can marshal.
#[derive(Debug, Clone, uniffi::Record)]
pub struct MobilePassInput {
    /// If `true`, the previous frame's contents are loaded (no clear).
    /// If `false`, the `color_rgba` value is used to clear the attachment.
    pub load: bool,
    /// Clear colour packed as 0xRRGGBBAA (same bit layout as `Color(u32)`).
    pub color_rgba: u32,
}

impl From<PassInput> for MobilePassInput {
    fn from(pi: PassInput) -> Self {
        Self {
            load: pi.load,
            color_rgba: pi.color.0,
        }
    }
}

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
    pub fn add_shader_mobile(&self, shader: Arc<Shader>) {
        self.object.add_shader(shader.object.clone());
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
    #[uniffi::method(name = "add")]
    #[lsp_doc("docs/api/core/pass/add.md")]
    pub fn add_mobile(&self, object: SceneObjectHandle) -> Result<(), FragmentColorError> {
        match object {
            SceneObjectHandle::Model(m) => self
                .add(m.as_ref())
                .map(|_| ())
                .map_err(|e: PassError| FragmentColorError::Render(e.to_string())),
            SceneObjectHandle::Camera(c) => self
                .add(c.as_ref())
                .map(|_| ())
                .map_err(|e: PassError| FragmentColorError::Render(e.to_string())),
            SceneObjectHandle::Light(l) => self
                .add(l.as_ref())
                .map(|_| ())
                .map_err(|e: PassError| FragmentColorError::Render(e.to_string())),
        }
    }

    /// Set the render target (color attachment) for this pass.
    ///
    /// The `target` must be a `TargetHandle::Texture` variant; window targets
    /// cannot be used as a render-to-texture colour target.
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
                "setTarget: WindowTarget cannot be used as a Pass colour target; use TextureTarget instead".into(),
            )),
        }
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

    /// Set the clear colour for this pass (as `[r, g, b, a]` in linear 0..1 space).
    ///
    /// Switching from load to clear mode: after calling this the pass will
    /// clear its colour attachment to `color` at the start of each frame.
    #[uniffi::method(name = "setClearColor")]
    #[lsp_doc("docs/api/core/pass/hidden/set_clear_color_mobile.md")]
    pub fn set_clear_color_mobile(&self, color: Vec<f32>) -> Result<(), FragmentColorError> {
        match color.len() {
            3 => {
                self.set_clear_color([color[0], color[1], color[2], 1.0]);
                Ok(())
            }
            4 => {
                self.set_clear_color([color[0], color[1], color[2], color[3]]);
                Ok(())
            }
            n => Err(FragmentColorError::Render(format!(
                "setClearColor: expected 3 or 4 components (r, g, b[, a]); got {}",
                n
            ))),
        }
    }

    /// Set the viewport (scissor + render region) for this pass.
    #[uniffi::method(name = "setViewport")]
    #[lsp_doc("docs/api/core/pass/set_viewport.md")]
    pub fn set_viewport_mobile(&self, region: ScreenRegion) {
        self.set_viewport(region);
    }

    /// Set the compute dispatch dimensions for this pass.
    /// All zero values are clamped to 1.
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

    /// Return the current input descriptor for this pass.
    ///
    /// The mobile binding returns a `MobilePassInput` record because
    /// `PassInput` contains a `Color(u32)` tuple-newtype that uniffi cannot
    /// serialize as a `Record`.
    #[uniffi::method(name = "getInput")]
    #[lsp_doc("docs/api/core/pass/hidden/get_input_mobile.md")]
    pub fn get_input_mobile(&self) -> MobilePassInput {
        self.get_input().into()
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
