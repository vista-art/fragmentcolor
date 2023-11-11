use crate::scene::{
    node::{Node, NodeId},
    Scene,
};

// What's the difference between SceneObject and RenderableBuilder?
//
// RenderableBuilder wraps a hecs::EntityBuilder and a MeshId.
// Entities in Baryon ALWAYS have a mesh (unlike hecs which is an id)
//
// SceneObject can have many types, including RenderableBuilder.
// This is because the original engine had many types of objects,
// and they all shared the same builder interface.
//
// Because I'm going all-in into ECS pattern, I want to remove the
// other types of objects and let them be components instead.
pub struct SceneObject<'a, T> {
    pub scene: &'a mut Scene,
    pub node: Node,
    pub object: T,
}

// This has to go away.
// Our design favors adding things explicitly
// to the scene with scene.add()
impl SceneObject<'_, ()> {
    pub fn add_to_scene(&mut self) -> NodeId {
        // The object builder is tightly
        // integrated with the Scene
        self.scene.insert_scene_tree_node(&mut self.node)
    }
}

// This Builder is actually responsible for POSITIONING the object in a Scene
// @TODO consider renaming it and exposing all those methods in the Scene itself
//       those will be the methods a user will use to position objects in the scene
impl<'s, T> SceneObject<'s, T> {
    pub fn new(scene: &'s mut Scene, object: T) -> Self {
        SceneObject {
            scene,
            node: Node::default(),
            object,
        }
    }

    pub fn parent(&mut self, parent: NodeId) -> &mut Self {
        self.node.set_parent(parent);
        self
    }

    pub fn position(&mut self, position: mint::Vector3<f32>) -> &mut Self {
        self.node.set_position(position);
        self
    }

    pub fn scale(&mut self, scale: mint::Vector3<f32>) -> &mut Self {
        self.node.set_scale(scale);
        self
    }

    pub fn rotation_around(&mut self, axis: mint::Vector3<f32>, angle_deg: f32) -> &mut Self {
        self.node.set_rotation(axis, angle_deg);
        self
    }

    pub fn rotation(&mut self, quat: mint::Quaternion<f32>) -> &mut Self {
        self.node.set_rotation_quaternion(quat);
        self
    }

    pub fn look_at(&mut self, target: mint::Vector3<f32>, up: mint::Vector3<f32>) -> &mut Self {
        self.node.look_at(target, up);
        self
    }
}
