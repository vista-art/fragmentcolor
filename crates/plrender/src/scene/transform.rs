use crate::{Quaternion, Vec3};
use serde::{Deserialize, Serialize};

/// A TransformId is a reference for a Transform in the Scene tree.
///
/// Objects that share the same spatial position in the
/// scene might share the same TransformId.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TransformId(pub u32);

impl TransformId {
    pub fn root() -> Self {
        Self::default()
    }
}

/// A Transform represents a spatial position in the Scene.
///
/// Each Transform contains a parent TransformId and a LocalTransform matrix
/// that represents its position, rotation and scale in the Scene.
///
/// Transforms are set to the root of the Scene by default. This means
/// their LocalTransform matrix will be relative to the Scene's origin.
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct Transform {
    pub(super) parent: TransformId,
    pub(super) local: LocalTransform,
}

/// Because the Transform is managed by the Scene, it does not have
/// a public constructor. All of its methods are Getters and
/// Setters for manipulating its spatial data and parent ID.
///
/// All Setters return a mutable referenceto the Transform, so
/// they can be chained.
impl Transform {
    // ------------------------------------------------------------------------
    // Getters for ID and Parent ID; Setter for Parent ID
    // ------------------------------------------------------------------------

    /// Creates a new Root Transform at origin.
    pub fn root() -> Self {
        Self::default()
    }

    /// Returns this Transform's parent TransformId in the Scene tree.
    pub fn parent(&self) -> TransformId {
        self.parent
    }

    /// Sets this Transform's parent TransformId.
    pub fn set_parent(&mut self, parent: TransformId) -> &mut Self {
        self.parent = parent;
        self
    }

    // ------------------------------------------------------------------------
    // Getters for Local LocalTransform
    // ------------------------------------------------------------------------

    /// Returns this Transform's local Transform Matrix.
    pub fn local_transform(&self) -> LocalTransform {
        self.local.clone()
    }

    /// Whether this Transform has moved relative to its parent.
    pub fn has_moved(&self) -> bool {
        self.local != LocalTransform::default()
    }

    // ------------------------------------------------------------------------
    // Getter and Setters for Position
    // ------------------------------------------------------------------------

    /// Returns this Transform's local position.
    pub fn position(&self) -> Vec3 {
        self.local.position.into()
    }

    /// Sets this Transform's local position.
    ///
    /// This method simply overwrites the current position data.
    pub fn set_position(&mut self, position: Vec3) -> &mut Self {
        self.local.position = position.into();
        self
    }

    /// Moves this Transform by the given offset.
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
    pub fn translate(&mut self, offset: Vec3) -> &mut Self {
        self.local.position += glam::Vec3::from(offset);
        self
    }

    /// Moves this Transform by the given offset.
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
    pub fn pre_translate(&mut self, offset: Vec3) {
        let other = LocalTransform {
            position: offset.into(),
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        };
        self.local = other.combine(&self.local);
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
        let (axis, angle) = self.local.rotation.to_axis_angle();
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
        let (axis, angle) = self.local.rotation.to_axis_angle();
        (axis.into(), angle)
    }

    /// Returns the raw quaternion representing the Transform's rotation.
    ///
    /// ## See also:
    /// - Use `Transform.rotation_degrees()` to work with Degrees.
    /// - Use `Transform.rotation_radians()` to work with Radians.
    pub fn rotation_quaternion(&self) -> Quaternion {
        self.local.rotation.into()
    }

    /// This method is an alias to `Transform.set_rotation_degrees()`.
    ///
    /// Sets the Transform's rotation (in degrees), overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Transform.set_rotation_radians()` to work with Radians instead.
    /// - Use `Transform.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    /// - Use `Transform.rotate()` to rotate the Transform by an angle relative to its current rotation.
    pub fn set_rotation(&mut self, axis: Vec3, degrees: f32) -> &mut Self {
        self.set_rotation_degrees(axis, degrees)
    }

    /// Sets the Transform's rotation (in degrees), overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Transform.set_rotation_radians()` to work with Radians instead.
    /// - Use `Transform.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    /// - Use `Transform.rotate()` to rotate the Transform by an angle relative to its current rotation.
    pub fn set_rotation_degrees(&mut self, axis: Vec3, degrees: f32) -> &mut Self {
        self.local.rotation = glam::Quat::from_axis_angle(axis.into(), degrees.to_radians());
        self
    }

    /// Sets the Transform's rotation (in radians), overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Transform.set_rotation_degrees()` to work with Degrees instead.
    /// - Use `Transform.set_rotation_quaternion()` to overwrite the rotation using a Quaternion.
    /// - Use `Transform.rotate_radians()` to rotate the Transform by an angle relative to its current rotation.
    pub fn set_rotation_radians(&mut self, axis: Vec3, radians: f32) -> &mut Self {
        self.local.rotation = glam::Quat::from_axis_angle(axis.into(), radians);
        self
    }

