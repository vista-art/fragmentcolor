#![allow(unused_imports)]

//! Asset loading and management module.
//!
//! This module is a Work In Progress.
//! Most of its features are behind the `assets` feature flag.
//!
//! There is no public interface yet.

// @TODO: Create public interface for asset loading and management.

#[cfg(feature = "assets")]
mod gltf;
#[cfg(feature = "assets")]
mod obj;

#[cfg(feature = "assets")]
pub use self::gltf::load_gltf;
#[cfg(feature = "assets")]
pub use self::obj::load_obj;
