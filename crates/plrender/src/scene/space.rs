// @TODO Rename this to Transform
// and change scale to vec3

#[derive(Clone, Debug, PartialEq)]
pub struct Space {
    pub position: glam::Vec3,
    pub scale: f32,
    pub orientation: glam::Quat,
}

impl Default for Space {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            scale: 1.0,
            orientation: glam::Quat::IDENTITY,
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

    fn to_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::splat(self.scale),
            self.orientation,
            self.position,
        )
    }
}

#[derive(Debug)]
pub struct RawSpace {
    pub pos_scale: [f32; 4],
    pub rot: [f32; 4],
}

impl From<Space> for RawSpace {
    fn from(s: Space) -> Self {
        Self {
            pos_scale: [s.position.x, s.position.y, s.position.z, s.scale],
            rot: s.orientation.into(),
        }
    }
}

impl RawSpace {
    pub fn to_space(&self) -> Space {
        Space {
            position: glam::Vec3::new(self.pos_scale[0], self.pos_scale[1], self.pos_scale[2]),
            scale: self.pos_scale[3],
            orientation: glam::Quat::from_array(self.rot),
        }
    }

    pub fn inverse_matrix(&self) -> mint::ColumnMatrix4<f32> {
        self.to_space().inverse().to_matrix().into()
    }
}
