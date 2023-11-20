use crate::components::Transform;
use serde::{Deserialize, Serialize};

/// A NodeId is a reference for a Node in the Scene tree.
///
/// Objects that share the same spatial position in the
/// scene might share the same NodeId.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NodeId(pub u32);

impl NodeId {
    pub fn root() -> Self {
        Self::default()
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

/// A Node represents a spatial position in the Scene tree.
///
/// Each Node contains a parent NodeId and a Transform matrix
/// that represents its position, rotation and scale in the Scene.
///
/// Nodes are set to the root of the Scene tree by default. This
/// means their parent NodeId is set to zero, and their Transform
/// matrix will be relative to the Scene's origin.
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct Node {
    pub(super) id: NodeId,
    pub(super) parent: NodeId,
    pub(super) local: Transform,
}

/// Because the Node is managed by the Scene, it does not have
/// a public constructor. All of its methods are Getters and
/// Setters for manipulating its spatial data and parent ID.
///
/// All Setters return a mutable referenceto the Node, so
/// they can be chained.
impl Node {
    // ------------------------------------------------------------------------
    // Getters for ID and Parent ID; Setter for Parent ID
    // ------------------------------------------------------------------------

    /// Creates a new Root Node at origin.
    pub fn root() -> Self {
        Self::default()
    }

    /// Returns this Node's NodeId in the Scene tree.
    pub fn id(&self) -> NodeId {
        self.id
        // match self.id {
        //     Some(id) => id,
        //     None => NodeId::root(),
        // }
    }

    /// Returns this Node's parent NodeId in the Scene tree.
    pub fn parent(&self) -> NodeId {
        self.parent
    }

    /// Sets this Node's parent NodeId.
    pub fn set_parent(&mut self, parent: NodeId) -> &mut Self {
        self.parent = parent;
        self
    }

    // ------------------------------------------------------------------------
    // Getters for Local Transform
    // ------------------------------------------------------------------------

    /// Returns this Node's local Transform Matrix.
    pub fn local_transform(&self) -> Transform {
        self.local.clone()
    }

    /// Whether this Node has moved relative to its parent.
    pub fn has_moved(&self) -> bool {
        self.local != Transform::default()
    }

    // ------------------------------------------------------------------------
    // Getter and Setters for Position
    // ------------------------------------------------------------------------

    /// Returns this Node's local position.
    pub fn position(&self) -> mint::Vector3<f32> {
        self.local.position.into()
    }

    /// Sets this Node's local position.
    ///
    /// This method simply overwrites the current position data.
    pub fn set_position(&mut self, position: mint::Vector3<f32>) -> &mut Self {
        self.local.position = position.into();
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
        self.local.position += glam::Vec3::from(offset);
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
        self.local = other.combine(&self.local);
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
        let (axis, angle) = self.local.rotation.to_axis_angle();
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
        let (axis, angle) = self.local.rotation.to_axis_angle();
        (axis.into(), angle)
    }

    /// Returns the raw quaternion representing the Node's rotation.
    ///
    /// ## See also:
    /// - Use `Node.rotation_degrees()` to work with Degrees.
    /// - Use `Node.rotation_radians()` to work with Radians.
    pub fn rotation_quaternion(&self) -> mint::Quaternion<f32> {
        self.local.rotation.into()
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
        self.local.rotation = glam::Quat::from_axis_angle(axis.into(), degrees.to_radians());
        self
    }

    /// Sets the Node's rotation (in radians), overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Node.set_rotation_degrees()` to work with Degrees instead.
    /// - Use `Node.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    /// - Use `Node.rotate_radians()` to rotate the Node by an angle relative to its current rotation.
    pub fn set_rotation_radians(&mut self, axis: mint::Vector3<f32>, radians: f32) -> &mut Self {
        self.local.rotation = glam::Quat::from_axis_angle(axis.into(), radians);
        self
    }

    /// Sets the Node's rotation using a Quaternion, overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Node.set_rotation_degrees()` to work with Degrees.
    /// - Use `Node.set_rotation_radians()` to work with Radians.
    /// - Use `Node.rotate()` to rotate the Node by an angle relative to its current rotation.
    pub fn set_rotation_quaternion(&mut self, quat: mint::Quaternion<f32>) -> &mut Self {
        self.local.rotation = quat.into();
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
        self.local.rotation =
            self.local.rotation * glam::Quat::from_axis_angle(axis.into(), degrees.to_radians());

        self
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
        self.local.rotation =
            self.local.rotation * glam::Quat::from_axis_angle(axis.into(), radians);

        self
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
        self.local = other.combine(&self.local);

        self
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
        self.local = other.combine(&self.local);

        self
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
        self.local = other.combine(&self.local);

        self
    }

    /// Sets the Node's rotation so that it faces the given target.
    pub fn look_at(&mut self, target: mint::Vector3<f32>, up: mint::Vector3<f32>) -> &mut Self {
        let affine = glam::Affine3A::look_at_rh(self.local.position, target.into(), up.into());
        let (_, rotation, _) = affine.inverse().to_scale_rotation_translation();
        self.local.rotation = rotation;

        self
    }

    // ------------------------------------------------------------------------
    // Getters and Setters for Scale
    // ------------------------------------------------------------------------

    /// Returns the Node's local scale
    pub fn scale(&self) -> glam::Vec3 {
        self.local.scale
    }

    /// Sets the Node's local scale
    pub fn set_scale(&mut self, scale: mint::Vector3<f32>) -> &mut Self {
        self.local.scale = scale.into();
        self
    }
}
