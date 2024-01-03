use crate::{
    math::{cg::Vec3, Quaternion},
    scene::transform::LocalTransform,
    scene::{
        transform::{Transform, TransformId},
        SceneId,
    },
    IsHidden, SceneState, Vec2or3,
};
use std::{
    fmt::{Debug, Formatter},
    sync::{Arc, RwLock},
};

type Error = Box<dyn std::error::Error>;
type SceneObjectPair = (SceneId, ObjectId);

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
        f.debug_struct("< FragmentColor ObjectBuilder >").finish()
    }
}

// Defines an interface for Spatial Components.
///
/// Spatial components constructors must return a `Object<Self>`.
///
/// All Spatial Components are associated with a Transform and must have a
/// TransformId. This trait provides methods for accessing its TransformId.
pub trait APIObject: Default + hecs::Component + Clone + Debug {
    /// Builds a Scene Object from this component.
    fn new() -> Object<Self> {
        Object::new(Self::default())
    }

    // @TODO - it would be useful to have "name" here and associate it with
    //         the Object's typeId. So we can list a scene with nice names.
    //         (context: Rust does not know the object name at runtime)
}

/// The Object is the interface for manipulating Scene objects.
///
/// A Object is a wrapper around public API objects containing
/// spatial information (position, rotation, orientation).
///
/// It is the main intermediator between user commands and the Scene,
/// and provides the object LocalTransform API.
///
/// While FragmentColor is built around an Entity-Component-System (ECS)
/// architecture, it does not expose the inner ECS interface to the
/// user. The Object converts it to a more intuitive, familiar
/// Object-Oriented approach.
#[derive(Debug, Clone)]
pub struct Object<T: APIObject> {
    pub(super) ids: Option<SceneObjectPair>,
    pub(super) builder: ObjectBuilder,
    scene: Option<Arc<RwLock<SceneState>>>,
    transform_id: TransformId,
    transform: Transform,
    batch: bool,
    _type: std::marker::PhantomData<T>,
}

/// This is the interface between the Scene and the Object.
///
/// Users are not supposed to use these methods directly,
/// but they are free to design any object that implements
/// this trait, so they can use custom types as Objects.
pub trait SceneObject {
    fn id(&self) -> Option<ObjectId>;
    fn scene_id(&self) -> Option<SceneId>;
    fn has_moved(&self) -> bool;
    fn transform(&self) -> Transform;
    fn transform_id(&self) -> TransformId;
    fn builder(&mut self) -> &mut hecs::EntityBuilder;
    fn added_to_scene(&mut self, ids: SceneObjectPair, scene: Arc<RwLock<SceneState>>);
    fn added_to_scene_tree(&mut self, transform_id: TransformId);
    fn removed_from_scene(&mut self, ids: SceneObjectPair);
}

impl<T: APIObject> SceneObject for Object<T> {
    /// Returns the ObjectId of this Object
    /// if it has been added to a Scene.
    ///
    /// Otherwise, returns None.
    fn id(&self) -> Option<ObjectId> {
        if let Some((_, object_id)) = self.ids {
            Some(object_id)
        } else {
            None
        }
    }

    /// Returns the SceneId of this Object
    /// if it has been added to a Scene.
    ///
    /// Otherwise, returns None.
    fn scene_id(&self) -> Option<SceneId> {
        if let Some((scene_id, _)) = self.ids {
            Some(scene_id)
        } else {
            None
        }
    }

    fn has_moved(&self) -> bool {
        self.transform.has_moved()
    }

    fn transform(&self) -> Transform {
        if let Some(transform) = self.read_transform_from_scene() {
            transform
        } else {
            self.transform
        }
    }

    fn transform_id(&self) -> TransformId {
        self.transform_id
    }

    fn builder(&mut self) -> &mut hecs::EntityBuilder {
        &mut self.builder.instance
    }

    fn added_to_scene_tree(&mut self, transform_id: TransformId) {
        self.transform_id = transform_id;
    }

    fn added_to_scene(&mut self, ids: SceneObjectPair, scene: Arc<RwLock<SceneState>>) {
        self.ids = Some(ids);
        self.scene = Some(scene);
    }

