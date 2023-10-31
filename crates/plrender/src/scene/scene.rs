use crate::color::Color;
use crate::renderer::{resources::mesh::Bundle, texture::TextureId};
use crate::scene::{
    builder::ObjectBuilder,
    entity::EntityBuilder,
    light::{Light, LightBuilder, LightId, LightVariant},
    node::{Node, NodeId},
    space::RawSpace,
    sprite::SpriteBuilder,
};
use std::{mem, ops};

impl ops::Index<NodeId> for Vec<Node> {
    type Output = Node;
    fn index(&self, node: NodeId) -> &Node {
        &self[node.0 as usize]
    }
}
impl ops::IndexMut<NodeId> for Vec<Node> {
    fn index_mut(&mut self, node: NodeId) -> &mut Node {
        &mut self[node.0 as usize]
    }
}
impl ops::Index<LightId> for Vec<Light> {
    type Output = Light;
    fn index(&self, light: LightId) -> &Light {
        &self[light.0 as usize]
    }
}
impl ops::IndexMut<LightId> for Vec<Light> {
    fn index_mut(&mut self, light: LightId) -> &mut Light {
        &mut self[light.0 as usize]
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
    pub nodes: Vec<Node>,
    pub lights: Vec<Light>,
}

impl ops::Index<NodeId> for Scene {
    type Output = Node;
    fn index(&self, node: NodeId) -> &Node {
        &self.nodes[node.0 as usize]
    }
}
impl ops::IndexMut<NodeId> for Scene {
    fn index_mut(&mut self, node: NodeId) -> &mut Node {
        &mut self.nodes[node.0 as usize]
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
            nodes: vec![Node::default()],
            lights: Vec::new(),
        }
    }

    /// Returns the currently active camera.
    pub fn camera() {
        // queries all entities with a Camera component

        todo!()
    }

    // @TODO this method is intended to replace all the other "add" methods below.
    pub fn add(&mut self, _components: impl hecs::DynamicBundle) -> hecs::Entity {
        todo!()
    }

    // this is supposed to be called by the builder
    pub(super) fn set_node_id(&mut self, node: &mut Node) -> NodeId {
        let index = self.nodes.len();
        self.nodes.push(mem::take(node));
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

    pub fn add_entity(&mut self, prototype: &Bundle) -> ObjectBuilder<EntityBuilder> {
        let mesh = prototype.reference;
        let mut builder = hecs::EntityBuilder::new();
        builder.add_bundle(prototype);
        ObjectBuilder {
            scene: self,
            node: Node::default(),
            kind: EntityBuilder { builder, mesh },
        }
    }

    // Try to implement this method using the generic add() method above.
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

    // Try to implement this method using the generic add() method above.
    pub fn add_light(&mut self, variant: LightVariant) -> ObjectBuilder<LightBuilder> {
        ObjectBuilder {
            scene: self,
            node: Node::default(),
            kind: LightBuilder {
                color: Color(0xFFFFFFFF),
                intensity: 1.0,
                variant,
            },
        }
    }

    pub fn add_directional_light(&mut self) -> ObjectBuilder<LightBuilder> {
        self.add_light(LightVariant::Directional)
    }

    pub fn add_point_light(&mut self) -> ObjectBuilder<LightBuilder> {
        self.add_light(LightVariant::Point)
    }

    /// Lists all lights in the Scene
    pub fn lights<'a>(&'a self) -> impl Iterator<Item = (LightId, &'a Light)> {
        // In this case, we should iterate over all Entities
        // containing a Light Component (maybe emissive component)
        self.lights
            .iter()
            .enumerate()
            .map(|(i, light)| (LightId(i as u32), light))
    }

    pub fn bake(&self) -> BakedScene {
        let mut spaces: Vec<RawSpace> = Vec::with_capacity(self.nodes.len());
        for n in self.nodes.iter() {
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
