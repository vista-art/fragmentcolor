/// A macro that implements the HasNodeId trait for the given type
pub(crate) mod macros;
/// A Node represents a spatial position in the Scene tree.
///
/// Each Node contains a parent NodeId and a Transform matrix
/// that represents its position, rotation and scale in the Scene.
///
/// Nodes are set to the root of the Scene tree by default. This
/// means their parent NodeId is set to zero, and their Transform
/// matrix will be relative to the Scene's origin.
pub mod node;

pub mod object;
pub mod scene;

pub use object::*;
pub use scene::*;