    /// Sets the Transform's rotation using a Quaternion, overwriting the current rotation.
    ///
    /// ## See also:
    /// - Use `Transform.set_rotation_degrees()` to work with Degrees.
    /// - Use `Transform.set_rotation_radians()` to work with Radians.
    /// - Use `Transform.rotate()` to rotate the Transform by an angle relative to its current rotation.
    pub fn set_rotation_quaternion<Q: Into<Quaternion>>(&mut self, quat: Q) -> &mut Self {
        self.local.rotation = quat.into().into();
        self
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
    pub fn rotate(&mut self, axis: Vec3, degrees: f32) -> &mut Self {
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
    pub fn rotate_degrees(&mut self, axis: Vec3, degrees: f32) -> &mut Self {
        self.local.rotation =
            self.local.rotation * glam::Quat::from_axis_angle(axis.into(), degrees.to_radians());

        self
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
    pub fn rotate_radians(&mut self, axis: Vec3, radians: f32) -> &mut Self {
        self.local.rotation =
            self.local.rotation * glam::Quat::from_axis_angle(axis.into(), radians);

        self
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
    pub fn pre_rotate(&mut self, axis: Vec3, degrees: f32) -> &mut Self {
        let other = LocalTransform {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::from_axis_angle(axis.into(), degrees.to_radians()),
            scale: glam::Vec3::ONE,
        };
        self.local = other.combine(&self.local);

        self
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
    pub fn pre_rotate_degrees(&mut self, axis: Vec3, degrees: f32) -> &mut Self {
        let other = LocalTransform {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::from_axis_angle(axis.into(), degrees.to_radians()),
            scale: glam::Vec3::ONE,
        };
        self.local = other.combine(&self.local);

        self
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
    pub fn pre_rotate_radians(&mut self, axis: Vec3, radians: f32) -> &mut Self {
        let other = LocalTransform {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::from_axis_angle(axis.into(), radians),
            scale: glam::Vec3::ONE,
        };
        self.local = other.combine(&self.local);

        self
    }

    /// Sets the Transform's rotation so that it faces the given target.
    pub fn look_at(&mut self, target: Vec3, up: Vec3) -> &mut Self {
        let affine = glam::Affine3A::look_at_rh(self.local.position, target.into(), up.into());
        let (_, rotation, _) = affine.inverse().to_scale_rotation_translation();
        self.local.rotation = rotation;

        self
    }

    // ------------------------------------------------------------------------
    // Getters and Setters for Scale
    // ------------------------------------------------------------------------

    /// Returns the Transform's local scale
    pub fn scale(&self) -> glam::Vec3 {
        self.local.scale
    }

    /// Sets the Transform's local scale
    pub fn set_scale(&mut self, scale: Vec3) -> &mut Self {
        self.local.scale = scale.into();
        self
    }
}

use crate::math::cg::Mat4;
use std::ops;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct LocalTransform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Default for LocalTransform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}

impl LocalTransform {
    pub(crate) fn combine(&self, other: &Self) -> Self {
        Self {
            position: self.scale * (self.rotation * other.position) + self.position,
            rotation: self.rotation * other.rotation,
            scale: self.scale * other.scale,
        }
    }

    fn inverse(&self) -> Self {
        let scale = 1.0 / self.scale;
        let rotation = self.rotation.inverse();
        let position = -scale * (rotation * self.position);
        Self {
            position,
            rotation,
            scale,
        }
    }

    fn to_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

#[derive(Debug)]
pub struct GPULocalTransform {
    pub position: [f32; 4],
    pub rotation: [f32; 4],
    pub scale: [f32; 4],
}

impl From<LocalTransform> for GPULocalTransform {
    fn from(transform: LocalTransform) -> Self {
        Self {
            position: [
                transform.position.x,
                transform.position.y,
                transform.position.z,
                1.0,
            ],
            rotation: transform.rotation.into(),
            scale: [transform.scale.x, transform.scale.y, transform.scale.z, 1.0],
        }
    }
}

impl GPULocalTransform {
    pub fn to_local_transform(&self) -> LocalTransform {
        LocalTransform {
            position: glam::Vec3::new(self.position[0], self.position[1], self.position[2]),
            rotation: glam::Quat::from_array(self.rotation),
            scale: glam::Vec3::new(self.scale[0], self.scale[1], self.scale[2]),
        }
    }

    pub fn inverse_matrix(&self) -> Mat4 {
        self.to_local_transform().inverse().to_matrix().into()
    }
}

pub struct GPUGlobalTransforms {
    pub transforms: Box<[GPULocalTransform]>,
}

impl ops::Index<TransformId> for GPUGlobalTransforms {
    type Output = GPULocalTransform;
    fn index(&self, transform: TransformId) -> &GPULocalTransform {
        &self.transforms[transform.0 as usize]
    }
}
