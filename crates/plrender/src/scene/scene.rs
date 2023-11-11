use crate::GlobalTransforms;
use crate::{
    app::{error::READ_LOCK_ERROR, error::WRITE_LOCK_ERROR, Container},
    components,
    components::{sprite::SpriteBuilder, transform::LocalsUniform, RenderableBuilder},
    renderer::{
        resources::mesh::MeshPrototype,
        target::{HasSize, Target},
        texture::TextureId,
    },
    scene::{
        node::{Node, NodeId},
        object::SceneObject,
    },
};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use std::{
    collections::HashMap,
    fmt::Debug,
    mem, ops,
    sync::{Arc, RwLock},
};

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

pub type EntityId = hecs::Entity;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SceneId(pub u32);

#[derive(Debug)]
pub struct Scenes {
    container: HashMap<SceneId, Arc<RwLock<SceneState>>>,
}

#[derive(Debug)]
pub struct SceneState {
    pub instance: Scene,
}

impl Container<SceneId, SceneState> for Scenes {
    fn new() -> Self {
        Self {
            container: HashMap::new(),
        }
    }

    fn get(&self, id: SceneId) -> Option<RwLockReadGuard<'_, SceneState>> {
        let window = self.container.get(&id)?;
        let window = window.read().expect(READ_LOCK_ERROR);
        Some(window)
    }

    fn get_mut(&mut self, id: SceneId) -> Option<RwLockWriteGuard<'_, SceneState>> {
        let window = self.container.get_mut(&id)?;
        let window = window.write().expect(WRITE_LOCK_ERROR);
        Some(window)
    }

    fn insert(&mut self, id: SceneId, window: Arc<RwLock<SceneState>>) {
        self.container.insert(id, window);
    }

    fn remove(&mut self, id: SceneId) -> Option<Arc<RwLock<SceneState>>> {
        self.container.remove(&id)
    }

    fn len(&self) -> usize {
        self.container.len()
    }
}

pub struct Scene {
    world: hecs::World,
    nodes: Vec<Node>,
}

impl Debug for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene").field("nodes", &self.nodes).finish()
    }
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
        }
    }

    // @TODO maybe map a scene camera to each rendering target?
    pub fn add_target(&self, target: Target) {
        let size = target.size();
        let extent_y = size.height as f32 / 2.0;

        let _camera = components::camera::Camera {
            projection: components::camera::Projection::Orthographic {
                // the sprite configuration is not centered
                center: [0.0, -10.0].into(),
                extent_y,
            },
            ..Default::default()
        };
    }

    /// Returns the currently active camera.
    ///
    /// Perhaps this could be a property of a target.
    /// Targets would create or query a camera component
    /// attached to it. The concept of "active" camera
    /// does not make sense if we can have multiple targets.
    pub fn camera(&self) -> components::camera::Camera {
        // queries all entities with a Camera component

        components::camera::Camera {
            projection: components::camera::Projection::Orthographic {
                // the sprite configuration is not centered
                center: [0.0, -10.0].into(),
                extent_y: 40.0,
            },
            ..Default::default()
        }
    }

    pub fn set_active_camera() {
        // @TODO
    }

    // @TODO this method is intended to replace all the other "add" methods below.
    //
    // Maybe instead of receiving a bundle, we should have a custom trait here.
    pub fn add(&mut self, components: impl hecs::DynamicBundle) -> EntityId {
        self.world.spawn(components)
    }

    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.world.query::<Q>()
    }

    pub fn get<'a, T: hecs::ComponentRef<'a>>(
        &'a self,
        entity: EntityId,
    ) -> Result<T::Ref, hecs::ComponentError> {
        self.world.get::<T>(entity)
    }

    /// Increases the scene tree level and returns the new node level.
    pub(crate) fn insert_scene_tree_node(&mut self, node: &mut Node) -> NodeId {
        let index = self.nodes.len();
        self.nodes.push(mem::take(node));
        NodeId(index as u32)
    }

    // I got the pattern now. Every "add" function in Baryon
    // returns a BUILDER. The insert_scene_tree_node is what actually
    // ADDS the node in the scene.
    pub fn new_node(&mut self) -> SceneObject<()> {
        SceneObject {
            scene: self,
            node: Node::default(),
            object: (),
        }
    }

    // Entity, in the context of the legacy Baryon code, represents
    // a bundle that contains a Mesh.
    //
    // This method should be removed after we get rid of the builders.
    // The architecture of this library will be based in pure ECS pattern
    // where an Entity is just an ID representing a collection of arbitrary
    // components that may or may not include a Mesh
    pub fn new_renderable(&mut self, prototype: MeshPrototype) -> SceneObject<RenderableBuilder> {
        let mesh_id = prototype.id;
        let mut builder = hecs::EntityBuilder::new();
        builder.add_bundle(prototype);
        SceneObject {
            scene: self,
            node: Node::default(),
            object: RenderableBuilder { builder, mesh_id },
        }
    }

    // @TODO implement this method using the generic add() method above.
    pub fn new_sprite(&mut self, image: TextureId) -> SceneObject<SpriteBuilder> {
        let builder = hecs::EntityBuilder::new();

        SceneObject {
            scene: self,
            node: Node::default(),
            object: SpriteBuilder {
                builder,
                image,
                uv: None,
            },
        }
    }

    pub fn get_global_transforms(&self) -> GlobalTransforms {
        let mut transforms: Vec<LocalsUniform> = Vec::with_capacity(self.nodes.len());
        for node in self.nodes.iter() {
            let transform = if node.parent() == NodeId::root() {
                node.local()
            } else {
                let parent_transform = transforms[node.parent().as_usize()].to_transform();
                parent_transform.combine(&node.local())
            };

            transforms.push(transform.into());
        }

        GlobalTransforms {
            transforms: transforms.into_boxed_slice(),
        }
    }
}
