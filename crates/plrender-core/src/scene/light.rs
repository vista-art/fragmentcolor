use crate::color::Color;
use crate::scene::{node::NodeRef, space::Space, ObjectBuilder};

#[derive(Clone, Copy, Debug)]
pub enum LightKind {
    Directional,
    Point,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LightRef(pub u32);

#[derive(Debug)]
pub struct Light {
    pub node: NodeRef,
    pub color: Color,
    pub intensity: f32,
    pub kind: LightKind,
}

pub struct LightBuilder {
    pub(super) color: Color,
    pub(super) intensity: f32,
    pub(super) kind: LightKind,
}

impl ObjectBuilder<'_, LightBuilder> {
    pub fn intensity(&mut self, intensity: f32) -> &mut Self {
        self.kind.intensity = intensity;
        self
    }

    pub fn color(&mut self, color: Color) -> &mut Self {
        self.kind.color = color;
        self
    }

    pub fn build(&mut self) -> LightRef {
        let light = Light {
            node: if self.node.local == Space::default() {
                self.node.parent
            } else {
                self.scene.add_node_impl(&mut self.node)
            },
            color: self.kind.color,
            intensity: self.kind.intensity,
            kind: self.kind.kind,
        };
        let index = self.scene.lights.0.len();
        self.scene.lights.0.push(light);
        LightRef(index as u32)
    }
}