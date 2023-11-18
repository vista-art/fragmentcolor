//! Scene Graph Module
//!
//! This is the module that most users will interact with.
//!
//! This module manages the spatial positions of every object in the Scene
//! and their hierarchical relationships to each other.

/// A Node represents a spatial position in the Scene tree.
///
/// Each Node contains a parent NodeId and a Transform matrix
/// that represents its position, rotation and scale in the Scene.
///
/// Nodes are set to the root of the Scene tree by default. This
/// means their parent NodeId is set to zero, and their Transform
/// matrix will be relative to the Scene's origin.
pub mod node;

/// A SceneObject is associated with a Node in the Scene and contains
/// a list of Components that define its behavior and appearance.
pub mod object;

/// A Scene is a collection of Nodes and SceneObjects.
pub mod scene;

/// Internal code-generation macro that implements
/// the SpatialObject trait for the given type
pub(crate) mod macros;

pub use object::*;
pub use scene::*;
