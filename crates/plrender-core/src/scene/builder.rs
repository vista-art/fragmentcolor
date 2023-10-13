use crate::scene::{
    node::{Node, NodeRef},
    Scene,
};

pub struct ObjectBuilder<'a, T> {
    pub(super) scene: &'a mut Scene,
    pub(super) node: Node,
    pub(super) kind: T,
}

impl ObjectBuilder<'_, ()> {
    pub fn build(&mut self) -> NodeRef {
        self.scene.add_node_impl(&mut self.node)
    }
}

impl<T> ObjectBuilder<'_, T> {
    pub fn parent(&mut self, parent: NodeRef) -> &mut Self {
        self.node.parent = parent;
        self
    }

    //TODO: should we accept `V: Into<mint::...>` here?
    pub fn position(&mut self, position: mint::Vector3<f32>) -> &mut Self {
        self.node.local.position = position.into();
        self
    }

    // @TODO scale should be 3D
    pub fn scale(&mut self, scale: f32) -> &mut Self {
        self.node.local.scale = scale;
        self
    }

    pub fn orientation_around(&mut self, axis: mint::Vector3<f32>, angle_deg: f32) -> &mut Self {
        self.node.local.orientation =
            glam::Quat::from_axis_angle(axis.into(), angle_deg.to_radians());
        self
    }

    pub fn orientation(&mut self, quat: mint::Quaternion<f32>) -> &mut Self {
        self.node.local.orientation = quat.into();
        self
    }

    pub fn look_at(&mut self, target: mint::Vector3<f32>, up: mint::Vector3<f32>) -> &mut Self {
        let affine = glam::Affine3A::look_at_rh(self.node.local.position, target.into(), up.into());
        let (_, rot, _) = affine.inverse().to_scale_rotation_translation();
        // translation here is expected to match `self.node.local.position`
        self.node.local.orientation = rot;

        self
    }
}
