//! [`Component`] — a scene-level value whose state lives in a shader's
//! uniform surface. Camera and Light are the in-tree examples; any type that
//! wants to inject a fixed set of uniforms can implement it.
//!
//! A Component holds its own Arc-shared backing, so a single `Camera` /
//! `Light` value can be passed to [`Pass::add`](crate::Pass::add) and then
//! mutated later (`camera.look_at(...)`, `light.set_direction(...)`) —
//! every Pass the component was added to picks the new values up
//! automatically, and so does every Model added to such a Pass afterwards.

use crate::Shader;

/// Implemented by types whose state lives in a shader's uniform surface and
/// that want live propagation when mutated. `pass.add(&component)` calls
/// [`Component::apply`] for every shader currently in the pass, then stores
/// the component so the same call fires on every later
/// [`Pass::add_model`](crate::Pass::add_model).
///
/// Implementors typically maintain a list of `Weak<ShaderObject>`
/// references inside their own Arc-shared backing; setter methods iterate
/// the list and call `Shader::set` for each entry so subsequent mutations
/// reach every shader the component was applied to.
pub trait Component: std::fmt::Debug + Send + Sync + 'static {
    /// Write the component's current state into `shader` and register the
    /// shader for future propagation. Idempotent across repeated calls on
    /// the same shader: the same `Arc<ShaderObject>` may be registered more
    /// than once, but the writes are best-effort `Shader::set` calls so
    /// duplicates are harmless.
    fn apply(&self, shader: &Shader);
}
