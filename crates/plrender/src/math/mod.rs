/// Defines Linear Algebra types useful for Computer Graphics.
///
/// We abbreviate the most commonly used Linear Algebra types,
/// namely Vector (Vec) and Matix (Mat), but their full names
/// are also available.
///
/// It also exports the type Pixel, which is a `Point2<u16>`.
///
/// All matrices in PLRender are Column Major.
///
/// All types, except Pixel, are f32.
///
/// # Note
/// This library uses the `mint` crate as an interoperability
/// layer between other Linear Algebra libraries. It just
/// re-exports abbreviated `mint` math types, all as f32,
/// so users can call `Vec3 { x, y, z }` instead of
/// `mint::Vector3<f32> { x, y, z }`.
///
/// Public API objects can define their interface with
/// `<T: Into<TypeName>>` so end users can easily pass
/// any compatible primitive types like arrays.
///
/// # Examples
/// ```
///  use plrender::math::cg::Vec3;
///
///  struct SomeObject {
///     pub position: Vec3,
///  }
///
///  impl SomeObject {
///    pub fn new(position: impl Into<Vec3>) -> Self {
///       Self {
///         position: position.into(),
///       }
///    }
///  }
///
///   let object = SomeObject::new([1.0, 2.0, 3.0]);
///
///   assert_eq!(object.position, Vec3 { x: 1.0, y: 2.0, z: 3.0 });
/// ```
pub mod cg;

/// Defines Internal 2D and 3D Geometry primitives.
///
/// Users normally do not need to use this module directly.
///
/// It contains the following objects:
/// - Primitive
/// - ShapeBuilder
/// - Quad
/// - Vertex
///
/// # Primitive
/// The `Primitive` object contains a MeshBuilder and
/// can be used to create an ad-hoc primitive Mesh.
/// It currently supports `cube`, `cubeoid`, `plane`,
/// and `sphere`.
///
/// The `build_mesh()` method automatically uploads the
/// Primitive shape vertices to the GPU and returns a
/// BuiltMesh instance, which is an object containing
/// references to the GPU buffers and the number and
/// types of vertices and indices it contains.
///
/// The BuiltMesh object can be used to create a Mesh
/// Object. Users normally do not need to use it
/// directly, as there are wrapper ScenObjects that
/// abstract away the creation of a Mesh, like the
/// `Cube` and `Sphere` objects.
///
/// # ShapeBuilder
/// The ShapeBuilder object creates a 2D Primitive from
/// custom Paths. It uses `lyon` as a tesselator.
///
/// This is currently experimental and will back the
/// creation of custom 2D shapes in the future, as
/// well the SVG import.
///
/// # Quad
/// The Quad object is useful for manipulating and
/// comparing 2D surface regions like textures,
/// viewports, windows and images.
///
/// # Vertex
/// This is the internal Type the MeshBuilder uses
/// for uploading a Mesh to the GPU. It contains
/// the Position, Normal, and Color variants.
pub mod geometry;

pub use cg::*;
pub use geometry::*;
