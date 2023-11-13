use crate::{
    app::{error::READ_LOCK_ERROR, error::WRITE_LOCK_ERROR, Container},
    components,
    components::{sprite::Sprite, transform::LocalTransform, GlobalTransforms, Renderable},
    renderer::{
        resources::mesh::MeshPrototype,
        target::{HasSize, Target},
        texture::TextureId,
    },
    scene::{
        node::{HasNodeId, Node, NodeId},
        object::{ObjectId, SceneObject, SceneObjectEntry},
    },
    Camera, Color, Projection,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SceneId(pub u32);

#[derive(Debug)]
pub struct Scenes {
    container: HashMap<SceneId, Arc<RwLock<SceneState>>>,
}

pub struct Scene {
    pub state: Arc<RwLock<SceneState>>,
}

impl Scene {
    pub fn new() -> Self {
        // @TODO implement Scenes collection in the App
        // let id = PLRender::app().new_scene();

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
            state: Arc::new(RwLock::new(SceneState {
                world: Default::default(),
                nodes: vec![Node::default()],
            })),
        }
    }

    // @TODO We're almost there to achieve the desired API.
    //       The methods below should be moved to their own
    //       Object constructors after we implement a global
    //       Scene collection in the App.

    /// Returns a new SceneObject with the given Type.
    pub fn new_object<T: HasNodeId>(&mut self, object: T) -> SceneObject<T> {
        SceneObject::new(self.state.clone(), object)
    }

    /// Returns an empty SceneObject without any components.
    pub fn new_empty(&self) -> SceneObject<()> {
        SceneObject::new(self.state.clone(), ())
    }

    /// Returns a new Sprite SceneObject.
    pub fn new_sprite(&self, image: TextureId) -> SceneObject<Sprite> {
        SceneObject {
            id: None,
            builder: hecs::EntityBuilder::new(),
            scene: self.state.clone(),
            node: Node::default(),
            object: Sprite {
                node_id: NodeId::root(),
                image,
                uv: None,
            },
        }
    }

    /// Returns a new Renderable SceneObject.
    pub fn new_renderable(&self, prototype: &MeshPrototype) -> SceneObject<Renderable> {
        let mesh_id = prototype.id;
        let mut builder = hecs::EntityBuilder::new();
        builder.add_bundle(prototype);

        SceneObject {
            id: None,
            builder,
            scene: self.state.clone(),
            node: Node::default(),
            object: Renderable::new(mesh_id),
        }
    }

    pub fn add_target(&self, target: Target) {
        self.state.write().unwrap().add_target(target);
    }

    pub fn camera(&self) -> components::camera::Camera {
        self.state().camera()
    }

    pub fn add(&mut self, object: &mut impl SceneObjectEntry) -> ObjectId {
        let mut state = self.state_mut();
        state.add(object)
    }

    pub fn get_global_transforms(&self) -> GlobalTransforms {
        todo!()
    }

    pub fn state(&self) -> RwLockReadGuard<'_, SceneState> {
        self.state.read().expect(READ_LOCK_ERROR)
    }

    pub fn state_mut(&self) -> RwLockWriteGuard<'_, SceneState> {
        self.state.write().expect(WRITE_LOCK_ERROR)
    }
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

pub struct SceneState {
    pub world: hecs::World,
    nodes: Vec<Node>,
}

impl Debug for SceneState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene").field("nodes", &self.nodes).finish()
    }
}

impl ops::Index<NodeId> for SceneState {
    type Output = Node;
    fn index(&self, node: NodeId) -> &Node {
        &self.nodes[node.0 as usize]
    }
}
impl ops::IndexMut<NodeId> for SceneState {
    fn index_mut(&mut self, node: NodeId) -> &mut Node {
        &mut self.nodes[node.0 as usize]
    }
}

impl SceneState {
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
    // Perhaps this could be a property of a target.
    // Targets would create or query a camera component
    // attached to it.
    //
    // The concept of "active" camera does not make sense
    // if we can have multiple targets.
    pub fn camera(&self) -> components::camera::Camera {
        // @TODO Query all entities with a Camera component

        // Hardcoded from the Pikachu Sprite example. This is
        // the camera we need for the Gaze Circle implementation.
        components::camera::Camera {
            projection: components::camera::Projection::Orthographic {
                // the sprite configuration is not centered
                center: [0.0, -10.0].into(),
                extent_y: 40.0,
            },
            ..Default::default()
        }
    }

