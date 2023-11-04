use crate::color::Color;
use crate::scene::{node::NodeId, space::Space, ObjectBuilder};
use crate::EntityId;

#[derive(Clone, Copy, Debug)]
pub enum LightType {
    Directional,
    Point,
}

#[derive(Debug)]
pub struct Light {
    pub node: NodeId,
    pub color: Color,
    pub intensity: f32,
    pub variant: LightType,
}

pub struct LightBuilder {
    pub(crate) color: Color,
    pub(crate) intensity: f32,
    pub(crate) variant: LightType,
}

// Note that UNLIKE the Entity Builder, this "subclass"
// contains only light-related information. If we are
// going to go all-in into ECS, Light should be just
// a regular entity containing an Emissive component
impl ObjectBuilder<'_, LightBuilder> {
    pub fn intensity(&mut self, intensity: f32) -> &mut Self {
        self.object.intensity = intensity;
        self
    }

    pub fn color(&mut self, color: Color) -> &mut Self {
        self.object.color = color;
        self
    }

    pub fn build(&mut self) -> EntityId {
        let light = Light {
            node: if self.node.local == Space::default() {
                self.node.parent
            } else {
                self.scene.set_node_id(&mut self.node)
            },
            color: self.object.color,
            intensity: self.object.intensity,
            variant: self.object.variant,
        };
        let mut builder = hecs::EntityBuilder::new();
        let light_entity = builder.add(light).build();
        self.scene.add(light_entity)
    }
}
