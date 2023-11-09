use crate::scene::transform::Transform;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NodeId(pub u32);

/// A Node represents a position in a Scene space
/// It contains a Transform and a Parent node.
#[derive(Default, Debug, PartialEq)]
pub struct Node {
    pub(super) parent: NodeId,
    pub(super) local: Transform,
}

impl Node {
    pub fn get_position(&self) -> mint::Vector3<f32> {
        self.local.position.into()
    }
    pub fn set_position(&mut self, pos: mint::Vector3<f32>) {
        self.local.position = pos.into();
    }
    /// This function is an alias to post_move.
    pub fn r#move(&mut self, offset: mint::Vector3<f32>) {
        // @TODO (not a priotity, though)
        //       maybe it would be a good idea to cache sequential
        //       transforms and auto detect which variant to use.
        self.post_move(offset)
    }
    pub fn pre_move(&mut self, offset: mint::Vector3<f32>) {
        let other = Transform {
            position: offset.into(),
            scale: glam::Vec3::ONE,
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
    /// This function is an alias to post_rotate
    pub fn rotate(&mut self, axis: mint::Vector3<f32>, angle_deg: f32) {
        self.post_rotate(axis, angle_deg)
    }
    /// Performs a rotation
    pub fn pre_rotate(&mut self, axis: mint::Vector3<f32>, angle_deg: f32) {
        self.local.orientation = self.local.orientation
            * glam::Quat::from_axis_angle(axis.into(), angle_deg.to_radians());
    }
    pub fn post_rotate(&mut self, axis: mint::Vector3<f32>, angle_deg: f32) {
        let other = Transform {
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            orientation: glam::Quat::from_axis_angle(axis.into(), angle_deg.to_radians()),
        };
        self.local = other.combine(&self.local);
    }

    pub fn get_scale(&self) -> glam::Vec3 {
        self.local.scale
    }
    pub fn set_scale(&mut self, scale: mint::Vector3<f32>) {
        self.local.scale = scale.into();
    }
}
