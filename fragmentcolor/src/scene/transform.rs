use crate::{Quaternion, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TransformId(pub u32);

impl TransformId {
    pub fn root() -> Self {
        Self::default()
    }
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct Transform {
    pub(super) parent: TransformId,
    pub(super) local: LocalTransform,
}

impl Transform {
    pub fn root() -> Self {
        Self::default()
    }

    pub fn parent(&self) -> TransformId {
        self.parent
    }

    pub fn set_parent(&mut self, parent: TransformId) -> &mut Self {
        self.parent = parent;
        self
    }

    pub fn local_transform(&self) -> LocalTransform {
        self.local
    }

    pub fn has_moved(&self) -> bool {
        self.local != LocalTransform::default()
    }

    pub fn position(&self) -> Vec3 {
        self.local.position.into()
    }

    pub fn set_position(&mut self, position: Vec3) -> &mut Self {
        self.local.position = position.into();
        self
    }
    pub fn translate(&mut self, offset: Vec3) -> &mut Self {
        self.local.position += glam::Vec3::from(offset);
        self
    }

    pub fn pre_translate(&mut self, offset: Vec3) {
        let other = LocalTransform {
            position: offset.into(),
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        };
        self.local = other.combine(&self.local);
    }
    pub fn rotation(&self) -> (Vec3, f32) {
        self.rotation_degrees()
    }

    pub fn rotation_degrees(&self) -> (Vec3, f32) {
        let (axis, angle) = self.local.rotation.to_axis_angle();
        (axis.into(), angle.to_degrees())
    }

    pub fn rotation_radians(&self) -> (Vec3, f32) {
        let (axis, angle) = self.local.rotation.to_axis_angle();
        (axis.into(), angle)
    }

    pub fn rotation_quaternion(&self) -> Quaternion {
        self.local.rotation.into()
    }

    pub fn set_rotation(&mut self, axis: Vec3, degrees: f32) -> &mut Self {
        self.set_rotation_degrees(axis, degrees)
    }

    pub fn set_rotation_degrees(&mut self, axis: Vec3, degrees: f32) -> &mut Self {
        self.local.rotation = glam::Quat::from_axis_angle(axis.into(), degrees.to_radians());
        self
    }

    pub fn set_rotation_radians(&mut self, axis: Vec3, radians: f32) -> &mut Self {
        self.local.rotation = glam::Quat::from_axis_angle(axis.into(), radians);
        self
    }

    pub fn set_rotation_quaternion<Q: Into<Quaternion>>(&mut self, quat: Q) -> &mut Self {
        self.local.rotation = quat.into().into();
        self
    }

    pub fn rotate(&mut self, axis: Vec3, degrees: f32) -> &mut Self {
        self.rotate_degrees(axis, degrees)
    }

    pub fn rotate_degrees(&mut self, axis: Vec3, degrees: f32) -> &mut Self {
        self.local.rotation *= glam::Quat::from_axis_angle(axis.into(), degrees.to_radians());

        self
    }

    pub fn rotate_radians(&mut self, axis: Vec3, radians: f32) -> &mut Self {
        self.local.rotation *= glam::Quat::from_axis_angle(axis.into(), radians);

        self
    }

    pub fn pre_rotate(&mut self, axis: Vec3, degrees: f32) -> &mut Self {
        let other = LocalTransform {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::from_axis_angle(axis.into(), degrees.to_radians()),
            scale: glam::Vec3::ONE,
        };
        self.local = other.combine(&self.local);

        self
    }

    pub fn pre_rotate_degrees(&mut self, axis: Vec3, degrees: f32) -> &mut Self {
        let other = LocalTransform {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::from_axis_angle(axis.into(), degrees.to_radians()),
            scale: glam::Vec3::ONE,
        };
        self.local = other.combine(&self.local);

        self
    }

    pub fn pre_rotate_radians(&mut self, axis: Vec3, radians: f32) -> &mut Self {
        let other = LocalTransform {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::from_axis_angle(axis.into(), radians),
            scale: glam::Vec3::ONE,
        };
        self.local = other.combine(&self.local);

        self
    }

    pub fn look_at(&mut self, target: Vec3, up: Vec3) -> &mut Self {
        let affine = glam::Affine3A::look_at_rh(self.local.position, target.into(), up.into());
        let (_, rotation, _) = affine.inverse().to_scale_rotation_translation();
        self.local.rotation = rotation;

        self
    }

    pub fn scale(&self) -> glam::Vec3 {
        self.local.scale
    }

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

    fn to_matrix(self) -> glam::Mat4 {
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
