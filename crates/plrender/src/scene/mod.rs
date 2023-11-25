//! Scene Graph Module
//!
//! This is the module that most users will interact with.
//!
//! This module manages the spatial positions of every object in the Scene
//! and their hierarchical relationships to each other.

/// A Transform represents a spatial position in the Scene tree.
///
/// Every Objects contain a Transform, which is a struct containing a
/// LocalTransform with raw data and a parent TransformId. This allows the
/// Scene to calculate the GlobalTransforms collection containing positions
/// and orientations of every object and upload them to the GPU..
///
/// Transforms are set to the root of the Scene tree by default. This
/// means their parent TransformId is set to zero, and their LocalTransform
/// matrix will be relative to the Scene's origin.
pub mod transform;

/// A Object is associated with a Transform in the Scene and contains
/// a list of Components that define its behavior and appearance.
pub mod object;

/// A Scene is a collection of Transforms and Objects.
pub mod scene;

/// Internal code-generation macro that implements
/// the SpatialObject trait for the given type
pub(crate) mod macros;

pub use object::*;
pub use scene::*;
