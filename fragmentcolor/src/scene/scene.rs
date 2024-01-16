use crate::{
    app::panics,
    components,
    renderer::target::{DescribesTarget, RenderTargetDescription},
    scene::{
        object::{ObjectId, SceneObject},
        transform::{GPUGlobalTransforms, GPULocalTransform, Transform, TransformId},
    },
    Camera, FragmentColor, Object, Quad, TargetId,
};
use std::{
    collections::HashMap,
    fmt::Debug,
    ops,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
};

impl ops::Index<TransformId> for Vec<Transform> {
    type Output = Transform;
    fn index(&self, transform: TransformId) -> &Transform {
        &self[transform.0 as usize]
    }
}
impl ops::IndexMut<TransformId> for Vec<Transform> {
    fn index_mut(&mut self, transform: TransformId) -> &mut Transform {
        &mut self[transform.0 as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneId(pub u32);

#[derive(Debug, Default)]
pub struct Scenes {
    pub keys: Vec<SceneId>,
    container: HashMap<SceneId, Arc<RwLock<SceneState>>>,
}
crate::app::macros::implements_container!(Scenes, <&SceneId, SceneState>);

/// The Scene is the main container for all Objects the user can interact with.
///
/// It is responsible for manageing all Object's relative positions and build a
/// list of Transforms that can be used by the Renderer to display the Objects.
#[derive(Debug, Clone)]
pub struct Scene {
    pub(crate) state: Arc<RwLock<SceneState>>,
}

/// Can't derive Default because of the SceneId
impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

static SCENE_ID: AtomicU32 = AtomicU32::new(1);
impl Scene {
    /// Creates a new Scene.
    ///
    /// The Scene maintains two records:
    ///
    /// - The Scene Tree, which is a list of Transforms representing
    ///   positions in the Scene Space. Objects might share the
    ///   same Transform if they occupy the same position in Space.
    ///
    /// - The ECS World, which is a list of Entities with their
    ///   Components. Entities are simple IDs, while Components
    ///   can be any type that implements Send + Sync + 'static.
    ///   Components contain the actual data of the Object.
    ///
    /// The Scene is registered in the main App as a resource,
    /// so users can create multiple scenes.
    ///
    /// If you need an ad-hoc Scene that acts as a simple
    /// collection, use [Scene::new_unregistered()] instead
    /// (useful for testing).
    pub fn new() -> Self {
        let mut scene = Self::new_unregistered();

        let app = FragmentColor::app();
        if let Ok(app) = app.try_read() {
            app.add_scene(&mut scene);
        } else {
            log::error!("App Locked! Could not register Scene {:?}!", scene.id());
        }

        scene
    }

    /// Creates a new unregistered Scene.
    ///
    /// The Scene is not registered in the main App as a resource,
    /// so it's not accessible by other parts of the system like
    /// the Renderer.
    ///
    /// Use it as a simple container that can report its state.
    /// Useful for testing.
    pub fn new_unregistered() -> Self {
        Self {
            state: Arc::new(RwLock::new(SceneState {
                id: SceneId(SCENE_ID.fetch_add(1, Ordering::Relaxed)),
                world: Default::default(),
                targets: Default::default(),
                transforms: vec![Transform::root()],
                target_indices: HashMap::new(),
            })),
        }
    }

    /// Adds an Object to the Scene and returns its ObjectID.
    ///   
    /// The Object must implement the ObjectEntry interface.
    /// It is expected that the Objects provide a list of their
    /// Components and a Transform object containing Spatial data.
    ///
    /// The Scene will add the Transform to the Scene Tree if it has
    /// moved relative to its parent, and return an optional
    /// TransformId to the Object, which will save it internally
    /// or use the same TransformId as its parent.
    ///
    /// The Scene will also create an Entity in the ECS World
    /// containing all the Object's Components, and return an
    /// ObjectId to the Object, which will save it internally.
    ///
    /// # Returns
    /// The Scene returns the ObjectID to the caller, but users
    /// rarely need to use it, as the Object keep track of
    /// its own ObjectId internally.
    pub fn add(&mut self, object: &mut impl SceneObject) -> ObjectId {
        if let Some(object_id) = object.id() {
            log::warn!("Object {:?} is already part of a Scene!", object.id());
            return object_id;
        }

        let mut state = self.write_state();
        let object_id = state.add(object);
        drop(state);

        object.added_to_scene((self.id(), object_id), self.state.clone());

        object_id
    }

    /// Counts the number of Objects in the Scene.
    pub fn count(&self) -> u32 {
        self.read_state().world.len()
    }

    /// Prints the list of Objects in the Scene.
    pub fn print(&self) {
        println!("Listing all Objects in Scene {:?}:", self.id());
        println!();
        for object in self.read_state().world.iter() {
            println!("{:?}: ___________", object.entity());

            for component in object.component_types() {
                println!(" - {:?}", component);
            }
            println!();
        }
        println!("Listing all Transforms in Scene {:?}:", self.id());
        println!();
        for (id, transform) in self.read_state().transforms.iter().enumerate() {
            println!("{:?}: ___________", id);
            println!(" - local: {:?}", transform.local_transform());
            println!(" - parent: {:?}", transform.parent());
            println!();
        }
        println!("Listing all Targets in Scene {:?}:", self.id());
        println!();
        for (id, (camera, targets)) in self.read_state().targets.iter().enumerate() {
            println!("{:?}: ___________", id);
            println!(" - camera: {:?}", camera);
            println!(" - targets:");
            for targets in targets {
                println!("  -- {:?}", targets);
            }
            println!();
        }
        println!("End of Scene {:?} listing.", self.id());
        println!();
    }

    /// Renders the Scene.
    pub fn render(&self) {
        // -> Result<(), wgpu::SurfaceError> {
        _ = if let Ok(renderer) = FragmentColor::renderer().try_read() {
            renderer.render(self)
        } else {
            log::warn!("Dropped Frame: Scene failed to Acquire Renderer Lock!");
            Err(wgpu::SurfaceError::Lost)
        }
    }

    /// Adds a new rendering target to the Scene.
    pub fn target<D: DescribesTarget>(&mut self, descriptor: &D) {
        if let Ok(description) = descriptor.describe_target() {
            self.add_target(description);
        } else {
            log::error!("Could not describe target! Maybe your texture is not writable?");
            log::info!(
                "Input texture must have the `wgpu::TextureUsage::RENDER_ATTACHMENT` flag set."
            )
        }
    }

    /// Adds a new rendering target to the Scene.
    pub fn target_with_camera<D: DescribesTarget>(
        &mut self,
        descriptor: &D,
        camera: &Object<Camera>,
    ) {
        if let Some(camera_id) = camera.id() {
            if let Ok(mut description) = descriptor.describe_target() {
                description.set_camera_id(camera_id);
                self.target(&description);
            }
        } else {
            log::warn!("Could not describe target! Your Camera is not part of a Scene.");
        }
    }

    /// Adds a new rendering target to the Scene.
    fn add_target(&mut self, target_description: RenderTargetDescription) {
        let camera_id = if let Some(camera_id) = target_description.camera_id {
            camera_id
        } else if let Some(camera_id) = self.first_camera() {
            log::warn!(
                "Scene {:?} has cameras, but no camera was specified for the target {:?}.
                    The target will be assigned to the first camera in the Scene.",
                self.id(),
                target_description.target_id
            );

            camera_id
        } else {
            log::info!(
                "Scene {:?} has no cameras. Creating a default 2D Camera.",
                self.id()
            );

            let mut camera = components::Camera::from_target_size(target_description.target_size);
            self.add(&mut camera)
        };

        let target_description = RenderTargetDescription {
            camera_id: Some(camera_id),
            ..target_description
        };

        let (index, target_id) = {
            let mut state = self.write_state();
            let targets = state.targets.entry(camera_id).or_default();
            let index = targets.len();
            let target_id = target_description.target_id;
            targets.push(target_description);

            (index, target_id)
        };

        let mut state = self.write_state();
        state
            .target_indices
            .entry(target_id)
            .or_default()
            .push((camera_id, index));
    }

    /// Returns the ObjectId of the first camera if the Scene has at least one camera.
    fn first_camera(&self) -> Option<ObjectId> {
        if let Some((camera_id, _camera)) = self.read_state().cameras().iter().next() {
            Some(camera_id)
        } else {
            None
        }
    }

    /// Returns the Scene's ID.
    pub(crate) fn id(&self) -> SceneId {
        self.read_state().id
    }

    /// Returns a reference to the Scene's internal state.
    pub(crate) fn state(&self) -> Arc<RwLock<SceneState>> {
        self.state.clone()
    }

    /// Returns a reference to the Scene's internal state.
    pub fn read_state(&self) -> RwLockReadGuard<'_, SceneState> {
        self.state
            .read()
            .expect(panics::SCENE_FAILED_TO_ACQUIRE_READ_LOCK)
    }

    /// Returns a mutable reference to the Scene's internal state.
    pub fn write_state(&self) -> RwLockWriteGuard<'_, SceneState> {
        self.state
            .write()
            .expect(panics::SCENE_FAILED_TO_ACQUIRE_WRITE_LOCK)
    }
}

/// A HashMap of Camera IDs to a list of Targets.
///
/// Allows the Renderer to efficiently get all the targets for a given camera.
type CameraTargets = HashMap<ObjectId, Vec<RenderTargetDescription>>;
type TargetIndices = HashMap<TargetId, Vec<(ObjectId, usize)>>;

/// Scene's internal state.
pub struct SceneState {
    id: SceneId,
    pub world: hecs::World,
    transforms: Vec<Transform>,
    targets: CameraTargets,
    target_indices: TargetIndices,
}

impl Debug for SceneState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("id", &self.id)
            .field("transforms", &self.transforms)
            .field("targets", &self.targets)
            .finish()
    }
}