    fn removed_from_scene(&mut self, ids: SceneObjectPair) {
        let (scene_id, object_id) = ids;
        if self.scene_id() == Some(scene_id) && self.id() == Some(object_id) {
            self.ids = None;
            self.scene = None;
            self.transform = Transform::root();
        } else {
            log::error!(
                "Trying to remove Ids ({:?}, {:?}) from {:?}",
                scene_id,
                object_id,
                self.ids
            );
        }
    }
}

impl<T: APIObject> Object<T> {
    pub fn new(object: T) -> Self {
        let mut scene_object = Object {
            ids: None,
            scene: None,
            transform: Transform::root(),
            transform_id: TransformId::root(),
            builder: ObjectBuilder::default(),
            batch: false,
            _type: std::marker::PhantomData,
        };

        scene_object.add_component(object);

        scene_object
    }

    /// Sets the Object to batch update mode.
    ///
    /// In batch mode, the Object will not update the Scene's Transform
    /// immediately on each change. It will cache the changes and apply
    /// them all at once when `.apply()` is called.
    pub fn batch(&mut self) {
        self.batch = true;
    }

    /// Turns off batch mode and applies all changes to the Scene.
    pub fn apply(&mut self) {
        self.batch = false;
        self.update_transform_in_scene();
    }

    /// Adds or updates one Component in this Object.
    ///
    /// Alias to [`Object::upsert_component()`].
    pub fn add_component<C: hecs::Component>(&mut self, component: C) -> &mut Self {
        self.add_components((component,))
    }

    /// Adds or updates a list of Components in this Object.
    ///
    /// Alias to [`Object::upsert_components()`].
    pub fn add_components<B: hecs::DynamicBundle>(&mut self, bundle: B) -> &mut Self {
        self.upsert_components(bundle)
    }

    /// Adds or updates one Component in this Object.
    ///
    /// Alias to [`Object::upsert_component()`].
    pub fn update_component<C: hecs::Component>(&mut self, component: C) -> &mut Self {
        self.update_components((component,))
    }

    /// Adds or updates a list of Components in this Object.
    ///
    /// Alias to [`Object::upsert_components()`].
    pub fn update_components<B: hecs::DynamicBundle>(&mut self, bundle: B) -> &mut Self {
        self.upsert_components(bundle)
    }

    /// Adds or updates one Component in this Object.
    ///
    /// If the Object already has a Component of the same type,
    /// the old Component will be replaced by the new one.
    pub fn upsert_component<C: hecs::Component>(&mut self, component: C) -> &mut Self {
        self.upsert_components((component,))
    }

    /// Adds or Updates a Bundle of Components to this Object.
    /// A Bundle is a struct which all properties are Components or a tuple of Components.
    ///
    /// Computational cost is proportional to the number of components entity has.
    /// If an entity already has a component of a certain type, it is dropped and replaced.
    pub fn upsert_components<B: hecs::DynamicBundle>(&mut self, bundle: B) -> &mut Self {
        if let Ok((scene, object_id)) = self.scene_object_pair() {
            if let Ok(mut scene) = scene.try_write() {
                if let Err(error) = scene.insert(object_id, bundle) {
                    log::error!(
                        "{:?}: {:?} <{:?}> Cannot add or update component: Object not found!",
                        error,
                        object_id,
                        std::any::type_name::<B>(),
                    );
                };
            } else {
                log::error!(
                    "Scene is locked! Cannot add or update component of {:?}.",
                    object_id
                );
            }
        } else {
            self.builder.instance.add_bundle(bundle);
        };
        self
    }

    /// Removes one Component from this Object.
    pub fn remove_component<C: hecs::Component>(&mut self) -> &mut Self {
        self.remove_components::<(C,)>()
    }

