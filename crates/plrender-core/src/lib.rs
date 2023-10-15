mod color;
mod renderer;
mod scene;

pub use color::Color;

pub use renderer::{
    context::{Context, ContextBuilder, ContextDetail},
    mesh::{IndexStream, Mesh, MeshBuilder, MeshRef, Prototype, Vertex, VertexStream},
    target::{HasWindow, Target, TargetInfo, TargetRef},
    texture::{Image, ImageInfo, ImageRef, Texture, TextureRef},
    RenderPass,
};

pub use scene::{
    builder::ObjectBuilder,
    camera::{Camera, Projection},
    entity::{Entity, EntityBuilder, EntityRef},
    light::{Light, LightBuilder, LightKind, LightRef},
    node::{Node, NodeRef},
    space::{RawSpace, Space},
    sprite::{Sprite, SpriteBuilder, UvRange},
    Array, BakedScene, Scene,
};
