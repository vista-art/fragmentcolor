//! Declares `Mesh`, `Vertex`, `Instance` and `Quad` and re-exports them.

mod error;
pub use error::*;

mod mesh;
pub use mesh::*;

mod vertex;
pub use vertex::*;

mod instance;
pub use instance::*;

mod vertex_data;
pub use vertex_data::VertexData;
pub(crate) use vertex_data::{Shape, describe};

mod quad;
pub use quad::*;

mod position;
pub(crate) use position::{POSITION, explain_position};

mod platform;