    /// Removes a Bundle of Components from this Object.
    /// A Bundle is a struct which all properties are Components or a tuple of Components.
    ///
    /// # Notes
    /// - This method will only work for objects that have been already added to a Scene.
    pub fn remove_components<B: hecs::Bundle + 'static>(&mut self) -> &mut Self {
        if let Ok((scene, object_id)) = self.scene_object_pair() {
            if let Ok(mut scene) = scene.try_write() {
                if let Err(error) = scene.world.remove::<B>(object_id) {
                    log::error!(
                        "{:?}: {:?} <{:?}> Object not found!",
                        error,
                        object_id,
                        std::any::type_name::<B>(),
                    );
                }
            } else {
                log::error!(
                    "Scene is locked! Cannot remove Components of {:?}.",
                    object_id
                );
            }
        } else {
            log::warn!("Cannot remove Components: Object not in Scene.")
        };
        self
    }

    /// Returns whether this Object has a Component.
    pub fn has_component<C: hecs::Component>(&self) -> bool {
        if let Ok((scene, object_id)) = self.scene_object_pair() {
            if let Ok(scene) = scene.try_read() {
                scene.world.get::<&C>(object_id).is_ok()
            } else {
                log::error!("Scene is locked! Cannot get component of {:?}.", object_id);
                false
            }
        } else {
            self.builder.instance.has::<&C>()
        }
    }

    /// Returns an immutable reference to a Component from this Object.
    pub fn read_component<C: hecs::Component + Copy>(&self) -> Option<C> {
        if let Ok((scene, object_id)) = self.scene_object_pair() {
            if let Ok(scene) = scene.try_read() {
                if let Ok(result) = scene.world.get::<&C>(object_id) {
                    Some(*result.clone())
                } else {
                    log::warn!(
                        "Component {:?} not found in {:?}.",
                        std::any::type_name::<C>(),
                        self.ids,
                    );
                    None
                }
            } else {
                log::error!("Scene is locked! Cannot get component of {:?}.", object_id);
                None
            }
        } else {
            let component = self.builder.instance.get::<&C>();
            component.copied()
        }
    }

    /// Returns a copy of the underlying object, either as a
    /// registered Scene Component or from the temporary builder.
    ///
    /// Callers who would like to update the object should
    /// produce a new instance T for the Object<T> and
    /// add it as a component using `my_object.add_component()`.
    ///
    /// The Object<T> will then replace the object in the
    /// Scene or in the temporary builder.
    pub(crate) fn object(&self) -> T {
        if let Ok((scene, object_id)) = self.scene_object_pair() {
            self.get_object_from_scene(scene, object_id).clone()
        } else {
            self.get_object_from_builder().clone()
        }
    }

    // ------------------------------------------------------------------------
    // Getter and Setters for Parent ID
    // ------------------------------------------------------------------------

    /// Returns this Object's parent TransformId in the Scene tree.
    ///
    /// # Notes:
    /// It's currently not possible to get the parent Object directly.
    /// This is a planned feature (requires reverse lookup in the scene).
    pub fn parent(&self) -> TransformId {
        self.transform.parent
    }

    /// Sets this Object's parent
    pub fn set_parent(&mut self, parent: &impl SceneObject) -> &mut Self {
        self.transform.parent = parent.transform_id();
        self.update_transform()
    }

    /// Internal method to set this Object's parent TransformId
    pub fn set_parent_transform(&mut self, parent: TransformId) -> &mut Self {
        self.transform.parent = parent;
        self.update_transform()
    }

    // ------------------------------------------------------------------------
    // Getters for Local LocalTransform
    // ------------------------------------------------------------------------

    /// Returns this Object's local Transform Matrix.
    pub fn local_transform(&self) -> LocalTransform {
        self.transform.local
    }

    /// Whether this Object has moved relative to its parent.
    pub fn has_moved(&self) -> bool {
        self.transform.has_moved()
    }

    // ------------------------------------------------------------------------
    // Getter and Setters for Position
    // ------------------------------------------------------------------------

    /// Returns this Object's local position.
    pub fn position(&self) -> Vec3 {
        self.transform.local.position.into()
    }

    /// Sets this Object's local position.
    ///
    /// This method simply overwrites the current position data.
    pub fn set_position<V: Into<Vec2or3>>(&mut self, position: V) -> &mut Self {
        let position: Vec3 = position.into().into();
        self.transform.local.position = position.into();
        self.update_transform()
    }

    /// Moves this Object by the given offset.
    ///
    /// The LocalTransformation is implemented as a simple Vec3 (offset) addition
    /// to the current Transform Matrix (M) position component:
    ///
    /// **M' = M.position + offset**
    ///
    /// This works for most use cases where users do not care about the
    /// order of transformations. If you need to apply the translation
    /// before any other transformation that has already been applied,
    /// you can use `Transform.pre_translate()` instead.
    pub fn translate<V: Into<Vec2or3>>(&mut self, offset: V) -> &mut Self {
        let offset: Vec3 = offset.into().into();
        self.transform.local.position += glam::Vec3::from(offset);
        self.update_transform()
    }

    /// Moves this Object by the given offset.
    ///
    /// This method creates a new Offset Transform Matrix (T) containing the
    /// offset vector and multiplies it with the current Transform Matrix (M):
    ///
    /// **M' = T(vec3) * M**
    ///
    /// This is the equivalent of calling Transform.translate() before
    /// applying any other transformation.
    ///
    /// ## Learn more:
    /// <https://stackoverflow.com/questions/3855578>
    pub fn pre_translate<V: Into<Vec2or3>>(&mut self, offset: V) -> &mut Self {
        let offset: Vec3 = offset.into().into();
        let other = LocalTransform {
            position: offset.into(),
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::IDENTITY,
        };
        self.transform.local = other.combine(&self.transform.local);
        self.update_transform()
    }

    // ------------------------------------------------------------------------
    // Getters and Setters for Rotation
    // ------------------------------------------------------------------------

    /// This method is an alias to `Transform.rotation_degrees()`.
    ///
    /// Returns a tuple of (Vec3, float32) representing the Transform's
    /// rotation axis (normalized) and angle (in degrees).
    ///
    /// ## See also:
    /// - Use `Transform.rotation_radians()` to work with Radians instead.
    /// - Use `Transform.rotation_quaternion()` to get a Quaternion
    ///   representing the Transform's rotation.
    pub fn rotation(&self) -> (Vec3, f32) {
        self.rotation_degrees()
    }

    /// Returns a tuple of (Vec3, float32) representing the Transform's
    /// rotation axis (normalized) and angle (in degrees).
    ///
    /// ## See also:
    /// - Use `Transform.rotation_radians()` to work with Radians instead.
    /// - Use `Transform.rotation_quaternion()` to get a Quaternion
    ///   representing the Transform's rotation.
    pub fn rotation_degrees(&self) -> (Vec3, f32) {
        let (axis, angle) = self.transform.local.rotation.to_axis_angle();
        (axis.into(), angle.to_degrees())
    }

    /// Returns a tuple of (Vec3, float32) representing the Transform's
    /// rotation axis (normalized) and angle (in radians).
    ///
    /// ## See also:
    /// - Use `Transform.rotation_degrees()` to work with Degrees instead.
    /// - Use `Transform.rotation_quaternion()` to get a Quaternion
    ///   representing the Transform's rotation.
    pub fn rotation_radians(&self) -> (Vec3, f32) {
        let (axis, angle) = self.transform.local.rotation.to_axis_angle();
        (axis.into(), angle)
    }

    /// Returns the raw quaternion representing the Transform's rotation.
    ///
    /// ## See also:
    /// - Use `Transform.rotation_degrees()` to work with Degrees.
    /// - Use `Transform.rotation_radians()` to work with Radians.
    pub fn rotation_quaternion(&self) -> Quaternion {
        self.transform.local.rotation.into()
    }

    /// This method is an alias to `Transform.set_rotation_degrees()`.
    ///
    /// Sets the Transform's rotation (in degrees), overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Transform.set_rotation_radians()` to work with Radians instead.
    /// - Use `Transform.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    /// - Use `Transform.rotate()` to rotate the Transform by an angle relative to its current rotation.
    pub fn set_rotation<V: Into<Vec2or3>>(&mut self, axis: V, degrees: f32) -> &mut Self {
        self.set_rotation_degrees(axis, degrees)
    }

    /// Sets the Transform's rotation (in degrees), overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Transform.set_rotation_radians()` to work with Radians instead.
    /// - Use `Transform.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    /// - Use `Transform.rotate()` to rotate the Transform by an angle relative to its current rotation.
    pub fn set_rotation_degrees<V: Into<Vec2or3>>(&mut self, axis: V, degrees: f32) -> &mut Self {
        let axis: Vec3 = axis.into().into();
        self.transform.local.rotation =
            glam::Quat::from_axis_angle(axis.into(), degrees.to_radians());
        self.update_transform()
    }

    /// Sets the Transform's rotation (in radians), overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Transform.set_rotation_degrees()` to work with Degrees instead.
    /// - Use `Transform.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    /// - Use `Transform.rotate_radians()` to rotate the Transform by an angle relative to its current rotation.
    pub fn set_rotation_radians<V: Into<Vec2or3>>(&mut self, axis: V, radians: f32) -> &mut Self {
        let axis: Vec3 = axis.into().into();
        self.transform.local.rotation = glam::Quat::from_axis_angle(axis.into(), radians);
        self.update_transform()
    }

    /// Sets the Transform's rotation using a Quaternion, overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Transform.set_rotation_degrees()` to work with Degrees.
    /// - Use `Transform.set_rotation_radians()` to work with Radians.
    /// - Use `Transform.rotate()` to rotate the Transform by an angle relative to its current rotation.
    pub fn set_rotation_quaternion<Q: Into<Quaternion>>(&mut self, quat: Q) -> &mut Self {
        self.transform.local.rotation = quat.into().into();
        self.update_transform()
    }

    /// This method is an alias to `Transform.rotate_degrees()`.
    ///
    /// Rotates the Transform by the given angle (in degrees) relative to its current rotation.
    ///
    /// The transformation (R) is applied as a multiplication of the given rotation
    /// by the `rotation` property of the current LocalTransform matrix (M):
    /// **M' = M * R(degrees)**
    ///
    /// If you need to apply the rotation before any other transformation, you
    /// can use `Transform.pre_rotate()` to set the rotation as the first operand.
    ///
    /// ## See also:
    /// - Use `Transform.rotate_radians()` to work with Radians instead.
    /// - Use `Transform.set_rotation()` to overwrite the rotation using an axis and angle.
    /// - Use `Transform.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    pub fn rotate<V: Into<Vec2or3>>(&mut self, axis: V, degrees: f32) -> &mut Self {
        let axis: Vec3 = axis.into().into();
        self.rotate_degrees(axis, degrees)
    }

    /// Rotates the Transform by the given angle (in degrees) relative to its current rotation.
    ///
    /// The transformation (R) is applied as a multiplication of the given rotation
    /// by the `rotation` property of the current LocalTransform matrix (M):
    /// **M' = M * R(degrees)**
    ///
    /// If you need to apply the rotation before any other transformation, you can
    /// use `Transform.pre_rotate_degrees()` to set the rotation as the first operand.
    ///
    /// ## See also:
    /// - Use `Transform.rotate_radians()` to work with Radians instead.
    /// - Use `Transform.set_rotation()` to overwrite the rotation using an axis and angle.
    /// - Use `Transform.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    pub fn rotate_degrees<V: Into<Vec2or3>>(&mut self, axis: V, degrees: f32) -> &mut Self {
        let axis: Vec3 = axis.into().into();
        self.transform.local.rotation *=
            glam::Quat::from_axis_angle(axis.into(), degrees.to_radians());

        self.update_transform()
    }

    /// Rotates the Transform by the given angle (in radians) relative to its current rotation.
    ///
    /// The transformation (R) is applied as a multiplication of the given rotation
    /// by the `rotation` property of the current LocalTransform matrix (M):
    /// **M' = M * R(degrees)**
    ///
    /// If you need to apply the rotation before any other transformation, you can
    /// use `Transform.pre_rotate_radians()` to set the rotation as the first operand.
    ///
    /// ## See also:
    /// - Use `Transform.rotate()` or `Transform.rotate_degrees()` to work with Degrees instead.
    /// - Use `Transform.set_rotation()` to overwrite the rotation using an axis and angle.
    /// - Use `Transform.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    pub fn rotate_radians<V: Into<Vec2or3>>(&mut self, axis: V, radians: f32) -> &mut Self {
        let axis: Vec3 = axis.into().into();
        self.transform.local.rotation *= glam::Quat::from_axis_angle(axis.into(), radians);

        self.update_transform()
    }

    /// This method is an alias to `Transform.pre_rotate_degrees()`.
    ///
    /// Rotates the Transform by the given angle (in degrees) relative to its current rotation.
    ///
    /// This method creates a new LocalTransform matrix containing the desired rotation and
    /// multiplies it with the current LocalTransform matrix. The Rotation LocalTransform (R)
    /// comes before the current LocalTransform (M) in the order of operands:
    /// **M' = R(degrees) * M**
    ///
    /// This is the equivalent of calling Transform.rotate()
    /// before applying any other transformation.
    ///
    /// ## Learn more:
    /// <https://stackoverflow.com/questions/3855578>
    pub fn pre_rotate<V: Into<Vec2or3>>(&mut self, axis: V, degrees: f32) -> &mut Self {
        let axis: Vec3 = axis.into().into();
        let other = LocalTransform {
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::from_axis_angle(axis.into(), degrees.to_radians()),
        };
        self.transform.local = other.combine(&self.transform.local);

        self.update_transform()
    }

    /// Rotates the Transform by the given angle (in degrees) relative to its current rotation.
    ///
    /// This method creates a new LocalTransform matrix containing the desired rotation and
    /// multiplies it with the current LocalTransform matrix. The Rotation LocalTransform (R)
    /// comes before the current LocalTransform (M) in the order of operands:
    /// **M' = R(degrees) * M**
    ///
    /// This is the equivalent of calling Transform.rotate()
    /// before applying any other transformation.
    ///
    /// ## Learn more:
    /// <https://stackoverflow.com/questions/3855578>
    pub fn pre_rotate_degrees<V: Into<Vec2or3>>(&mut self, axis: V, degrees: f32) -> &mut Self {
        let axis: Vec3 = axis.into().into();
        let other = LocalTransform {
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::from_axis_angle(axis.into(), degrees.to_radians()),
        };
        self.transform.local = other.combine(&self.transform.local);

        self.update_transform()
    }

    /// Rotates the Transform by the given angle (in radians) relative to its current rotation.
    ///
    /// This method creates a new LocalTransform matrix containing the desired rotation and
    /// multiplies it with the current LocalTransform matrix. The Rotation LocalTransform (R)
    /// comes before the current LocalTransform (M) in the order of operands:
    /// **M' = R(degrees) * M**
    ///
    /// This is the equivalent of calling Transform.rotate()
    /// before applying any other transformation.
    ///
    /// ## Learn more:
    /// <https://stackoverflow.com/questions/3855578>
    pub fn pre_rotate_radians<V: Into<Vec2or3>>(&mut self, axis: V, radians: f32) -> &mut Self {
        let axis: Vec3 = axis.into().into();
        let other = LocalTransform {
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::from_axis_angle(axis.into(), radians),
        };
        self.transform.local = other.combine(&self.transform.local);

        self.update_transform()
    }

    /// Sets the Transform's rotation so that it faces the given target.
    pub fn look_at<V: Into<Vec2or3>>(&mut self, target: V, up: V) -> &mut Self {
        let up: Vec3 = up.into().into();
        let target: Vec3 = target.into().into();
        let affine =
            glam::Affine3A::look_at_rh(self.transform.local.position, target.into(), up.into());
        let (_, rotation, _) = affine.inverse().to_scale_rotation_translation();
        self.transform.local.rotation = rotation;

        self.update_transform()
    }

    /// Sets the Transform's rotation to look at (0, 0, 0)
    pub fn look_at_origin<V: Into<Vec2or3>>(&mut self, up: V) -> &mut Self {
        let up: Vec3 = up.into().into();
        let origin = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        self.look_at(origin, up)
    }

    // ------------------------------------------------------------------------
    // Getters and Setters for Scale
    // ------------------------------------------------------------------------

    /// Returns the Transform's local scale
    pub fn scale(&self) -> Vec3 {
        self.transform.local.scale.into()
    }

    /// Sets the Transform's local scale
    pub fn set_scale<S: Into<Vec2or3>>(&mut self, scale: S) -> &mut Self {
        let scale: Vec3 = scale.into().into();
        self.transform.local.scale = scale.into();

        self.update_transform()
    }

    /// Hides this Object from the Scene.
    pub fn hide(&mut self) {
        self.add_component(IsHidden);
    }

    /// Shows this Object in the Scene.
    pub fn show(&mut self) {
        self.remove_component::<IsHidden>();
    }

    //
    //
    // ------------------------------------------------------------------------
    // Internal methods
    // ------------------------------------------------------------------------

    /// Immediately updates the Scene's Transform associated with this Object
    /// if the object is not in batch mode.
    fn update_transform(&mut self) -> &mut Self {
        if !self.batch {
            self.update_transform_in_scene();
        }

        self
    }

    /// Updates the Scene's Transform associated with this Object.
    fn update_transform_in_scene(&self) {
        if let Some(scene) = self.scene.clone() {
            if let Ok(mut scene) = scene.try_write() {
                scene.update_transform(self.transform_id, self.transform)
            } else {
                log::error!("{:?}: Could not write to Scene State.", self.ids);
            }
        };
    }

    /// Reads the Scene's Transform associated with this Object.
    fn read_transform_from_scene(&self) -> Option<Transform> {
        if let Some(scene) = self.scene.clone() {
            if let Ok(scene) = scene.try_read() {
                scene.read_transform(self.transform_id)
            } else {
                log::error!("{:?}: Could not read Scene State.", self.ids);
                None
            }
        } else {
            log::info!("{:?}: Object not in Scene.", self.ids);
            None
        }
    }

    /// Convenience method to return a pair of Scene and Object Ids.
    ///
    /// This is needed while Rust does not support `&&` conditions in
    /// `if let Some(..)` pattern:
    /// https://github.com/rust-lang/rust/issues/53667
    fn scene_object_pair(&self) -> Result<(Arc<RwLock<SceneState>>, ObjectId), Error> {
        if let Some(scene) = self.scene.clone() {
            if let Some(object_id) = self.id() {
                Ok((scene, object_id))
            } else {
                log::error!("{:?}: Object does not have an Id.", self.ids);
                Err("Object does not have an Id".into())
            }
        } else {
            Err("Object is not added to a Scene".into())
        }
    }

    /// Used by the `object()` method.
    /// Returns a copy of the underlying object from the Scene or creates a defult one.
    fn get_object_from_scene(&self, scene: Arc<RwLock<SceneState>>, object_id: ObjectId) -> T {
        if let Ok(scene) = scene.try_read() {
            if let Ok(mut query) = scene.world.query_one::<&T>(object_id) {
                if let Some(object) = query.get() {
                    object.clone()
                } else {
                    log::error!(
                        "{:?} <{:?}> Object not in Scene! Returning a default object.",
                        self.ids,
                        std::any::type_name::<T>(),
                    );
                    T::default()
                }
            } else {
                log::error!(
                    "{:?} <{:?}> Object does not own a component of type {:?}
                    or does not exist in the Scene. Returning a default object.",
                    self.ids,
                    std::any::type_name::<T>(),
                    std::any::type_name::<T>(),
                );
                T::default()
            }
        } else {
            log::error!(
                "{:?} <{:?}> Could not read Scene State. Returning a default object.",
                self.ids,
                std::any::type_name::<T>(),
            );
            T::default()
        }
    }

    /// Used by the `object()` method.
    /// Returns a copy of the underlying object from the temporary builder or creates a defult one.
    fn get_object_from_builder(&self) -> T {
        if let Some(object) = self.builder.instance.get::<&T>() {
            object.clone()
        } else {
            log::error!(
                "{:?} <{:?}> Object does not own a component of type {:?}
                or did not initialize its Builder. Returning a default object.",
                self.ids,
                std::any::type_name::<T>(),
                std::any::type_name::<T>(),
            );
            T::default()
        }
    }
}
