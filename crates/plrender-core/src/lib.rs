mod color;
mod gpu;
mod scene;

pub use color::Color;

pub use gpu::{
    context::{Context, ContextBuilder, ContextDetail},
    renderpass::{Pass, RemderPass},
    target::{HasWindow, Target, TargetInfo, TargetRef},
    texture::{Image, ImageInfo, ImageRef},
};

pub use scene::{
    builder::ObjectBuilder,
    camera::{Camera, Projection},
    entity::{Entity, EntityBuilder, EntityRef},
    light::{Light, LightBuilder, LightKind, LightRef},
    mesh::{IndexStream, Mesh, MeshBuilder, MeshRef, Prototype, Vertex, VertexStream},
    node::{Node, NodeRef},
    space::{RawSpace, Space},
    sprite::{Sprite, SpriteBuilder, UvRange},
    Array, BakedScene, Scene,
};
