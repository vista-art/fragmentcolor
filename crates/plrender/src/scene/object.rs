use crate::{
    app::{
        error::{READ_LOCK_ERROR, WRITE_LOCK_ERROR},
        Container,
    },
    components::Transform,
    scene::{
        node::{Node, NodeId},
        SceneId, Scenes,
    },
    PLRender,
};
use std::{
    fmt::{Debug, Formatter},
    sync::{Arc, RwLock},
};

/// Defines an interface for Spatial SceneObjects.
///
/// Spatial objects constructors must return a SceneObject<Self>.
///
/// All Spatial Objects are associated with a Node and must have a
/// Node Id. This trait provides methods for accessing its Node Id.
pub trait SpatialObject: Default + hecs::Component + Copy {
    /// Builds a SceneObject from this component.
    fn new() -> SceneObject<Self> {
        SceneObject::new(Self::default())
    }

    /// Returns the NodeId associated with this component.
    fn node_id(&self) -> NodeId;

    /// Sets the NodeId associated with this component.
    fn set_node_id(&mut self, node_id: NodeId);
}

pub type ObjectId = hecs::Entity;

#[derive(Default)]
pub(crate) struct ObjectBuilder {
    pub(crate) instance: hecs::EntityBuilder,
}

impl Clone for ObjectBuilder {
    fn clone(&self) -> Self {
        Self {
            instance: hecs::EntityBuilder::new(),
        }
    }
}

impl Debug for ObjectBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("< PLRender ObjectBuilder >").finish()
    }
}

/// The SceneObject is the interface for manipulating Scene objects.
///
/// A SceneObject is a wrapper around public API objects containing
/// spatial information (position, rotation, orientation).
///
/// It is the main intermediator between user commands and the Scene,
/// and provides the object Transform API.
///
/// While PLRender is built around an Entity-Component-System (ECS)
/// architecture, it does not expose the inner ECS interface to the
/// user. The SceneObject converts it to a more intuitive, familiar
/// Object-Oriented approach.
#[derive(Debug, Clone)]
pub struct SceneObject<T: SpatialObject> {
    pub(super) id: Option<ObjectId>,
    pub(super) scene_id: Option<SceneId>,
    pub(super) scenes: Arc<RwLock<Scenes>>,
    pub(crate) builder: ObjectBuilder,
    pub node: Node,
    batch: bool,
    object: T,
}

/// This is the interface between the Scene and the SceneObject.
pub trait SceneObjectEntry {
    fn node(&mut self) -> Node;
    fn node_id(&self) -> NodeId;
    fn builder(&mut self) -> &mut hecs::EntityBuilder;
    fn has_moved(&mut self) -> bool;
    fn added_to_scene(&mut self, scene: SceneId, object_id: ObjectId);
    fn removed_from_scene(&mut self, id: ObjectId);
    fn added_to_scene_tree(&mut self, node_id: NodeId);
}

impl<T: SpatialObject> SceneObjectEntry for SceneObject<T> {
    fn node(&mut self) -> Node {
        self.node.clone()
    }

    fn node_id(&self) -> NodeId {
        self.object.node_id()
    }

    fn has_moved(&mut self) -> bool {
        self.node.has_moved()
    }

    fn builder(&mut self) -> &mut hecs::EntityBuilder {
        &mut self.builder.instance
    }

    fn added_to_scene_tree(&mut self, node_id: NodeId) {
        self.object.set_node_id(node_id)
    }

    fn added_to_scene(&mut self, scene_id: SceneId, object_id: ObjectId) {
        self.id = Some(object_id);
        self.scene_id = Some(scene_id);
    }

    fn removed_from_scene(&mut self, object_id: ObjectId) {
        if self.id == Some(object_id) {
            self.id = None;
            self.node = Node::root();
            self.scene_id = None;
            self.object.set_node_id(NodeId::root());
        } else {
            log::error!(
                "Trying to remove ObjectId {} from SceneObject id {}",
                object_id.id(),
                self.id.unwrap().id()
            );
        }
    }
}

