pub mod animation;
pub mod app;
pub mod color;
pub mod events;
pub mod geometry;
pub mod loader;
pub mod renderer;
pub mod scene;
pub mod target;

pub use animation::Animator;

pub use app::App;

pub use color::Color;

pub use geometry::{Geometry, Vertex};

pub use renderer::{
    mesh::{Bundle, IndexStream, Mesh, MeshBuilder, MeshId, VertexStream},
    renderer::{RenderContext, Renderer},
    renderpass::{Flat2D, Phong, Real, Shader, Solid},
    texture::{Texture, TextureId},
    RenderPass,
};

pub use scene::{
    builder::ObjectBuilder,
    components::{
        camera::{Camera, Projection},
        light::{Light, LightBuilder, LightId, LightVariant},
        sprite::{Sprite, SpriteBuilder, UvRange},
    },
    entity::{Entity, EntityBuilder, EntityId},
    node::{Node, NodeId},
    space::{RawSpace, Space},
    BakedScene, Scene,
};

pub use target::{HasSize, IsWindow, RenderTarget, Target, TargetId, Window};
