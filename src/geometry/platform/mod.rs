//! Binding shims for `Mesh`, `Vertex`, `Instance` and `Quad`.

#[cfg(mobile)]
pub mod mobile;
pub mod python;
pub mod web;