impl<T: SpatialObject> SceneObject<T> {
    pub fn new(object: T) -> Self {
        let app = PLRender::app().read().expect(READ_LOCK_ERROR);

        let mut scene_object = SceneObject {
            id: None,
            node: Node::root(),
            scene_id: None,
            scenes: app.scenes(),
            builder: ObjectBuilder::default(),
            batch: false,
            object,
        };

        scene_object.add_component(object);

        scene_object
    }

    /// Returns the ObjectId of this Object if it has been added to a Scene.
    ///
    /// Otherwise, returns None.
    pub fn id(&self) -> Option<ObjectId> {
        self.id
    }

    /// Updates the spatial data of the Scene's Node associated with this Object..
    fn update_node_in_scene(&mut self) {
        if let Some(scene_id) = self.scene_id {
            let scenes = self.scenes.clone();
            let mut scenes = scenes.write().expect(WRITE_LOCK_ERROR);

            let scene = scenes.get_mut(&scene_id);
            let mut scene = if let Some(scene) = scene {
                scene
            } else {
                log::error!(
                    "Scene not found! Cannot update the SceneObject<{:?}'s position, rotation or scale.",
                    std::any::type_name::<T>(),
                );
                return;
            };

            scene.update_node(self.object.node_id(), self.node.clone())
        }
    }

    /// Sets the SceneObject to batch update mode.
    ///
    /// In batch mode, the SceneObject will not update the Scene's Node
    /// immediately on each change. It will cache the changes and apply
    /// them all at once when `.apply()` is called.
    pub fn batch(&mut self) {
        self.batch = true;
    }

    /// Immediately updates the Scene's Node associated with this Object
    /// if the object is not in batch mode.
    fn update_node(&mut self) -> &mut Self {
        if !self.batch {
            self.update_node_in_scene();
        }

        self
    }

    /// Turns off batch mode and applies all changes to the Scene.
    pub fn apply(&mut self) {
        self.batch = false;
        self.update_node_in_scene();
    }

    /// Sets the NodeId of this Object
    pub(crate) fn set_node_id(&mut self, node_id: NodeId) {
        self.object.set_node_id(node_id);
        self.add_component(self.object);
    }

