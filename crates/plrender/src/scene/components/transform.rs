use crate::scene::node::NodeId;
use std::ops;

#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: glam::Vec3,
    pub scale: glam::Vec3,
    pub orientation: glam::Quat,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            orientation: glam::Quat::IDENTITY,
        }
    }
}

impl Transform {
    pub(crate) fn combine(&self, other: &Self) -> Self {
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

    fn to_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.orientation, self.position)
    }
}

#[derive(Debug)]
pub struct LocalTransform {
    pub position: [f32; 4],
    pub scale: [f32; 4],
    pub rotation: [f32; 4],
}

impl From<Transform> for LocalTransform {
    fn from(space: Transform) -> Self {
        Self {
            position: [space.position.x, space.position.y, space.position.z, 1.0],
            scale: [space.scale.x, space.scale.y, space.scale.z, 1.0],
            rotation: space.orientation.into(),
        }
    }
}

impl LocalTransform {
    pub fn to_transform(&self) -> Transform {
        Transform {
            position: glam::Vec3::new(self.position[0], self.position[1], self.position[2]),
            scale: glam::Vec3::new(self.scale[0], self.scale[1], self.scale[2]),
            orientation: glam::Quat::from_array(self.rotation),
        }
    }

    pub fn inverse_matrix(&self) -> mint::ColumnMatrix4<f32> {
        self.to_transform().inverse().to_matrix().into()
    }
}

pub struct GlobalTransforms {
    pub transforms: Box<[LocalTransform]>,
}

impl ops::Index<NodeId> for GlobalTransforms {
    type Output = LocalTransform;
    fn index(&self, node: NodeId) -> &LocalTransform {
        &self.transforms[node.0 as usize]
    }
}
