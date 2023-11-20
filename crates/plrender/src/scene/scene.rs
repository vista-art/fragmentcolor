use crate::{
    app::error::{READ_LOCK_ERROR, WRITE_LOCK_ERROR},
    components,
    components::{GlobalTransforms, LocalTransform},
    renderer::target::RenderTargetDescription,
    scene::{
        node::{Node, NodeId},
        object::{ObjectId, SceneObjectEntry},
    },
    PLRender,
};
use std::{
    collections::HashMap,
    fmt::Debug,
    ops,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneId(pub u32);

#[derive(Debug, Default)]
pub struct Scenes {
    keys: Vec<SceneId>,
    container: HashMap<SceneId, Arc<RwLock<SceneState>>>,
}
crate::app::macros::implements_container!(Scenes, <&SceneId, SceneState>);

#[derive(Debug, Clone)]
pub struct Scene {
    pub(crate) state: Arc<RwLock<SceneState>>,
}

use std::sync::atomic::{AtomicU32, Ordering};

static SCENE_ID: AtomicU32 = AtomicU32::new(1);
impl Scene {
    pub(crate) fn id(&self) -> SceneId {
        self.read_state().id
    }

    /// Creates a new Scene.
    pub fn new() -> Self {
        let app = PLRender::app().read().expect("Could not get App Read lock");

        let mut scene = Self {
            state: Arc::new(RwLock::new(SceneState {
                id: SceneId(SCENE_ID.fetch_add(1, Ordering::Relaxed)),
                world: Default::default(),
                targets: Default::default(),
                nodes: vec![Node::root()],
                has_camera: false,
            })),
        };

        app.add_scene(&mut scene);

        scene
    }

    /// Where all the Scene magic happens! 🧙
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
        let mut state = self.write_state();
        state.add(object)
    }

    /// Renders the Scene.
    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let renderer = PLRender::renderer();
        let renderer = renderer.read().expect(READ_LOCK_ERROR);

        renderer.render(self)
    }

    /// Adds a new rendering target to the Scene.
    pub fn add_target(&mut self, target_data: RenderTargetDescription) {
        let mut state = self.write_state();

        let camera_id = if let Some(camera_id) = target_data.camera_id {
            camera_id
        } else {
            if self.has_camera() {
                log::warn!(
                    "Scene {:?} has cameras, but no camera was specified for the target {:?}.
                    The target will be assigned to the first camera in the Scene.",
                    self.id(),
                    target_data.target_id
                );
                let camera_id = state
                    .cameras()
                    .iter()
                    .next()
                    .expect("Scene has no cameras")
                    .0;

                camera_id
            } else {
                log::info!(
                    "Scene {:?} has no cameras. Creating a default 2D Camera.",
                    self.id()
                );
                let mut camera = components::Camera::from_target_size(target_data.target_size);
                let camera_id = state.add(&mut camera);
                state.has_camera = true;

                camera_id
            }
        };

        let targets = state.targets.entry(camera_id).or_insert_with(Vec::new);
        targets.push(target_data);
    }

    /// Returns true if the Scene has at least one camera.
    pub fn has_camera(&self) -> bool {
        if self.read_state().has_camera {
            //@TODO: invalidate this cache on camera deleted
            return true;
        } else {
            if self.read_state().cameras().iter().next().is_some() {
                self.write_state().has_camera = true;
                return true;
            } else {
                false
            }
        }
    }

    /// Returns a reference to the Scene's internal state.
    pub fn state(&self) -> Arc<RwLock<SceneState>> {
        self.state.clone()
    }

    /// Returns a reference to the Scene's internal state.
    pub fn read_state(&self) -> RwLockReadGuard<'_, SceneState> {
        self.state.read().expect(READ_LOCK_ERROR)
    }

    /// Returns a mutable reference to the Scene's internal state.
    pub fn write_state(&self) -> RwLockWriteGuard<'_, SceneState> {
        self.state.write().expect(WRITE_LOCK_ERROR)
    }
}

/// A HashMap of Camera IDs to a list of RenderTargetDescription.
///
/// Allows the RenderPass to efficiently get all the targets for a given camera.
type CameraTargets = HashMap<ObjectId, Vec<RenderTargetDescription>>;