    /// Returns the SceneId and ObjectId associated with this Object
    pub(crate) fn scene_object_tuple(&self) -> Option<(SceneId, ObjectId)> {
        if let Some(scene_id) = self.scene_id {
            if let Some(object_id) = self.id {
                Some((scene_id, object_id))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns a copy of the underlying object, either as a
    /// registered Scene Component or in the temporary builder.
    ///
    /// Users who would like to update the object should
    /// produce a new inner T for the SceneObject<T> and add
    /// it as a component using `SceneObject::add_component()`.
    ///
    /// The SceneObject<T> will then replace the object in the
    /// Scene or in the temporary builder.
    ///
    /// Examples:
    /// ```
    /// use plrender::scene::{SceneObject, node::NodeId};
    ///
    /// #[derive(Default, Clone, Copy)]
    /// struct MyComponent {
    ///    pub node_id: NodeId,
    ///    pub my_data: u32,
    /// }
    ///
    /// impl SceneObject<MyComponent> {
    ///     pub fn set_my_data(&mut self, new_data: u32) -> &mut Self {
    ///         let old_data = self.object();
    ///
    ///         self.add_component(Sprite {
    ///             my_data: new_data,
    ///             ..old_data
    ///         });
    ///
    ///         self
    ///     }
    /// }
    /// ````
    pub(crate) fn object(&mut self) -> T {
        // Can't use && expression with `let Some(var)`
        // https://github.com/rust-lang/rust/issues/53667
        self.object = if let Some(object_id) = self.id {
            let mut scenes = self.scenes.write().expect(WRITE_LOCK_ERROR);
            let scene = if let Some(scene_id) = self.scene_id {
                scenes.get_mut(&scene_id)
            } else {
                None
            };

            let mut scene = if let Some(scene) = scene {
                scene
            } else {
                log::error!(
                    "SceneObject<{:?}>: scene and object_id out of sync!
                    Returning a default value.",
                    std::any::type_name::<T>()
                );
                return T::default();
            };

            let object = if let Ok(object) = scene.world.query_one_mut::<&T>(object_id) {
                object.clone()
            } else {
                log::error!(
                    "The SceneObject<{:?}> {} does not own a component of type {:?}
                    or does not exist in the Scene. Returning a default value.",
                    std::any::type_name::<T>(),
                    object_id.id(),
                    std::any::type_name::<T>(),
                );
                T::default()
            };

            object
        } else {
            if let Some(object) = self.builder.instance.get::<&T>() {
                object.clone()
            } else {
                log::error!(
                    "The SceneObject<{:?}> does not own a component of type {:?}
                    or did not initialize its Builder. Returning a default value.",
                    std::any::type_name::<T>(),
                    std::any::type_name::<T>(),
                );
                T::default()
            }
        };

        self.object
    }

    pub fn add_component<C: hecs::Component>(&mut self, component: C) -> &mut Self {
        self.add_components((component,))
    }

    pub fn add_components<B: hecs::DynamicBundle>(&mut self, bundle: B) -> &mut Self {
        if let Some((scene_id, object_id)) = self.scene_object_tuple() {
            let scenes = self.scenes.clone();
            let mut scenes = scenes.write().expect(WRITE_LOCK_ERROR);

            let scene = scenes.get_mut(&scene_id);
            let mut scene = if let Some(scene) = scene {
                scene
            } else {
                log::error!(
                    "Scene not found! The component {:?} has not been added to SceneObject<{:?}>.",
                    std::any::type_name::<B>(),
                    std::any::type_name::<T>(),
                );
                return self;
            };

            let result = scene.insert(object_id, bundle);
            match result {
                Ok(_) => {}
                Err(error) => log::error!(
                    "The SceneObject<{:?}> (id {}) has not been found in the Scene {:?}: {}",
                    std::any::type_name::<T>(),
                    object_id.id(),
                    scene.id(),
                    error
                ),
            }
        } else {
            self.builder.instance.add_bundle(bundle);
        }
        self
    }

    // ------------------------------------------------------------------------
    // Getter and Setters for Parent ID
    // ------------------------------------------------------------------------

    // @TODO return ObjectId instead in the public API
    //       needs a reverse lookup in the Scene
    //
    /// Returns this Node's parent NodeId in the Scene tree.
    pub fn parent(&self) -> NodeId {
        self.node.parent
    }

    /// Sets this Object's parent
    pub fn set_parent(&mut self, parent: &impl SceneObjectEntry) -> &mut Self {
        self.node.parent = parent.node_id();
        self.update_node()
    }

    /// Internal method to set this Object's parent NodeId
    pub fn set_parent_node(&mut self, parent: NodeId) -> &mut Self {
        self.node.parent = parent;
        self.update_node()
    }

    // ------------------------------------------------------------------------
    // Getters for Local Transform
    // ------------------------------------------------------------------------

    /// Returns this Node's local Transform Matrix.
    pub fn local_transform(&self) -> Transform {
        self.node.local.clone()
    }

    /// Whether this Node has moved relative to its parent.
    pub fn has_moved(&self) -> bool {
        self.node.has_moved()
    }

    // ------------------------------------------------------------------------
    // Getter and Setters for Position
    // ------------------------------------------------------------------------

    /// Returns this Node's local position.
    pub fn position(&self) -> mint::Vector3<f32> {
        self.node.local.position.into()
    }

    /// Sets this Node's local position.
    ///
    /// This method simply overwrites the current position data.
    pub fn set_position(&mut self, position: mint::Vector3<f32>) -> &mut Self {
        self.node.local.position = position.into();
        self.update_node()
    }

    /// Moves this Node by the given offset.
    ///
    /// The Transformation is implemented as a simple Vec3 (offset) addition
    /// to the current Transform Matrix (M) position component:
    ///
    /// **M' = M.position + offset**
    ///
    /// This works for most use cases where users do not care about the
    /// order of transformations. If you need to apply the translation
    /// before any other transformation that has already been applied,
    /// you can use `Node.pre_translate()` instead.
    pub fn translate(&mut self, offset: mint::Vector3<f32>) -> &mut Self {
        self.node.local.position += glam::Vec3::from(offset);
        self.update_node()
    }

    /// Moves this Node by the given offset.
    ///
    /// This method creates a new Offset Transform Matrix (T) containing the
    /// offset vector and multiplies it with the current Transform Matrix (M):
    ///
    /// **M' = T(vec3) * M**
    ///
    /// This is the equivalent of calling Node.translate() before
    /// applying any other transformation.
    ///
    /// ## Learn more:
    /// <https://stackoverflow.com/questions/3855578>
    pub fn pre_translate(&mut self, offset: mint::Vector3<f32>) -> &mut Self {
        let other = Transform {
            position: offset.into(),
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::IDENTITY,
        };
        self.node.local = other.combine(&self.node.local);
        self.update_node()
    }

    // ------------------------------------------------------------------------
    // Getters and Setters for Rotation
    // ------------------------------------------------------------------------

    /// This method is an alias to `Node.rotation_degrees()`.
    ///
    /// Returns a tuple of (Vec3, float32) representing the Node's
    /// rotation axis (normalized) and angle (in degrees).
    ///
    /// ## See also:
    /// - Use `Node.rotation_radians()` to work with Radians instead.
    /// - Use `Node.rotation_quaternion()` to get a Quaternion
    ///   representing the Node's rotation.
    pub fn rotation(&self) -> (mint::Vector3<f32>, f32) {
        self.rotation_degrees()
    }

    /// Returns a tuple of (Vec3, float32) representing the Node's
    /// rotation axis (normalized) and angle (in degrees).
    ///
    /// ## See also:
    /// - Use `Node.rotation_radians()` to work with Radians instead.
    /// - Use `Node.rotation_quaternion()` to get a Quaternion
    ///   representing the Node's rotation.
    pub fn rotation_degrees(&self) -> (mint::Vector3<f32>, f32) {
        let (axis, angle) = self.node.local.rotation.to_axis_angle();
        (axis.into(), angle.to_degrees())
    }

    /// Returns a tuple of (Vec3, float32) representing the Node's
    /// rotation axis (normalized) and angle (in radians).
    ///
    /// ## See also:
    /// - Use `Node.rotation_degrees()` to work with Degrees instead.
    /// - Use `Node.rotation_quaternion()` to get a Quaternion
    ///   representing the Node's rotation.
    pub fn rotation_radians(&self) -> (mint::Vector3<f32>, f32) {
        let (axis, angle) = self.node.local.rotation.to_axis_angle();
        (axis.into(), angle)
    }

    /// Returns the raw quaternion representing the Node's rotation.
    ///
    /// ## See also:
    /// - Use `Node.rotation_degrees()` to work with Degrees.
    /// - Use `Node.rotation_radians()` to work with Radians.
    pub fn rotation_quaternion(&self) -> mint::Quaternion<f32> {
        self.node.local.rotation.into()
    }

    /// This method is an alias to `Node.set_rotation_degrees()`.
    ///
    /// Sets the Node's rotation (in degrees), overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Node.set_rotation_radians()` to work with Radians instead.
    /// - Use `Node.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    /// - Use `Node.rotate()` to rotate the Node by an angle relative to its current rotation.
    pub fn set_rotation(&mut self, axis: mint::Vector3<f32>, degrees: f32) -> &mut Self {
        self.set_rotation_degrees(axis, degrees)
    }

    /// Sets the Node's rotation (in degrees), overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Node.set_rotation_radians()` to work with Radians instead.
    /// - Use `Node.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    /// - Use `Node.rotate()` to rotate the Node by an angle relative to its current rotation.
    pub fn set_rotation_degrees(&mut self, axis: mint::Vector3<f32>, degrees: f32) -> &mut Self {
        self.node.local.rotation = glam::Quat::from_axis_angle(axis.into(), degrees.to_radians());
        self.update_node()
    }

    /// Sets the Node's rotation (in radians), overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Node.set_rotation_degrees()` to work with Degrees instead.
    /// - Use `Node.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    /// - Use `Node.rotate_radians()` to rotate the Node by an angle relative to its current rotation.
    pub fn set_rotation_radians(&mut self, axis: mint::Vector3<f32>, radians: f32) -> &mut Self {
        self.node.local.rotation = glam::Quat::from_axis_angle(axis.into(), radians);
        self.update_node()
    }

    /// Sets the Node's rotation using a Quaternion, overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Node.set_rotation_degrees()` to work with Degrees.
    /// - Use `Node.set_rotation_radians()` to work with Radians.
    /// - Use `Node.rotate()` to rotate the Node by an angle relative to its current rotation.
    pub fn set_rotation_quaternion(&mut self, quat: mint::Quaternion<f32>) -> &mut Self {
        self.node.local.rotation = quat.into();
        self.update_node()
    }

    /// This method is an alias to `Node.rotate_degrees()`.
    ///
    /// Rotates the Node by the given angle (in degrees) relative to its current rotation.
    ///
    /// The transformation (R) is applied as a multiplication of the given rotation
    /// by the `rotation` property of the current Transform matrix (M):
    /// **M' = M * R(degrees)**
    ///
    /// If you need to apply the rotation before any other transformation, you
    /// can use `Node.pre_rotate()` to set the rotation as the first operand.
    ///
    /// ## See also:
    /// - Use `Node.rotate_radians()` to work with Radians instead.
    /// - Use `Node.set_rotation()` to overwrite the rotation using an axis and angle.
    /// - Use `Node.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    pub fn rotate(&mut self, axis: mint::Vector3<f32>, degrees: f32) -> &mut Self {
        self.rotate_degrees(axis, degrees)
    }

    /// Rotates the Node by the given angle (in degrees) relative to its current rotation.
    ///
    /// The transformation (R) is applied as a multiplication of the given rotation
    /// by the `rotation` property of the current Transform matrix (M):
    /// **M' = M * R(degrees)**
    ///
    /// If you need to apply the rotation before any other transformation, you can
    /// use `Node.pre_rotate_degrees()` to set the rotation as the first operand.
    ///
    /// ## See also:
    /// - Use `Node.rotate_radians()` to work with Radians instead.
    /// - Use `Node.set_rotation()` to overwrite the rotation using an axis and angle.
    /// - Use `Node.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    pub fn rotate_degrees(&mut self, axis: mint::Vector3<f32>, degrees: f32) -> &mut Self {
        self.node.local.rotation = self.node.local.rotation
            * glam::Quat::from_axis_angle(axis.into(), degrees.to_radians());

        self.update_node()
    }

    /// Rotates the Node by the given angle (in radians) relative to its current rotation.
    ///
    /// The transformation (R) is applied as a multiplication of the given rotation
    /// by the `rotation` property of the current Transform matrix (M):
    /// **M' = M * R(degrees)**
    ///
    /// If you need to apply the rotation before any other transformation, you can
    /// use `Node.pre_rotate_radians()` to set the rotation as the first operand.
    ///
    /// ## See also:
    /// - Use `Node.rotate()` or `Node.rotate_degrees()` to work with Degrees instead.
    /// - Use `Node.set_rotation()` to overwrite the rotation using an axis and angle.
    /// - Use `Node.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    pub fn rotate_radians(&mut self, axis: mint::Vector3<f32>, radians: f32) -> &mut Self {
        self.node.local.rotation =
            self.node.local.rotation * glam::Quat::from_axis_angle(axis.into(), radians);

        self.update_node()
    }