/// Allows the Transforms to be accessed by their TransformId like an Array.
///
/// Users can access the Scene's Transforms by using the Scene's index operator.
impl ops::Index<TransformId> for SceneState {
    type Output = Transform;
    fn index(&self, transform: TransformId) -> &Transform {
        &self.transforms[transform.0 as usize]
    }
}
impl ops::IndexMut<TransformId> for SceneState {
    fn index_mut(&mut self, transform: TransformId) -> &mut Transform {
        &mut self.transforms[transform.0 as usize]
    }
}

impl SceneState {
    /// Returns the Scene's ID.
    pub fn id(&self) -> SceneId {
        self.id
    }

    /// Updates a Transform in the Scene Tree.
    pub(crate) fn update_transform(&mut self, transform_id: TransformId, transform: Transform) {
        if transform_id.0 as usize >= self.transforms.len() {
            return;
        }
        self[transform_id] = transform;
    }

    /// Reads a Transform in the Scene Tree.
    pub(crate) fn read_transform(&self, transform_id: TransformId) -> Option<Transform> {
        if transform_id.0 as usize >= self.transforms.len() {
            return None;
        }
        Some(self[transform_id])
    }

    /// Intenal implementation of the Scene.add() public method.
    pub(crate) fn add(&mut self, object: &mut impl SceneObject) -> ObjectId {
        let transform_id = self.add_to_scene_tree(object);
        object.added_to_scene_tree(transform_id);

        self.add_to_scene(object)
    }

