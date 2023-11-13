use crate::{
    components::Color,
    scene::macros::has_node_id,
    scene::{node::NodeId, SceneObject},
};

#[derive(Clone, Copy, Debug)]
pub enum LightType {
    Directional,
    Point,
}

#[derive(Debug, Clone)]
pub struct Light {
    pub node_id: NodeId,
    pub color: Color,
    pub intensity: f32,
    pub variant: LightType,
}

has_node_id!(Light);

impl SceneObject<Light> {
    pub fn set_intensity(&mut self, intensity: f32) -> &mut Self {
        self.object.intensity = intensity;
        self
    }

    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.object.color = color;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LightOptions {
    color: Color,
    intensity: f32,
    variant: LightType,
}

impl Light {
    pub fn new(options: LightOptions) -> Self {
        Light {
            node_id: NodeId::root(),
            color: options.color,
            intensity: options.intensity,
            variant: options.variant,
        }
    }

    pub fn set_intensity(&mut self, intensity: f32) -> &mut Self {
        self.intensity = intensity;
        self
    }

    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }
}
