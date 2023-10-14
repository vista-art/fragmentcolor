pub use plrender::{
    window::Window, Camera, Color, Context, Entity, EntityRef, ImageRef, Light, LightBuilder,
    LightRef, MeshBuilder, MeshRef, Node, NodeRef, Pass, Projection, Prototype, Scene, Sprite,
    SpriteBuilder, TargetInfo, TargetRef, UvRange,
};
use pyo3::prelude::*;

// @FIXME code generation works partially.
// It's still unrealiable for production.
//
// Example usage:
// use plrender_macros::wrap_py;
// wrap_py!(Camera);
// wrap_py!(Color);
// wrap_py!(Context);
// wrap_py!(Scene);

// Context
#[pyclass]
pub struct PyContext {
    inner: plrender::Context,
}

#[pymethods]
impl PyContext {
    #[new]
    fn new() -> PyResult<Self> {
        let window = Window::new().title("PLRender").build();
        let context = pollster::block_on(plrender::Context::init().build(&window));
        Ok(PyContext { inner: context })
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.inner.resize(width, height);
    }
}

// - Solid
//   - new_offscreen(& SolidConfig, crate :: TargetInfo, & crate :: Context) -> Self
//   - new(& SolidConfig, & crate :: Context) -> Self

// - Context
//   - resize(u32, u32) -> None
//   - surface_info() -> Option < TargetInfo >
//   - get_image_info(ImageRef) -> ImageInfo
//   - init() -> ContextBuilder
//   - load_image(impl AsRef < Path >) -> ImageRef
//   - add_mesh() -> MeshBuilder
//   - add_image_from_texture(wgpu :: Texture, wgpu :: Extent3d) -> ImageRef
//   - add_image_from_bytes(& wgpu :: TextureDescriptor, & [u8]) -> ImageRef
//   - present(& mut P, & Scene, & Camera) -> None

// - PhongConfig

// - Phong
//   - new(& PhongConfig, & crate :: Context) -> Self
//   - new_offscreen(& PhongConfig, crate :: TargetInfo, & crate :: Context) -> Self

// - Camera
//   - projection_matrix(f32) -> mint :: ColumnMatrix4 < f32 >

// - RawSpace
//   - to_space() -> Space
//   - inverse_matrix() -> mint :: ColumnMatrix4 < f32 >

// - asset_gltf.rs
//   - load_gltf(impl AsRef < Path >, & mut crate :: Scene, crate :: NodeRef, & mut crate :: Context) -> Module

// - Mesh
//   - vertex_stream() -> Option < & VertexStream >
//   - vertex_slice() -> wgpu :: BufferSlice

// - RealConfig

// - Ambient

// - Geometry
//   - stroke(& Path, & StrokeOptions) -> Self
//   - plane(f32) -> Self
//   - fill(& Path) -> Self
//   - sphere(super :: Streams, f32, usize) -> Self
//   - cuboid(super :: Streams, mint :: Vector3 < f32 >) -> Self
//   - bake(& mut plr :: Context) -> plr :: Prototype

// - MeshBuilder
//   - vertex(& [T]) -> & 's mut Self
//   - name(& str) -> & 's mut Self
//   - index(& [u16]) -> & 's mut Self
//   - new(& 'a mut gpu :: context :: Context) -> Self
//   - build() -> Prototype
//   - radius(f32) -> & mut Self

// - Window
//   - run(impl 'static + FnMut (Event)) -> !
//   - new() -> WindowBuilder

// - Array

// - Normal

// - Flat
//   - new_offscreen(crate :: TargetInfo, & crate :: Context) -> Self
//   - new(& crate :: Context) -> Self

// - Position

// - Real
//   - new_offscreen(& RealConfig, crate :: TargetInfo, & crate :: Context) -> Self
//   - new(& RealConfig, & crate :: Context) -> Self

// - SpriteMap
//   - at(mint :: Point2 < usize >) -> crate :: UvRange

// - BufferPool

// - ContextBuilder
//   - build(& W) -> Context
//   - build_offscreen() -> Context
//   - power_hungry(bool) -> Self
//   - software(bool) -> Self

// - WindowBuilder
//   - title(& str) -> Self
//   - size(u32, u32) -> Self
//   - build() -> Window

// - TexCoords

// - asset_obj.rs
//   - load_obj(impl AsRef < Path >, & mut crate :: Scene, crate :: NodeRef, & mut crate :: Context) -> fxhash :: FxHashMap < String , (crate :: EntityRef , crate :: Prototype) >

// - Space

// - Color
//   - green() -> f32
//   - into_vec4() -> [f32 ; 4]
//   - red() -> f32
//   - new(f32, f32, f32, f32) -> Self
//   - from_rgba([f32 ; 4]) -> Self
//   - from_rgb_alpha([f32 ; 3], f32) -> Self
//   - blue() -> f32
//   - into_vec4_gamma() -> [f32 ; 4]
//   - alpha() -> f32

// - Scene
//   - add_directional_light() -> ObjectBuilder < LightBuilder >
//   - new() -> Self
//   - add_node() -> ObjectBuilder < () >
//   - add_entity(& Prototype) -> ObjectBuilder < EntityBuilder >
//   - add_point_light() -> ObjectBuilder < LightBuilder >
//   - lights() -> impl Iterator < Item = (LightRef , & 'a Light) >
//   - add_light(LightKind) -> ObjectBuilder < LightBuilder >
//   - add_sprite(ImageRef) -> ObjectBuilder < SpriteBuilder >
//   - bake() -> BakedScene

// - Material

// - BakedScene

// - ObjectBuilder
//   - orientation_around(mint :: Vector3 < f32 >, f32) -> & mut Self
//   - orientation(mint :: Quaternion < f32 >) -> & mut Self
//   - component(T) -> & mut Self
//   - intensity(f32) -> & mut Self
//   - look_at(mint :: Vector3 < f32 >, mint :: Vector3 < f32 >) -> & mut Self
//   - color(Color) -> & mut Self
//   - build() -> NodeRef
//   - parent(NodeRef) -> & mut Self
//   - uv(UvRange) -> & mut Self
//   - position(mint :: Vector3 < f32 >) -> & mut Self
//   - scale(f32) -> & mut Self

// - SolidConfig

// - Target
//   - aspect() -> f32

// - Node
//   - set_rotation(mint :: Vector3 < f32 >, f32) -> None
//   - pre_move(mint :: Vector3 < f32 >) -> None
//   - set_position(mint :: Vector3 < f32 >) -> None
//   - post_move(mint :: Vector3 < f32 >) -> None
//   - set_scale(f32) -> None
//   - get_position() -> mint :: Vector3 < f32 >
//   - get_scale() -> f32
//   - post_rotate(mint :: Vector3 < f32 >, f32) -> None
//   - pre_rotate(mint :: Vector3 < f32 >, f32) -> None
//   - get_rotation() -> (mint :: Vector3 < f32 > , f32)
