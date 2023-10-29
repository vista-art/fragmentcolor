use crate::color::Color;
use crate::renderer::{resources::mesh::Prototype, texture::TextureId};
use crate::scene::{
    builder::ObjectBuilder,
    entity::EntityBuilder,
    light::{Light, LightBuilder, LightId, LightKind},
    node::{Node, NodeId},
    space::RawSpace,
    sprite::SpriteBuilder,
};
use std::{mem, ops};

// @TODO is this really necessary?
pub struct Array<T>(pub Vec<T>);

impl ops::Index<NodeId> for Array<Node> {
    type Output = Node;
    fn index(&self, node: NodeId) -> &Node {
        &self.0[node.0 as usize]
    }
}
impl ops::IndexMut<NodeId> for Array<Node> {
    fn index_mut(&mut self, node: NodeId) -> &mut Node {
        &mut self.0[node.0 as usize]
    }
}
impl ops::Index<LightId> for Array<Light> {
    type Output = Light;
    fn index(&self, light: LightId) -> &Light {
        &self.0[light.0 as usize]
    }
}
impl ops::IndexMut<LightId> for Array<Light> {
    fn index_mut(&mut self, light: LightId) -> &mut Light {
        &mut self.0[light.0 as usize]
    }
}

pub struct BakedScene {
    spaces: Box<[RawSpace]>,
}

impl ops::Index<NodeId> for BakedScene {
    type Output = RawSpace;
    fn index(&self, node: NodeId) -> &RawSpace {
        &self.spaces[node.0 as usize]
    }
}

pub struct Scene {
    pub world: hecs::World,
    pub nodes: Array<Node>,
    pub lights: Array<Light>,
}

impl ops::Index<NodeId> for Scene {
    type Output = Node;
    fn index(&self, node: NodeId) -> &Node {
        &self.nodes.0[node.0 as usize]
    }
}
impl ops::IndexMut<NodeId> for Scene {
    fn index_mut(&mut self, node: NodeId) -> &mut Node {
        &mut self.nodes.0[node.0 as usize]
    }
}

impl Scene {
    pub fn new() -> Self {
        // @TODO Scene should pick a default camera
        //       without the user having to manually
        //       set it up.
        //
        // let camera = plrender::Camera {
        //     projection: plrender::Projection::Orthographic {
        //         // the sprite configuration is not centered
        //         center: [0.0, -10.0].into(),
        //         extent_y: 40.0,
        //     },
        //     ..Default::default()
        // };

        Self {
            world: Default::default(),
            nodes: Array(vec![Node::default()]),
            lights: Array(Vec::new()),
        }
    }

    // @TODO a scene can contain many cameras, but only one active at a time.
    pub fn camera() {
        todo!()
    }

    // @TODO this method is intended to replace all the other "add" methods below.
    pub fn add(&mut self, _components: impl hecs::DynamicBundle) -> hecs::Entity {
        todo!()
    }

    // this is supposed to be called by the builder
    pub(super) fn set_node_id(&mut self, node: &mut Node) -> NodeId {
        let index = self.nodes.0.len();
        self.nodes.0.push(mem::take(node));
        NodeId(index as u32)
    }

    // I got the pattern now. Every "add" function in Baryon
    // returns a BUILDER. The set_node_id is what actually
    // ADDS the node in the scene.
    pub fn add_node(&mut self) -> ObjectBuilder<()> {
        ObjectBuilder {
            scene: self,
            node: Node::default(),
            kind: (),
        }
    }

    pub fn add_entity(&mut self, prototype: &Prototype) -> ObjectBuilder<EntityBuilder> {
        let mesh = prototype.reference;
        let mut raw = hecs::EntityBuilder::new();
        raw.add_bundle(prototype);
        ObjectBuilder {
            scene: self,
            node: Node::default(),
            kind: EntityBuilder { raw, mesh },
        }
    }

    pub fn add_sprite(&mut self, image: TextureId) -> ObjectBuilder<SpriteBuilder> {
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

    /// Lists all lights in the Scene
    pub fn lights<'a>(&'a self) -> impl Iterator<Item = (LightId, &'a Light)> {
        // In this case, we should iterate over all Entities
        // containing a Light Component (maybe emissive component)
        self.lights
            .0
            .iter()
            .enumerate()
            .map(|(i, light)| (LightId(i as u32), light))
    }

    pub fn bake(&self) -> BakedScene {
        let mut spaces: Vec<RawSpace> = Vec::with_capacity(self.nodes.0.len());
        for n in self.nodes.0.iter() {
            let space = if n.parent == NodeId::default() {
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
