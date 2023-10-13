use crate::color::Color;
use crate::gpu::texture::ImageRef;
use crate::scene::{
    builder::ObjectBuilder,
    entity::EntityBuilder,
    light::{Light, LightBuilder, LightKind, LightRef},
    mesh::Prototype,
    node::{Node, NodeRef},
    space::RawSpace,
    sprite::SpriteBuilder,
};
use std::{mem, ops};

// @TODO is this really necessary?
pub struct Array<T>(pub Vec<T>);

impl ops::Index<NodeRef> for Array<Node> {
    type Output = Node;
    fn index(&self, node: NodeRef) -> &Node {
        &self.0[node.0 as usize]
    }
}
impl ops::IndexMut<NodeRef> for Array<Node> {
    fn index_mut(&mut self, node: NodeRef) -> &mut Node {
        &mut self.0[node.0 as usize]
    }
}
impl ops::Index<LightRef> for Array<Light> {
    type Output = Light;
    fn index(&self, light: LightRef) -> &Light {
        &self.0[light.0 as usize]
    }
}
impl ops::IndexMut<LightRef> for Array<Light> {
    fn index_mut(&mut self, light: LightRef) -> &mut Light {
        &mut self.0[light.0 as usize]
    }
}

pub struct BakedScene {
    spaces: Box<[RawSpace]>,
}

impl ops::Index<NodeRef> for BakedScene {
    type Output = RawSpace;
    fn index(&self, node: NodeRef) -> &RawSpace {
        &self.spaces[node.0 as usize]
    }
}

pub struct Scene {
    pub world: hecs::World,
    pub nodes: Array<Node>,
    pub lights: Array<Light>,
}

impl ops::Index<NodeRef> for Scene {
    type Output = Node;
    fn index(&self, node: NodeRef) -> &Node {
        &self.nodes.0[node.0 as usize]
    }
}
impl ops::IndexMut<NodeRef> for Scene {
    fn index_mut(&mut self, node: NodeRef) -> &mut Node {
        &mut self.nodes.0[node.0 as usize]
    }
}

impl Scene {
    pub fn new() -> Self {
        Self {
            world: Default::default(),
            nodes: Array(vec![Node::default()]),
            lights: Array(Vec::new()),
        }
    }

    pub(super) fn add_node_impl(&mut self, node: &mut Node) -> NodeRef {
        let index = self.nodes.0.len();
        self.nodes.0.push(mem::take(node));
        NodeRef(index as u32)
    }

    pub fn add_node(&mut self) -> ObjectBuilder<()> {
        ObjectBuilder {
            scene: self,
            node: Node::default(),
            kind: (),
        }
    }

    pub fn add_entity(&mut self, prototype: &Prototype) -> ObjectBuilder<EntityBuilder> {
        let mut raw = hecs::EntityBuilder::new();
        raw.add_bundle(prototype);
        ObjectBuilder {
            scene: self,
            node: Node::default(),
            kind: EntityBuilder {
                raw,
                mesh: prototype.reference,
            },
        }
    }

    pub fn add_sprite(&mut self, image: ImageRef) -> ObjectBuilder<SpriteBuilder> {
        let raw = hecs::EntityBuilder::new();
        ObjectBuilder {
            scene: self,
            node: Node::default(),
            kind: SpriteBuilder {
                raw,
                image,
                uv: None,
            },
        }
    }

    pub fn add_light(&mut self, kind: LightKind) -> ObjectBuilder<LightBuilder> {
        ObjectBuilder {
            scene: self,
            node: Node::default(),
            kind: LightBuilder {
                color: Color(0xFFFFFFFF),
                intensity: 1.0,
                kind,
            },
        }
    }

    pub fn add_directional_light(&mut self) -> ObjectBuilder<LightBuilder> {
        self.add_light(LightKind::Directional)
    }

    pub fn add_point_light(&mut self) -> ObjectBuilder<LightBuilder> {
        self.add_light(LightKind::Point)
    }

    pub fn lights<'a>(&'a self) -> impl Iterator<Item = (LightRef, &'a Light)> {
        self.lights
            .0
            .iter()
            .enumerate()
            .map(|(i, light)| (LightRef(i as u32), light))
    }

    pub fn bake(&self) -> BakedScene {
        let mut spaces: Vec<RawSpace> = Vec::with_capacity(self.nodes.0.len());
        for n in self.nodes.0.iter() {
            let space = if n.parent == NodeRef::default() {
                n.local.clone()
            } else {
                let parent_space = spaces[n.parent.0 as usize].to_space();
                parent_space.combine(&n.local)
            };
            spaces.push(space.into());
        }
        BakedScene {
            spaces: spaces.into_boxed_slice(),
        }
    }
}
