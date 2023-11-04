pub mod animation;
pub mod app;
pub mod asset;
pub mod color;
pub mod geometry;
pub mod renderer;
pub mod scene;

pub use animation::Animator;

pub use app::{
    window::{IsWindow, Window},
    App, Event, EventLoop,
};

pub use color::Color;

pub use geometry::{Geometry, Vertex};

pub use renderer::{
    mesh::{Bundle, IndexStream, Mesh, MeshBuilder, MeshId, VertexStream},
    renderer::{RenderContext, Renderer},
    renderpass::{Flat2D, Phong, Real, Shader, Solid},
    target::{HasSize, RenderTarget, Target, TargetId},
    texture::{Texture, TextureId},
    RenderPass,
};

pub use scene::{
    builder::ObjectBuilder,
    components::{
        camera::{Camera, Projection},
        light::{Light, LightBuilder, LightType},
        sprite::{Sprite, SpriteBuilder, UvRange},
    },
    entity::{Entity, EntityBuilder, EntityId},
    node::{Node, NodeId},
    space::{RawSpace, Space},
    BakedScene, Scene,
};
