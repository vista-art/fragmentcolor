//! Multiplatform GPU Rendering API for Javascript, Python and Beyond

pub mod app;
pub mod components;
pub mod math;
pub mod renderer;
pub mod resources;
pub mod scene;

pub use app::{
    window::{IsWindow, Window},
    App, Event, EventLoop, PLRender,
};

pub use resources::{
    mesh::{BuiltMesh, MeshBuilder, MeshData, MeshId, VertexData, VertexIds},
    texture::{Texture, TextureId},
};

pub use components::{
    animation::Animator,
    camera::{Camera, Projection},
    color::Color,
    light::{Light, LightType},
    mesh::Mesh,
    sprite::Sprite,
    transform::{GlobalTransforms, LocalTransform, Transform},
};

pub use renderer::{
    renderer::{RenderContext, Renderer},
    renderpass::{Flat2D, Phong, Real, Shader, Solid},
    target::{Dimensions, RenderTarget, Target, TargetId},
    RenderPass,
};

pub use scene::{
    node::{Node, NodeId},
    object::SceneObject,
    ObjectId, Scene,
};

pub use math::geometry::{Primitive, Quad, Vertex};
