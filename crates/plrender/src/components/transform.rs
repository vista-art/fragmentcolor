use crate::scene::node::NodeId;
use std::ops;

#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}

impl Transform {
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
            scale,
            rotation,
        }
    }

    fn to_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

#[derive(Debug)]
pub struct LocalsUniform {
    pub position: [f32; 4],
    pub rotation: [f32; 4],
    pub scale: [f32; 4],
}

impl From<Transform> for LocalsUniform {
    fn from(space: Transform) -> Self {
        Self {
            position: [space.position.x, space.position.y, space.position.z, 1.0],
            scale: [space.scale.x, space.scale.y, space.scale.z, 1.0],
            rotation: space.rotation.into(),
        }
    }
}

impl LocalsUniform {
    pub fn to_transform(&self) -> Transform {
        Transform {
            position: glam::Vec3::new(self.position[0], self.position[1], self.position[2]),
            scale: glam::Vec3::new(self.scale[0], self.scale[1], self.scale[2]),
            rotation: glam::Quat::from_array(self.rotation),
        }
    }

    pub fn inverse_matrix(&self) -> mint::ColumnMatrix4<f32> {
        self.to_transform().inverse().to_matrix().into()
    }
}

pub struct GlobalTransforms {
    pub transforms: Box<[LocalsUniform]>,
}

impl ops::Index<NodeId> for GlobalTransforms {
    type Output = LocalsUniform;
    fn index(&self, node: NodeId) -> &LocalsUniform {
        &self.transforms[node.0 as usize]
    }
}