    /// Internal method to add the Object's Spatial data (Transform) to the Scene Tree.
    ///
    /// Adds the object to the Scene Tree if it has moved relative to its parent.
    /// Otherwise, the object will share the same Transform as its parent.
    fn add_to_scene_tree(&mut self, object: &mut impl SceneObject) -> TransformId {
        if object.has_moved() {
            let index = self.transforms.len();
            self.transforms.push(object.transform());
            TransformId(index as u32)
        } else {
            object.transform().parent()
        }
    }

    /// Internal method to add the Object to the Scene,
    /// used by the Scene.add() public method.
    ///
    /// Adds the Object's components to the internal ECS World
    /// and returns the Entity ID (typed as ObjectId in our API).
    fn add_to_scene(&mut self, object: &mut impl SceneObject) -> ObjectId {
        self.world.spawn(object.builder().build())
    }

    /// Used by the RenderPass to get the targets for a given camera.
    /// Alias to [Scene::get_object_targets()]
    pub(crate) fn get_camera_targets(&self, camera: ObjectId) -> Vec<RenderTargetDescription> {
        self.get_object_targets(camera)
    }

    /// Used by the RenderPass to get the targets for a given object,
    /// normally a Camera or a Target Sprite.
    pub(crate) fn get_object_targets(&self, object_id: ObjectId) -> Vec<RenderTargetDescription> {
        if let Some(targets) = self.targets.get(&object_id) {
            targets.clone()
        } else {
            Vec::new()
        }
    }

