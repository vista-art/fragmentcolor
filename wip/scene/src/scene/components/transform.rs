use cgmath::{
    Affine3A, InnerSpace, Matrix3, Matrix4, One, Quaternion, Rotation3, SquareMatrix,
    Transform as CgTransform, Vector3, Zero,
};
use std::ops::Mul;

pub struct Transform {
    pub translation: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

/// Describe the position of an entity. If the entity has a parent, the position is relative
/// to its parent position.
///
/// * To place or move an entity, you should set its [`Transform`].
/// * To get the global transform of an entity, you should get its [`GlobalTransform`].
/// * To be displayed, an entity must have both a [`Transform`] and a [`GlobalTransform`].
///   * You may use the [`TransformBundle`](crate::TransformBundle) to guarantee this.
///
/// ## [`Transform`] and [`GlobalTransform`]
///
/// [`Transform`] is the position of an entity relative to its parent position, or the reference
/// frame if it doesn't have a [`Parent`](bevy_hierarchy::Parent).
///
/// [`GlobalTransform`] is the position of an entity relative to the reference frame.
///
/// [`GlobalTransform`] is updated from [`Transform`] by systems in the system set
/// [`TransformPropagate`](crate::TransformSystem::TransformPropagate).
///
/// This system runs during [`PostUpdate`](bevy_app::PostUpdate). If you
/// update the [`Transform`] of an entity during this set or after, you will notice a 1 frame lag
/// before the [`GlobalTransform`] is updated.
///
/// # Examples
///
/// - [`transform`]
/// - [`global_vs_local_translation`]
///
/// [`global_vs_local_translation`]: https://github.com/bevyengine/bevy/blob/latest/examples/transforms/global_vs_local_translation.rs
/// [`transform`]: https://github.com/bevyengine/bevy/blob/latest/examples/transforms/transform.rs
/// [`Transform`]: super::Transform
#[derive(Debug, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Transform {
    /// Position of the entity. In 2d, the last value of the `Vector3` is used for z-ordering.
    ///
    /// See the [`translations`] example for usage.
    ///
    /// [`translations`]: https://github.com/bevyengine/bevy/blob/latest/examples/transforms/translation.rs
    pub translation: Vector3<f32>,
    /// Rotation of the entity.
    ///
    /// See the [`3d_rotation`] example for usage.
    ///
    /// [`3d_rotation`]: https://github.com/bevyengine/bevy/blob/latest/examples/transforms/3d_rotation.rs
    pub rotation: Quaternion<f32>,
    /// Scale of the entity.
    ///
    /// See the [`scale`] example for usage.
    ///
    /// [`scale`]: https://github.com/bevyengine/bevy/blob/latest/examples/transforms/scale.rs
    pub scale: Vector3<f32>,
}

impl Transform {
    /// An identity [`Transform`] with no translation, rotation, and a scale of 1 on all axes.
    pub const IDENTITY: Self = Transform {
        translation: cgmath::Vector3::zero(),
        rotation: cgmath::Quaternion::one(),
        scale: cgmath::Vector3::one(),
    };

