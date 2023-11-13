use log::warn;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    components::Transform,
    scene::{
        node::{HasNodeId, Node, NodeId},
        SceneState,
    },
};

pub type ObjectId = hecs::Entity;

pub struct SceneObject<T: HasNodeId> {
    pub(super) id: Option<ObjectId>,
    pub(crate) builder: hecs::EntityBuilder,
    pub scene: Arc<RwLock<SceneState>>,
    pub node: Node,
    pub object: T,
}

/// This is the interface between the Scene and the SceneObject.
pub trait SceneObjectEntry {
    fn node(&mut self) -> &mut Node;
    fn has_moved(&mut self) -> bool;
    fn builder(&mut self) -> &mut hecs::EntityBuilder;
    fn added_to_scene(&mut self, id: ObjectId);
    fn removed_from_scene(&mut self, id: ObjectId);
    fn added_to_scene_tree(&mut self, node_id: Option<NodeId>);
}

impl<T: HasNodeId> SceneObjectEntry for SceneObject<T> {
    fn node(&mut self) -> &mut Node {
        &mut self.node
    }

    fn has_moved(&mut self) -> bool {
        self.node.has_moved()
    }

    fn builder(&mut self) -> &mut hecs::EntityBuilder {
        &mut self.builder
    }

    fn added_to_scene(&mut self, id: ObjectId) {
        self.id = Some(id);
    }

    fn removed_from_scene(&mut self, _: ObjectId) {
        if let Some(_self_id) = self.id {
            self.id = None;
        }
    }

    fn added_to_scene_tree(&mut self, node_id: Option<NodeId>) {
        let node_id = match node_id {
            None => self.node.parent,
            Some(id) => id,
        };

        self.object.set_node_id(node_id);
    }
}

// @TODO I can remove the generics if I pass a SceneId instead.
//       Then, I would query the global App to get the Scene.
//
//       For the T component's node_ids, it would maybe be better to
//       add the Node as a component, then make the renderer query
//       the Nodes (or the Transforms) directly.
impl<T: HasNodeId> SceneObject<T> {
    // NOTE: this assumes the child object is constructed
    //       should we have a trait for child objects?
    pub fn new(scene: Arc<RwLock<SceneState>>, object: T) -> Self {
        SceneObject {
            id: None,
            node: Node::default(),
            builder: hecs::EntityBuilder::new(),
            scene,
            object,
        }
    }

    pub fn add_component<C: hecs::Component>(&mut self, component: C) -> &mut Self {
        self.add_components((component,))
    }

    pub fn add_components<B: hecs::DynamicBundle>(&mut self, bundle: B) -> &mut Self {
        if let Some(entity) = self.id {
            let mut scene = self.state_mut();
            let result = scene.insert(entity, bundle);
            match result {
                Ok(_) => {}
                Err(error) => warn!(
                    "The Object {} has not been found in the Scene: {:?}",
                    entity.id(),
                    error
                ),
            }
        } else {
            self.builder.add_bundle(bundle);
        }
        self
    }

    // pub fn add_to_scene(&mut self) -> ObjectId {
    //     let mut scene = self.state_mut();
    //     scene.add(self)
    // }

    pub fn scene(&self) -> RwLockReadGuard<'_, SceneState> {
        let scene = &self.scene;
        scene.read().expect("Could not get SceneState read lock")
    }

    pub fn state_mut(&self) -> RwLockWriteGuard<'_, SceneState> {
        let scene = &self.scene;
        scene.write().expect("Could not get SceneState write lock")
    }

    // ------------------------------------------------------------------------
    // Getter and Setter for Parent ID
    // ------------------------------------------------------------------------

    pub fn id(&self) -> Option<ObjectId> {
        self.id
    }

    /// Returns this Node's parent NodeId in the Scene tree.
    pub fn parent(&self) -> NodeId {
        self.node.parent
    }

    /// Sets this Node's parent NodeId.
    pub fn set_parent(&mut self, parent: NodeId) -> &mut Self {
        self.node.parent = parent;
        self
    }

    // ------------------------------------------------------------------------
    // Getters for Local Transform
    // ------------------------------------------------------------------------

    /// Returns this Node's local Transform Matrix.
    pub fn local_transform(&self) -> Transform {
        self.node.local.clone()
    }

    /// Whether this Node has moved relative to its parent.
    pub fn has_moved(&mut self) -> bool {
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
        self
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
        self
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
    pub fn pre_translate(&mut self, offset: mint::Vector3<f32>) {
        let other = Transform {
            position: offset.into(),
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::IDENTITY,
        };
        self.node.local = other.combine(&self.node.local);
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
        self
    }

    /// Sets the Node's rotation (in radians), overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Node.set_rotation_degrees()` to work with Degrees instead.
    /// - Use `Node.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    /// - Use `Node.rotate_radians()` to rotate the Node by an angle relative to its current rotation.
    pub fn set_rotation_radians(&mut self, axis: mint::Vector3<f32>, radians: f32) -> &mut Self {
        self.node.local.rotation = glam::Quat::from_axis_angle(axis.into(), radians);
        self
    }

    /// Sets the Node's rotation using a Quaternion, overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Node.set_rotation_degrees()` to work with Degrees.
    /// - Use `Node.set_rotation_radians()` to work with Radians.
    /// - Use `Node.rotate()` to rotate the Node by an angle relative to its current rotation.
    pub fn set_rotation_quaternion(&mut self, quat: mint::Quaternion<f32>) -> &mut Self {
        self.node.local.rotation = quat.into();
        self
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
    pub fn rotate(&mut self, axis: mint::Vector3<f32>, degrees: f32) {
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
    pub fn rotate_degrees(&mut self, axis: mint::Vector3<f32>, degrees: f32) {
        self.node.local.rotation = self.node.local.rotation
            * glam::Quat::from_axis_angle(axis.into(), degrees.to_radians());
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
    pub fn rotate_radians(&mut self, axis: mint::Vector3<f32>, radians: f32) {
        self.node.local.rotation =
            self.node.local.rotation * glam::Quat::from_axis_angle(axis.into(), radians);
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
    pub fn pre_rotate(&mut self, axis: mint::Vector3<f32>, degrees: f32) {
        let other = Transform {
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::from_axis_angle(axis.into(), degrees.to_radians()),
        };
        self.node.local = other.combine(&self.node.local);
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
    pub fn pre_rotate_degrees(&mut self, axis: mint::Vector3<f32>, degrees: f32) {
        let other = Transform {
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::from_axis_angle(axis.into(), degrees.to_radians()),
        };
        self.node.local = other.combine(&self.node.local);
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
    pub fn pre_rotate_radians(&mut self, axis: mint::Vector3<f32>, radians: f32) {
        let other = Transform {
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::from_axis_angle(axis.into(), radians),
        };
        self.node.local = other.combine(&self.node.local);
    }

    /// Sets the Node's rotation so that it faces the given target.
    pub fn look_at(&mut self, target: mint::Vector3<f32>, up: mint::Vector3<f32>) -> &mut Self {
        let affine = glam::Affine3A::look_at_rh(self.node.local.position, target.into(), up.into());
        let (_, rotation, _) = affine.inverse().to_scale_rotation_translation();
        self.node.local.rotation = rotation;

        self
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
        self
    }
}