    /// Resizes a Target Description by TargetId
    pub(crate) fn resize_target(&mut self, target_id: TargetId, size: Quad) {
        let instances = if let Some(instances) = self.target_indices.get(&target_id) {
            instances
        } else {
            return;
        };

        for (camera_id, index) in instances {
            if let Some(targets) = self.targets.get_mut(camera_id) {
                if *index < targets.len() {
                    targets[*index].target_size = size;
                }
            }
        }
    }

    /// Removes a target from the Scene.
    pub(crate) fn remove_target(&mut self, target_id: TargetId) {
        let instances = if let Some(instances) = self.target_indices.remove(&target_id) {
            instances
        } else {
            return;
        };

        for (camera_id, index) in instances {
            if let Some(targets) = self.targets.get_mut(&camera_id) {
                if index < targets.len() {
                    targets.remove(index);
                }
            }
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

    /// Components required to build the Locals Uniform for 2D rendering.
    /// This query will return all Sprites, Text, and 2D Shapes.
    ///
    /// Used by the "Toy", "Flat" and "Text" RenderPasses.
    pub(crate) fn get_2d_objects(
        &self,
    ) -> hecs::QueryBorrow<
        '_,
        (
            &TransformId,
            &components::Color,
            &components::Bounds,
            &components::Border,
            &components::ShapeFlag,
        ),
    > {
        self.query::<(
            &TransformId,
            &components::Color,
            &components::Bounds,
            &components::Border,
            &components::ShapeFlag,
        )>()
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
        components: C,
    ) -> Result<(), hecs::NoSuchEntity> {
        self.world.insert(entity, components)
    }

    /// Calculates the GPUGlobalTransforms for all transforms in the Scene.
    pub(crate) fn calculate_global_transforms(&self) -> GPUGlobalTransforms {
        let mut transforms: Vec<GPULocalTransform> = Vec::with_capacity(self.transforms.len());
        for transform in self.transforms.iter() {
            let transform = if transform.parent == TransformId::root() {
                transform.local_transform()
            } else {
                let parent_transform = transforms[transform.parent.0 as usize].to_local_transform();
                parent_transform.combine(&transform.local_transform())
            };

            transforms.push(transform.into());
        }

        GPUGlobalTransforms {
            transforms: transforms.into_boxed_slice(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Circle, CircleOptions};
    use std::ops::Index;

    #[test]
    fn test_new_scene_has_default_state() {
        let scene = Scene::new_unregistered();
        let state = scene.read_state();
        assert!(state.world.is_empty());
        assert_eq!(state.transforms.len(), 1);
        assert!(state.targets.is_empty());
        assert!(state.target_indices.is_empty());
    }

    #[test]
    fn test_add_to_scene() {
        let mut scene = Scene::new_unregistered();
        let mut shape1 = Circle::new(CircleOptions::default());
        let mut shape2 = Circle::new(CircleOptions::default());

        let object1_id = scene.add(&mut shape1);
        let object2_id = scene.add(&mut shape2);

        assert_eq!(object1_id, shape1.id().unwrap());
        assert_eq!(object2_id, shape2.id().unwrap());
        assert_eq!(scene.count(), 2);
    }

    #[test]
    fn test_target_addition() {
        let mut scene = Scene::new_unregistered();
        let descriptor =
            RenderTargetDescription::create_texture_target(Quad::from_size(1, 1)).unwrap();
        scene.target(&descriptor);
        let state = scene.read_state();
        assert!(!state.targets.is_empty());
    }

    #[test]
    fn test_target_with_camera_addition() {
        let mut scene = Scene::new_unregistered();
        let descriptor =
            RenderTargetDescription::create_texture_target(Quad::from_size(1, 1)).unwrap();
        let mut camera = Object::new(Camera::default());
        scene.add(&mut camera);

        scene.target_with_camera(&descriptor, &camera);

        let state = scene.read_state();
        assert!(!state.targets.is_empty());
    }

    #[test]
    fn test_scene_id_increments_properly() {
        let scene1 = Scene::new_unregistered();
        let scene2 = Scene::new_unregistered();

        assert_ne!(scene1.id(), scene2.id());
    }

    #[test]
    fn test_indexing_transforms() {
        let scene = Scene::new_unregistered();
        let state = scene.read_state();
        let first_transform = state.transforms.first().expect("No transforms found");
        assert_eq!(*state.index(TransformId(0)), *first_transform);
    }

    #[test]
    fn test_scene_state_update_transform() {
        let scene = Scene::new_unregistered();
        let mut state = scene.write_state();
        let transform_id = TransformId(0);
        let new_transform = Transform::default();

        state.update_transform(transform_id, new_transform.clone());

        assert_eq!(state.read_transform(transform_id).unwrap(), new_transform);
    }

    #[test]
    fn test_scene_state_add_to_scene_tree() {
        let scene = Scene::new_unregistered();
        let mut state = scene.write_state();
        let mut shape1 = Circle::new(CircleOptions::default());
        let mut shape2 = Circle::new(CircleOptions::default());
        let mut shape3 = Circle::new(CircleOptions::default());

        shape3.translate([1.0, 0.0, 0.0]);

        // not moved
        let shape1_transform = state.add_to_scene_tree(&mut shape1);
        let shape2_transform = state.add_to_scene_tree(&mut shape2);
        // moved
        let shape3_transform = state.add_to_scene_tree(&mut shape3);

        // objects that have not moved are in Scene's root (TransformId(0))
        assert_eq!(shape1.has_moved(), false);
        assert_eq!(shape1_transform, TransformId(0));
        assert_eq!(shape2.has_moved(), false);
        assert_eq!(shape2_transform, TransformId(0));
        assert_eq!(shape3.has_moved(), true);
        assert_eq!(shape3_transform, TransformId(1));
    }

    #[test]
    fn test_scene_state_resize_target() {
        let mut scene = Scene::new_unregistered();
        let descriptor =
            RenderTargetDescription::create_texture_target(Quad::from_size(1, 1)).unwrap();
        scene.target(&descriptor);

        let mut state = scene.write_state();
        let camera_id = state.cameras().iter().next().map(|(id, _)| id).unwrap();
        let new_size = Quad {
            min_x: 0,
            min_y: 0,
            max_x: 100,
            max_y: 100,
        };

        state.resize_target(descriptor.target_id, new_size);

        let updated_target_description = state
            .targets
            .get(&camera_id)
            .unwrap()
            .iter()
            .next()
            .unwrap();

        assert_eq!(updated_target_description.target_size, new_size);
    }

    #[test]
    fn test_scene_state_remove_target() {
        let mut scene = Scene::new_unregistered();
        let descriptor =
            RenderTargetDescription::create_texture_target(Quad::from_size(100, 100)).unwrap();
        scene.target(&descriptor);

        let mut state = scene.write_state();
        let camera_id = state.cameras().iter().next().map(|(id, _)| id).unwrap();
        let target_id = descriptor.target_id;

        state.remove_target(target_id);

        assert!(state.targets.get(&camera_id).unwrap().is_empty());
    }
}