    /// Creates a new [`Transform`] at the position `(x, y, z)`. In 2d, the `z` component
    /// is used for z-ordering elements: higher `z`-value will be in front of lower
    /// `z`-value.
    #[inline]
    pub const fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_translation(Vector3::new(x, y, z))
    }

    /// Extracts the translation, rotation, and scale from `matrix`. It must be a 3d affine
    /// transformation matrix.
    #[inline]
    pub fn from_matrix(matrix: Mat4) -> Self {
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();

        Transform {
            translation,
            rotation,
            scale,
        }
    }

    /// Creates a new [`Transform`], with `translation`. Rotation will be 0 and scale 1 on
    /// all axes.
    #[inline]
    pub const fn from_translation(translation: Vector3) -> Self {
        Transform {
            translation,
            ..Self::IDENTITY
        }
    }

    /// Creates a new [`Transform`], with `rotation`. Translation will be 0 and scale 1 on
    /// all axes.
    #[inline]
    pub const fn from_rotation(rotation: Quat) -> Self {
        Transform {
            rotation,
            ..Self::IDENTITY
        }
    }

    /// Creates a new [`Transform`], with `scale`. Translation will be 0 and rotation 0 on
    /// all axes.
    #[inline]
    pub const fn from_scale(scale: Vector3) -> Self {
        Transform {
            scale,
            ..Self::IDENTITY
        }
    }

    /// Returns this [`Transform`] with a new rotation so that [`Transform::forward`]
    /// points towards the `target` position and [`Transform::up`] points towards `up`.
    ///
    /// In some cases it's not possible to construct a rotation. Another axis will be picked in those cases:
    /// * if `target` is the same as the transform translation, `Vector3::Z` is used instead
    /// * if `up` is zero, `Vector3::Y` is used instead
    /// * if the resulting forward direction is parallel with `up`, an orthogonal vector is used as the "right" direction
    #[inline]
    #[must_use]
    pub fn looking_at(mut self, target: Vector3, up: Vector3) -> Self {
        self.look_at(target, up);
        self
    }

    /// Returns this [`Transform`] with a new rotation so that [`Transform::forward`]
    /// points in the given `direction` and [`Transform::up`] points towards `up`.
    ///
    /// In some cases it's not possible to construct a rotation. Another axis will be picked in those cases:
    /// * if `direction` is zero, `Vector3::Z` is used instead
    /// * if `up` is zero, `Vector3::Y` is used instead
    /// * if `direction` is parallel with `up`, an orthogonal vector is used as the "right" direction
    #[inline]
    #[must_use]
    pub fn looking_to(mut self, direction: Vector3, up: Vector3) -> Self {
        self.look_to(direction, up);
        self
    }

    /// Returns this [`Transform`] with a new translation.
    #[inline]
    #[must_use]
    pub const fn with_translation(mut self, translation: Vector3) -> Self {
        self.translation = translation;
        self
    }

    /// Returns this [`Transform`] with a new rotation.
    #[inline]
    #[must_use]
    pub const fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// Returns this [`Transform`] with a new scale.
    #[inline]
    #[must_use]
    pub const fn with_scale(mut self, scale: Vector3) -> Self {
        self.scale = scale;
        self
    }

    /// Returns the 3d affine transformation matrix from this transforms translation,
    /// rotation, and scale.
    #[inline]
    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Returns the 3d affine transformation matrix from this transforms translation,
    /// rotation, and scale.
    #[inline]
    pub fn compute_affine(&self) -> Affine3A {
        Affine3A::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Get the unit vector in the local `X` direction.
    #[inline]
    pub fn local_x(&self) -> Vector3 {
        self.rotation * Vector3::X
    }

    /// Equivalent to [`-local_x()`][Transform::local_x()]
    #[inline]
    pub fn left(&self) -> Vector3 {
        -self.local_x()
    }

    /// Equivalent to [`local_x()`][Transform::local_x()]
    #[inline]
    pub fn right(&self) -> Vector3 {
        self.local_x()
    }

    /// Get the unit vector in the local `Y` direction.
    #[inline]
    pub fn local_y(&self) -> Vector3 {
        self.rotation * Vector3::Y
    }

    /// Equivalent to [`local_y()`][Transform::local_y]
    #[inline]
    pub fn up(&self) -> Vector3 {
        self.local_y()
    }

    /// Equivalent to [`-local_y()`][Transform::local_y]
    #[inline]
    pub fn down(&self) -> Vector3 {
        -self.local_y()
    }

    /// Get the unit vector in the local `Z` direction.
    #[inline]
    pub fn local_z(&self) -> Vector3 {
        self.rotation * Vector3::Z
    }

    /// Equivalent to [`-local_z()`][Transform::local_z]
    #[inline]
    pub fn forward(&self) -> Vector3 {
        -self.local_z()
    }

    /// Equivalent to [`local_z()`][Transform::local_z]
    #[inline]
    pub fn back(&self) -> Vector3 {
        self.local_z()
    }

    /// Rotates this [`Transform`] by the given rotation.
    ///
    /// If this [`Transform`] has a parent, the `rotation` is relative to the rotation of the parent.
    ///
    /// # Examples
    ///
    /// - [`3d_rotation`]
    ///
    /// [`3d_rotation`]: https://github.com/bevyengine/bevy/blob/latest/examples/transforms/3d_rotation.rs
    #[inline]
    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
    }

    /// Rotates this [`Transform`] around the given `axis` by `angle` (in radians).
    ///
    /// If this [`Transform`] has a parent, the `axis` is relative to the rotation of the parent.
    #[inline]
    pub fn rotate_axis(&mut self, axis: Vector3, angle: f32) {
        self.rotate(Quat::from_axis_angle(axis, angle));
    }

    /// Rotates this [`Transform`] around the `X` axis by `angle` (in radians).
    ///
    /// If this [`Transform`] has a parent, the axis is relative to the rotation of the parent.
    #[inline]
    pub fn rotate_x(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_x(angle));
    }

    /// Rotates this [`Transform`] around the `Y` axis by `angle` (in radians).
    ///
    /// If this [`Transform`] has a parent, the axis is relative to the rotation of the parent.
    #[inline]
    pub fn rotate_y(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_y(angle));
    }

    /// Rotates this [`Transform`] around the `Z` axis by `angle` (in radians).
    ///
    /// If this [`Transform`] has a parent, the axis is relative to the rotation of the parent.
    #[inline]
    pub fn rotate_z(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_z(angle));
    }

    /// Rotates this [`Transform`] by the given `rotation`.
    ///
    /// The `rotation` is relative to this [`Transform`]'s current rotation.
    #[inline]
    pub fn rotate_local(&mut self, rotation: Quat) {
        self.rotation *= rotation;
    }

    /// Rotates this [`Transform`] around its local `axis` by `angle` (in radians).
    #[inline]
    pub fn rotate_local_axis(&mut self, axis: Vector3, angle: f32) {
        self.rotate_local(Quat::from_axis_angle(axis, angle));
    }

    /// Rotates this [`Transform`] around its local `X` axis by `angle` (in radians).
    #[inline]
    pub fn rotate_local_x(&mut self, angle: f32) {
        self.rotate_local(Quat::from_rotation_x(angle));
    }

    /// Rotates this [`Transform`] around its local `Y` axis by `angle` (in radians).
    #[inline]
    pub fn rotate_local_y(&mut self, angle: f32) {
        self.rotate_local(Quat::from_rotation_y(angle));
    }

    /// Rotates this [`Transform`] around its local `Z` axis by `angle` (in radians).
    #[inline]
    pub fn rotate_local_z(&mut self, angle: f32) {
        self.rotate_local(Quat::from_rotation_z(angle));
    }

    /// Translates this [`Transform`] around a `point` in space.
    ///
    /// If this [`Transform`] has a parent, the `point` is relative to the [`Transform`] of the parent.
    #[inline]
    pub fn translate_around(&mut self, point: Vector3, rotation: Quat) {
        self.translation = point + rotation * (self.translation - point);
    }

    /// Rotates this [`Transform`] around a `point` in space.
    ///
    /// If this [`Transform`] has a parent, the `point` is relative to the [`Transform`] of the parent.
    #[inline]
    pub fn rotate_around(&mut self, point: Vector3, rotation: Quat) {
        self.translate_around(point, rotation);
        self.rotate(rotation);
    }

    /// Rotates this [`Transform`] so that [`Transform::forward`] points towards the `target` position,
    /// and [`Transform::up`] points towards `up`.
    ///
    /// In some cases it's not possible to construct a rotation. Another axis will be picked in those cases:
    /// * if `target` is the same as the transform translation, `Vector3::Z` is used instead
    /// * if `up` is zero, `Vector3::Y` is used instead
    /// * if the resulting forward direction is parallel with `up`, an orthogonal vector is used as the "right" direction
    #[inline]
    pub fn look_at(&mut self, target: Vector3, up: Vector3) {
        self.look_to(target - self.translation, up);
    }

    /// Rotates this [`Transform`] so that [`Transform::forward`] points in the given `direction`
    /// and [`Transform::up`] points towards `up`.
    ///
    /// In some cases it's not possible to construct a rotation. Another axis will be picked in those cases:
    /// * if `direction` is zero, `Vector3::NEG_Z` is used instead
    /// * if `up` is zero, `Vector3::Y` is used instead
    /// * if `direction` is parallel with `up`, an orthogonal vector is used as the "right" direction
    #[inline]
    pub fn look_to(&mut self, direction: Vector3, up: Vector3) {
        let back = -direction.try_normalize().unwrap_or(Vector3::NEG_Z);
        let up = up.try_normalize().unwrap_or(Vector3::Y);
        let right = up
            .cross(back)
            .try_normalize()
            .unwrap_or_else(|| up.any_orthonormal_vector());
        let up = back.cross(right);
        self.rotation = Quat::from_mat3(&Mat3::from_cols(right, up, back));
    }

    /// Multiplies `self` with `transform` component by component, returning the
    /// resulting [`Transform`]
    #[inline]
    #[must_use]
    pub fn mul_transform(&self, transform: Transform) -> Self {
        let translation = self.transform_point(transform.translation);
        let rotation = self.rotation * transform.rotation;
        let scale = self.scale * transform.scale;
        Transform {
            translation,
            rotation,
            scale,
        }
    }

    /// Transforms the given `point`, applying scale, rotation and translation.
    ///
    /// If this [`Transform`] has a parent, this will transform a `point` that is
    /// relative to the parent's [`Transform`] into one relative to this [`Transform`].
    ///
    /// If this [`Transform`] does not have a parent, this will transform a `point`
    /// that is in global space into one relative to this [`Transform`].
    ///
    /// If you want to transform a `point` in global space to the local space of this [`Transform`],
    /// consider using [`GlobalTransform::transform_point()`] instead.
    #[inline]
    pub fn transform_point(&self, mut point: Vector3) -> Vector3 {
        point = self.scale * point;
        point = self.rotation * point;
        point += self.translation;
        point
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// The transform is expected to be non-degenerate and without shearing, or the output
/// will be invalid.
impl From<GlobalTransform> for Transform {
    fn from(transform: GlobalTransform) -> Self {
        transform.compute_transform()
    }
}

impl Mul<Transform> for Transform {
    type Output = Transform;

    fn mul(self, transform: Transform) -> Self::Output {
        self.mul_transform(transform)
    }
}

impl Mul<Vector3> for Transform {
    type Output = Vector3;

    fn mul(self, value: Vector3) -> Self::Output {
        self.transform_point(value)
    }
}