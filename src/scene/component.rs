//! [`Component`] — anything a [`Material`](crate::Material) can absorb to
//! provide ambient shader state. Camera and Light are the in-tree examples.
//!
//! A Component holds its own Arc-shared backing, so a single `Camera` /
//! `Light` value can be passed to `material.add(&camera)` and then mutated
//! later (`camera.look_at(...)`, `light.set_direction(...)`) — every Material
//! the component was added to picks the new values up automatically.

use crate::Shader;

/// Implemented by types whose state lives in a shader's uniform surface and
/// that want live propagation when mutated. `material.add(&component)` calls
/// [`Component::apply`] to seed the shader's uniforms and register the
/// component for future updates.
///
/// Implementors maintain a list of `Weak<ShaderObject>` references inside
/// their own Arc-shared backing; setter methods iterate the list and call
/// `Shader::set` for each entry so subsequent mutations reach every Material
/// that consumed the component.
pub trait Component {
    /// Write the component's current state into `shader` and register the
    /// shader for future propagation. Idempotent across repeated calls on
    /// the same shader: the same `Arc<ShaderObject>` may be registered more
    /// than once, but the writes are best-effort `Shader::set` calls so
    /// duplicates are harmless.
    fn apply(&self, shader: &Shader);
}