    pub fn perspective(&self, camera_node: NodeId) -> Camera {
        // Hardcoded from the Cubes example. A basic 3D camer
        Camera {
            projection: Projection::Perspective { fov_y: 45.0 },
            depth: 1.0..10.0,
            node_id: camera_node, //camera_node.node.id(),
            background: Color(0xFF203040),
        }
    }

    /// Where all the magic happens! 🧙
    ///
    /// Adds a SceneObject to the Scene and returns its ObjectID.
    ///
    /// The Scene maintains two records:
    ///
    /// - The Scene Tree, which is a list of Nodes representing
    ///   positions in the Scene Space. Objects might share the
    ///   same Node if they occupy the same position in Space.
    ///
    /// - The ECS World, which is a list of Entities with their
    ///   Components. Entities are simple IDs, while Components
    ///   can be any type that implements Send + Sync + 'static.
    ///   Components contain the actual data of the SceneObject.
    ///   
    /// The Object must implement the SceneObjectEntry interface.
    /// It is expected that the Objects provide a list of their
    /// Components and a Node object containing Spatial data.
    ///
    /// The Scene will add the Node to the Scene Tree if it has
    /// moved relative to its parent, and return an optional
    /// NodeId to the Object, which will save it internally
    /// or use the same NodeId as its parent.
    ///
    /// The Scene will also create an Entity in the ECS World
    /// containing all the Object's Components, and return an
    /// ObjectId to the Object, which will save it internally.
    ///
    /// # Returns
    /// The Scene returns the ObjectID to the caller, but users
    /// rarely need to use it, as the SceneObject keep track of
    /// its own ObjectId internally.
    pub fn add(&mut self, object: &mut impl SceneObjectEntry) -> ObjectId {
        let node_id = self.add_to_scene_tree(object);
        object.added_to_scene_tree(node_id);

        let object_id = self.add_to_scene(object);
        object.added_to_scene(object_id);

        object_id
    }

    /// Adds the object to the Scene Tree if it has moved relative to its parent.
    /// Otherwise, the object will share the same Node as its parent.
    fn add_to_scene_tree(&mut self, object: &mut impl SceneObjectEntry) -> Option<NodeId> {
        if object.has_moved() {
            let node = object.node();
            let index = self.nodes.len();
            self.nodes.push(mem::take(node));
            let id = NodeId(index as u32);

            Some(id)
        } else {
            None
        }
    }

    /// Adds the Object's components to the internal ECS World
    /// and returns the Entity ID (typed as ObjectId in our API).
    fn add_to_scene(&mut self, object: &mut impl SceneObjectEntry) -> ObjectId {
        self.world.spawn(object.builder().build())
    }

    /// Iterate over all entities that have certain components,
    /// using dynamic borrow checking.
    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.world.query::<Q>()
    }

    /// Reurns a reference to a component of an entity.
    pub fn get<'a, C: hecs::ComponentRef<'a>>(
        &'a self,
        entity: ObjectId,
    ) -> Result<C::Ref, hecs::ComponentError> {
        self.world.get::<C>(entity)
    }

    pub fn insert<C: hecs::DynamicBundle>(
        &mut self,
        entity: ObjectId,
        component: C,
    ) -> Result<(), hecs::NoSuchEntity> {
        self.world.insert(entity, component)
    }

    /// Calculates the GlobalTransforms for all nodes in the Scene.
    pub fn get_global_transforms(&self) -> GlobalTransforms {
        let mut transforms: Vec<LocalTransform> = Vec::with_capacity(self.nodes.len());
        for node in self.nodes.iter() {
            let transform = if node.parent() == NodeId::root() {
                node.local_transform()
            } else {
                let parent_transform = transforms[node.parent().as_usize()].to_transform();
                parent_transform.combine(&node.local_transform())
            };

            transforms.push(transform.into());
        }

        GlobalTransforms {
            transforms: transforms.into_boxed_slice(),
        }
    }
}
