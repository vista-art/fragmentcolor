mod color;
mod renderer;
mod scene;

pub use color::Color;

pub use renderer::{
    mesh::{IndexStream, Mesh, MeshBuilder, MeshId, Prototype, Vertex, VertexStream},
    renderer::{RenderContext, Renderer},
    target::{HasSize, IsWindow, RenderTarget, Target, TargetId},
    texture::{Texture, TextureId},
    RenderPass,
};

pub use scene::{
    builder::ObjectBuilder,
    camera::{Camera, Projection},
    entity::{Entity, EntityBuilder, EntityRef},
    light::{Light, LightBuilder, LightKind, LightRef},
    node::{Node, NodeId},
    space::{RawSpace, Space},
    sprite::{Sprite, SpriteBuilder, UvRange},
    Array, BakedScene, Scene,
};
