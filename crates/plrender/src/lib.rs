//! Multiplatform GPU Rendering API for Javascript, Python and Beyond

pub mod app;
pub mod asset;
pub mod components;
pub mod geometry;
pub mod renderer;
pub mod scene;

pub use app::{
    window::{IsWindow, Window},
    App, Event, EventLoop, PLRender,
};

pub use geometry::{Geometry, Vertex};

pub use components::{
    animation::Animator,
    camera::{Camera, Projection},
    color::Color,
    light::{Light, LightBuilder, LightType},
    renderable::{Renderable, RenderableBuilder},
    sprite::{Sprite, SpriteBuilder, UvRange},
    transform::{GlobalTransforms, LocalsUniform, Transform},
};

pub use renderer::{
    mesh::{Mesh, MeshBuilder, MeshId, MeshPrototype, VertexIds, VertexStream},
    renderer::{RenderContext, Renderer},
    renderpass::{Flat2D, Phong, Real, Shader, Solid},
    target::{HasSize, RenderTarget, Target, TargetId},
    texture::{Texture, TextureId},
    RenderPass,
};

pub use scene::{
    node::{Node, NodeId},
    object::SceneObject,
    EntityId, Scene,
};
