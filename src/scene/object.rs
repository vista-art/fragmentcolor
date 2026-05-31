//! [`SceneObject`] — a value that can be attached to a [`Pass`](crate::Pass).
//! [`Model`](crate::Model), [`Camera`](crate::Camera), and
//! [`Light`](crate::Light) all implement it; together they cover the
//! three shapes any 3D scene needs (geometry, viewpoint, illumination).
//! The trait mirrors the glTF / USD node model — every "thing that lives
//! in a scene" is a node attached to its parent.
//!
//! Implementors are responsible for whatever bookkeeping makes them part of
//! the pass — registering themselves in the relevant storage list, applying
//! initial uniform state to existing shaders, etc. — and may also opt into
//! a hook the pass calls when a *new* shader joins, so they can re-apply.
//! Camera and Light use this hook for live propagation; Model doesn't
//! need it (its work is fully done at attach time).

use crate::Shader;

/// A node that can be attached to a [`Pass`](crate::Pass). Implementations
/// drive their own state changes on the pass through
/// [`SceneObject::attach`]; types that want live propagation when *new*
/// shaders join the pass after attachment override
/// [`SceneObject::apply_to_shader`].
///
/// The trait requires `Send + Sync` on every target except wasm32. wgpu's
/// Web backend uses `Rc`/`RefCell` for browser resources (see
/// `wgpu::backend::webgpu::WebBuffer`), so the GPU handles transitively
/// owned by Mesh, Camera and Light are `!Send + !Sync` on wasm. The web
/// runtime is single-threaded. `render()` never crosses a thread boundary
/// there, so dropping the bound is sound.
#[cfg(not(wasm))]
pub trait SceneObject: std::fmt::Debug + Send + Sync + 'static {
    /// Attach to a Pass. The implementation owns the bookkeeping:
    /// registering on the appropriate storage list, applying initial state
    /// to existing shaders, validating against the pass, anything else.
    fn attach(&self, pass: &crate::Pass) -> Result<(), crate::PassError>;

    /// Hook the pass invokes when a new shader joins after attachment.
    /// Default is a no-op; Camera and the light types override it to write
    /// their current state into the new shader so order of attachment
    /// doesn't matter. Model doesn't need this. Its work is one-shot at
    /// attach.
    fn apply_to_shader(&self, shader: &Shader) {
        let _ = shader;
    }
}

/// wasm32 variant: same contract, no `Send + Sync` bound. See the
/// non-wasm doc for rationale.
#[cfg(wasm)]
pub trait SceneObject: std::fmt::Debug + 'static {
    /// Attach to a Pass. See the non-wasm definition for the contract.
    fn attach(&self, pass: &crate::Pass) -> Result<(), crate::PassError>;

    /// Hook the pass invokes when a new shader joins after attachment.
    /// See the non-wasm definition for the contract.
    fn apply_to_shader(&self, shader: &Shader) {
        let _ = shader;
    }
}
