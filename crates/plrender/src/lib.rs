pub mod color;
pub mod geometry;
pub mod loader;
pub mod renderer;
pub mod scene;
pub mod target;

pub use color::Color;

pub use geometry::{Geometry, Vertex};

pub use renderer::{
    mesh::{IndexStream, Mesh, MeshBuilder, MeshId, Prototype, VertexStream},
    renderer::{RenderContext, Renderer},
    renderpass::{Flat2D, Phong, Real, Shader, Solid},
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

pub use target::{HasSize, IsWindow, RenderTarget, Target, TargetId, Window};
