//! Developer Guides
//!
//! This module exists so that guide pages under docs/guides/*.md can be included as
//! module-level documentation and their Rust code blocks are executed as doctests
//! by `cargo test`.
//!
//! Build notes (to be wired later):
//! - The website build can still use docs/guides/*.md as source and augment it with
//!   language tabs (Python/JS) from code regions found in healthcheck scripts or examples.
//! - Rust doctests will only execute the Rust code blocks contained in these files.

#[doc = include_str!("../docs/guides/materials.md")]
pub mod materials {}

#[doc = include_str!("../docs/guides/hello-triangle.md")]
pub mod hello_triangle {}

#[doc = include_str!("../docs/guides/textures-and-sampling.md")]
pub mod textures_and_sampling {}

#[doc = include_str!("../docs/guides/instancing-basics.md")]
pub mod instancing_basics {}

#[doc = include_str!("../docs/guides/uniforms-and-push-constants.md")]
pub mod uniforms_and_push_constants {}

#[doc = include_str!("../docs/guides/storage-buffers-1m-particles.md")]
pub mod storage_buffers_1m_particles {}

#[doc = include_str!("../docs/guides/compute-driven-sim.md")]
pub mod compute_driven_sim {}