    /// This method is an alias to `Node.pre_rotate_degrees()`.
    ///
    /// Rotates the Node by the given angle (in degrees) relative to its current rotation.
    ///
    /// This method creates a new Transform matrix containing the desired rotation and
    /// multiplies it with the current Transform matrix. The Rotation Transform (R)
    /// comes before the current Transform (M) in the order of operands:
    /// **M' = R(degrees) * M**
    ///
    /// This is the equivalent of calling Node.rotate()
    /// before applying any other transformation.
    ///
    /// ## Learn more:
    /// <https://stackoverflow.com/questions/3855578>
    pub fn pre_rotate(&mut self, axis: mint::Vector3<f32>, degrees: f32) -> &mut Self {
        let other = Transform {
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::from_axis_angle(axis.into(), degrees.to_radians()),
        };
        self.node.local = other.combine(&self.node.local);

        self.update_node()
    }

    /// Rotates the Node by the given angle (in degrees) relative to its current rotation.
    ///
    /// This method creates a new Transform matrix containing the desired rotation and
    /// multiplies it with the current Transform matrix. The Rotation Transform (R)
    /// comes before the current Transform (M) in the order of operands:
    /// **M' = R(degrees) * M**
    ///
    /// This is the equivalent of calling Node.rotate()
    /// before applying any other transformation.
    ///
    /// ## Learn more:
    /// <https://stackoverflow.com/questions/3855578>
    pub fn pre_rotate_degrees(&mut self, axis: mint::Vector3<f32>, degrees: f32) -> &mut Self {
        let other = Transform {
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::from_axis_angle(axis.into(), degrees.to_radians()),
        };
        self.node.local = other.combine(&self.node.local);

        self.update_node()
    }

