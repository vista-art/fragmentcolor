mod color;
mod geometry;
pub mod loader;
mod renderer;
mod renderpass;
mod scene;
pub mod window;

pub use color::Color;

pub use geometry::{Geometry, Vertex};

pub use renderpass::{Flat, Phong, Real, Shader, Solid};

pub use renderer::{
    mesh::{IndexStream, Mesh, MeshBuilder, MeshId, Prototype, VertexStream},
    renderer::{RenderContext, Renderer},
    target::{HasSize, IsWindow, RenderTarget, Target, TargetId},
    texture::{Texture, TextureId},
    RenderPass,
};

pub use scene::{
    builder::ObjectBuilder,
    camera::{Camera, Projection},
    entity::{Entity, EntityBuilder, EntityId},
    light::{Light, LightBuilder, LightId, LightKind},
    node::{Node, NodeId},
    space::{RawSpace, Space},
    sprite::{Sprite, SpriteBuilder, UvRange},
    Array, BakedScene, Scene,
};
