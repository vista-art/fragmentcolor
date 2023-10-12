use std::ops;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Space {
    position: cgmath::Vector3<f32>,
    scale: cgmath::Vector3<f32>,
    orientation: cgmath::Quaternion<f32>,
}

impl Default for Space {
    fn default() -> Self {
        Self {
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
            orientation: cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0),
        }
    }
}

impl Space {
    pub(super) fn combine(&self, other: &Self) -> Self {
        Self {
            scale: self.scale * other.scale,
            orientation: self.orientation * other.orientation,
            position: self.scale * (self.orientation * other.position) + self.position,
        }
    }

    fn inverse(&self) -> Self {
        let scale = 1.0 / self.scale;
        let orientation = self.orientation.inverse();
        let position = -scale * (orientation * self.position);
        Self {
            position,
            scale,
            orientation,
        }
    }

    fn to_matrix(&self) -> cgmath::Matrix4 {
        cgmath::Matrix4::from_scale_rotation_translation(
            self.scale,
            self.orientation,
            self.position,
        )
    }
}

impl<T> super::ObjectBuilder<'_, T> {
    //TODO: should we accept `V: Into<cgmath::...>` here?
    pub fn position(&mut self, position: cgmath::Vector3<f32>) -> &mut Self {
        self.node.local.position = position.into();
        self
    }

    pub fn scale(&mut self, scale: f32) -> &mut Self {
        self.node.local.scale = scale;
        self
    }

    pub fn orientation_around(&mut self, axis: cgmath::Vector3<f32>, angle_deg: f32) -> &mut Self {
        self.node.local.orientation =
            cgmath::Quaternion::from_axis_angle(axis.into(), angle_deg.to_radians());
        self
    }

    pub fn orientation(&mut self, quat: cgmath::Quaternion<f32>) -> &mut Self {
        self.node.local.orientation = quat.into();
        self
    }

    pub fn look_at(&mut self, target: cgmath::Vector3<f32>, up: cgmath::Vector3<f32>) -> &mut Self {
        let direction = self.node.local.position - target;
        let forward = direction.normalize();
        let right = up.cross(forward).normalize();
        let up = forward.cross(right);
        self.node.local.orientation =
            cgmath::Quaternion::from(cgmath::Matrix3::from_cols(right, up, forward));

        self
    }
}

impl super::Node {
    pub fn get_position(&self) -> cgmath::Vector3<f32> {
        self.local.position.into()
    }
    pub fn set_position(&mut self, pos: cgmath::Vector3<f32>) {
        self.local.position = pos.into();
    }
    pub fn pre_move(&mut self, offset: cgmath::Vector3<f32>) {
        let other = Space {
            position: offset.into(),
            scale: 1.0,
            orientation: cgmath::Quaternion::IDENTITY,
        };
        self.local = other.combine(&self.local);
    }
    pub fn post_move(&mut self, offset: cgmath::Vector3<f32>) {
        self.local.position += cgmath::Vector3::from(offset);
    }

    pub fn get_rotation(&self) -> (cgmath::Vector3<f32>, f32) {
        let (axis, angle) = self.local.orientation.to_axis_angle();
        (axis.into(), angle.to_degrees())
    }
    pub fn set_rotation(&mut self, axis: cgmath::Vector3<f32>, angle_deg: f32) {
        self.local.orientation =
            cgmath::Quaternion::from_axis_angle(axis.into(), angle_deg.to_radians());
    }
    pub fn pre_rotate(&mut self, axis: cgmath::Vector3<f32>, angle_deg: f32) {
        self.local.orientation = self.local.orientation
            * cgmath::Quaternion::from_axis_angle(axis.into(), angle_deg.to_radians());
    }
    pub fn post_rotate(&mut self, axis: cgmath::Vector3<f32>, angle_deg: f32) {
        let other = Space {
            position: cgmath::Vector3::ZERO,
            scale: 1.0,
            orientation: cgmath::Quaternion::from_axis_angle(axis.into(), angle_deg.to_radians()),
        };
        self.local = other.combine(&self.local);
    }

    pub fn get_scale(&self) -> f32 {
        self.local.scale
    }
    pub fn set_scale(&mut self, scale: f32) {
        self.local.scale = scale;
    }
}

#[derive(Debug)]
pub struct SpaceUniform {
    pub position: [f32; 4],
    pub rotation: [f32; 4],
    pub scale: [f32; 4],
}

impl From<Space> for SpaceUniform {
    fn from(s: Space) -> Self {
        Self {
            position: [s.position.x, s.position.y, s.position.z, 0.0],
            rotation: s.orientation.into(),
            scale: [s.scale.x, s.scale.y, s.scale.z, 1.0],
        }
    }
}

impl SpaceUniform {
    pub(super) fn to_space(&self) -> Space {
        Space {
            position: cgmath::Vector3::new(self.position[0], self.position[1], self.position[2]),
            scale: cgmath::Vector3::new(self.scale[0], self.scale[1], self.scale[2]),
            orientation: cgmath::Quaternion::new(
                self.rotation[0],
                self.rotation[1],
                self.rotation[2],
                self.rotation[3],
            ),
        }
    }

    pub fn inverse_matrix(&self) -> cgmath::Matrix4<f32> {
        self.to_space().inverse().to_matrix().into()
    }
}
