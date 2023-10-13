use crate::scene::space::Space;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NodeRef(pub u32);

#[derive(Default, Debug, PartialEq)]
pub struct Node {
    pub(super) parent: NodeRef,
    pub(super) local: Space,
}

impl Node {
    pub fn get_position(&self) -> mint::Vector3<f32> {
        self.local.position.into()
    }
    pub fn set_position(&mut self, pos: mint::Vector3<f32>) {
        self.local.position = pos.into();
    }
    pub fn pre_move(&mut self, offset: mint::Vector3<f32>) {
        let other = Space {
            position: offset.into(),
            scale: 1.0,
            orientation: glam::Quat::IDENTITY,
        };
        self.local = other.combine(&self.local);
    }
    pub fn post_move(&mut self, offset: mint::Vector3<f32>) {
        self.local.position += glam::Vec3::from(offset);
    }

    pub fn get_rotation(&self) -> (mint::Vector3<f32>, f32) {
        let (axis, angle) = self.local.orientation.to_axis_angle();
        (axis.into(), angle.to_degrees())
    }
    pub fn set_rotation(&mut self, axis: mint::Vector3<f32>, angle_deg: f32) {
        self.local.orientation = glam::Quat::from_axis_angle(axis.into(), angle_deg.to_radians());
    }
    pub fn pre_rotate(&mut self, axis: mint::Vector3<f32>, angle_deg: f32) {
        self.local.orientation = self.local.orientation
            * glam::Quat::from_axis_angle(axis.into(), angle_deg.to_radians());
    }
    pub fn post_rotate(&mut self, axis: mint::Vector3<f32>, angle_deg: f32) {
        let other = Space {
            position: glam::Vec3::ZERO,
            scale: 1.0,
            orientation: glam::Quat::from_axis_angle(axis.into(), angle_deg.to_radians()),
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
