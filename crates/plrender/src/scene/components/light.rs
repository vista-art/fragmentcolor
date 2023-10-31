use crate::color::Color;
use crate::scene::{node::NodeId, space::Space, ObjectBuilder};

#[derive(Clone, Copy, Debug)]
pub enum LightVariant {
    Directional,
    Point,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LightId(pub u32);

#[derive(Debug)]
pub struct Light {
    pub node: NodeId,
    pub color: Color,
    pub intensity: f32,
    pub variant: LightVariant,
}

pub struct LightBuilder {
    pub(crate) color: Color,
    pub(crate) intensity: f32,
    pub(crate) variant: LightVariant,
}

// Note that UNLIKE the Entity Builder, this "subclass"
// contains only light-related information. If we are
// going to go all-in into ECS, Light should be just
// a regular entity containing an Emissive component
impl ObjectBuilder<'_, LightBuilder> {
    pub fn intensity(&mut self, intensity: f32) -> &mut Self {
        self.kind.intensity = intensity;
        self
    }

    pub fn color(&mut self, color: Color) -> &mut Self {
        self.kind.color = color;
        self
    }

    pub fn build(&mut self) -> LightId {
        let light = Light {
            node: if self.node.local == Space::default() {
                self.node.parent
            } else {
                self.scene.set_node_id(&mut self.node)
            },
            color: self.kind.color,
            intensity: self.kind.intensity,
            variant: self.kind.variant,
        };
        let index = self.scene.lights.len();
        self.scene.lights.push(light);
        LightId(index as u32)
    }
}