/// Scene's internal state.
pub struct SceneState {
    id: SceneId,
    pub world: hecs::World,
    targets: CameraTargets,
    nodes: Vec<Node>,
    has_camera: bool,
}

impl Debug for SceneState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("id", &self.id)
            .field("nodes", &self.nodes)
            .field("targets", &self.targets)
            .field("has_camera", &self.has_camera)
            .finish()
    }
}

/// Allows the Nodes to be accessed by their NodeId like an Array.
///
/// Users can access the Scene's Nodes by using the Scene's index operator.
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
    /// Returns the Scene's ID.
    pub(crate) fn id(&self) -> SceneId {
        self.id
    }

    /// Updates a Node in the Scene Tree.
    pub(crate) fn update_node(&mut self, node_id: NodeId, node: Node) {
        self[node_id] = node;
    }

    /// Intenal implementation of the Scene.add() public method.
    pub(crate) fn add(&mut self, object: &mut impl SceneObjectEntry) -> ObjectId {
        let node_id = self.add_to_scene_tree(object);
        object.added_to_scene_tree(node_id);

        let object_id = self.add_to_scene(object);
        object.added_to_scene(self.id(), object_id);

        object_id
    }

    /// Internal method to add the Object's Spatial data (Node) to the Scene Tree.
    ///
    /// Adds the object to the Scene Tree if it has moved relative to its parent.
    /// Otherwise, the object will share the same Node as its parent.
    fn add_to_scene_tree(&mut self, object: &mut impl SceneObjectEntry) -> NodeId {
        if object.has_moved() {
            let index = self.nodes.len();
            object.node().id = NodeId(index as u32);
            self.nodes.push(object.node());
            NodeId(index as u32)
        } else {
            object.node().parent()
        }
    }

    /// Internal method to add the Object to the Scene,
    /// used by the Scene.add() public method.
    ///
    /// Adds the Object's components to the internal ECS World
    /// and returns the Entity ID (typed as ObjectId in our API).
    fn add_to_scene(&mut self, object: &mut impl SceneObjectEntry) -> ObjectId {
        self.world.spawn(object.builder().build())
    }

    /// Used by the RenderPass to get the targets for a given camera
    pub(crate) fn get_camera_targets(&self, camera: ObjectId) -> Vec<RenderTargetDescription> {
        if let Some(targets) = self.targets.get(&camera) {
            targets.clone()
        } else {
            Vec::new()
        }
    }

    /// Iterate over all entities that have certain components
    /// using dynamic borrow checking.
    pub(crate) fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.world.query::<Q>()
    }

    /// Used by the RenderPass to get all cameras in the Scene.
    pub(crate) fn cameras(&self) -> hecs::QueryBorrow<'_, &components::Camera> {
        self.query::<&components::Camera>()
    }

    /// Used by the RenderPass to get all sprites in the Scene.
    pub(crate) fn sprites(&self) -> hecs::QueryBorrow<'_, &components::Sprite> {
        self.query::<&components::Sprite>()
    }

    /// Reurns a reference to a component of an entity.
    pub(crate) fn get<'a, C: hecs::ComponentRef<'a>>(
        &'a self,
        entity: ObjectId,
    ) -> Result<C::Ref, hecs::ComponentError> {
        self.world.get::<C>(entity)
    }

    /// Add components to an Entity
    ///
    /// Computational cost is proportional to the number of components entity has.
    /// If an entity already has a component of a certain type, it is dropped and replaced.
    ///
    /// When inserting a single component, see insert_one() for convenience.
    pub(crate) fn insert<C: hecs::DynamicBundle>(
        &mut self,
        entity: ObjectId,
        component: C,
    ) -> Result<(), hecs::NoSuchEntity> {
        self.world.insert(entity, component)
    }

    /// Calculates the GlobalTransforms for all nodes in the Scene.
    pub(crate) fn get_global_transforms(&self) -> GlobalTransforms {
        let mut transforms: Vec<LocalTransform> = Vec::with_capacity(self.nodes.len());
        for node in self.nodes.iter() {
            let transform = if node.parent == NodeId::root() {
                node.local_transform()
            } else {
                let parent_transform = transforms[node.parent.0 as usize].to_transform();
                parent_transform.combine(&node.local_transform())
            };

            transforms.push(transform.into());
        }

        GlobalTransforms {
            transforms: transforms.into_boxed_slice(),
        }
    }
}