    /// Rotates the Node by the given angle (in radians) relative to its current rotation.
    ///
    /// This method creates a new Transform matrix containing the desired rotation and
    /// multiplies it with the current Transform matrix. The Rotation Transform (R)
    /// comes before the current Transform (M) in the order of operands:
    /// **M' = R(degrees) * M**
    ///
    /// This is the equivalent of calling Node.rotate()
    /// before applying any other transformation.
    ///
    /// ## Learn more:
    /// <https://stackoverflow.com/questions/3855578>
    pub fn pre_rotate_radians(&mut self, axis: mint::Vector3<f32>, radians: f32) -> &mut Self {
        let other = Transform {
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::from_axis_angle(axis.into(), radians),
        };
        self.node.local = other.combine(&self.node.local);

        self.update_node()
    }

    /// Sets the Node's rotation so that it faces the given target.
    pub fn look_at(&mut self, target: mint::Vector3<f32>, up: mint::Vector3<f32>) -> &mut Self {
        let affine = glam::Affine3A::look_at_rh(self.node.local.position, target.into(), up.into());
        let (_, rotation, _) = affine.inverse().to_scale_rotation_translation();
        self.node.local.rotation = rotation;

        self.update_node()
    }

    /// Sets the Node's rotation to look at (0, 0, 0)
    pub fn look_at_origin(&mut self, up: mint::Vector3<f32>) -> &mut Self {
        let origin = mint::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        self.look_at(origin, up)
    }

    // ------------------------------------------------------------------------
    // Getters and Setters for Scale
    // ------------------------------------------------------------------------

    /// Returns the Node's local scale
    pub fn scale(&self) -> glam::Vec3 {
        self.node.local.scale
    }

    /// Sets the Node's local scale
    pub fn set_scale(&mut self, scale: mint::Vector3<f32>) -> &mut Self {
        self.node.local.scale = scale.into();

        self.update_node()
    }
}
