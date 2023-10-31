use crate::scene::{
    node::{Node, NodeId},
    Scene,
};

// What's the difference between ObjectBuilder and EntityBuilder?
//
// EntityBuilder wraps a hecs::EntityBuilder and a MeshId.
//
// ObjectBuilder can have many types, including EntityBuilder.
// This is because the original engine had many types of objects,
// and they all shared the same builder interface.
//
// Because I'm going all-in into ECS pattern, I want to remove the
// other types of objects and let them be components instead.
pub struct ObjectBuilder<'a, T> {
    pub(super) scene: &'a mut Scene,
    pub(super) node: Node,
    pub(super) kind: T,
}

impl ObjectBuilder<'_, ()> {
    pub fn build(&mut self) -> NodeId {
        // The object builder is tightly
        // integrated with the Scene
        self.scene.set_node_id(&mut self.node)
    }
}

// This Builder is actually responsible
// for POSITIONING the object in a Scene
impl<T> ObjectBuilder<'_, T> {
    pub fn parent(&mut self, parent: NodeId) -> &mut Self {
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
