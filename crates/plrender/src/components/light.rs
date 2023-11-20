use crate::{
    components::Color,
    scene::macros::spatial_object,
    scene::{node::NodeId, SceneObject},
};

#[derive(Clone, Copy, Debug)]
pub enum LightType {
    Directional,
    Point,
}

impl Default for LightType {
    fn default() -> Self {
        Self::Point
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Light {
    pub color: Color,
    pub intensity: f32,
    pub variant: LightType,
    pub(crate) node_id: NodeId,
}

spatial_object!(Light);

#[derive(Debug, Clone, Copy)]
pub struct LightOptions {
    pub color: Color,
    pub intensity: f32,
    pub variant: LightType,
}

impl SceneObject<Light> {
    pub fn set_intensity(&mut self, intensity: f32) -> &mut Self {
        let light = self.object();
        self.add_component(Light { intensity, ..light });

        self
    }

    pub fn set_color(&mut self, color: Color) -> &mut Self {
        let light = self.object();
        self.add_component(Light { color, ..light });

        self
    }
}

impl Light {
    pub fn new(options: LightOptions) -> SceneObject<Self> {
        SceneObject::new(Light {
            node_id: NodeId::root(),
            color: options.color,
            intensity: options.intensity,
            variant: options.variant,
        })
    }
}
