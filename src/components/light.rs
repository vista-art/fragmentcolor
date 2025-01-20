use crate::{components::Color, scene::transform::TransformId};

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
    pub transform_id: TransformId,
}

#[derive(Debug, Clone, Copy)]
pub struct LightOptions {
    pub color: Color,
    pub intensity: f32,
    pub variant: LightType,
}

impl Light {
    pub fn new(options: LightOptions) -> Self {
        Light {
            transform_id: TransformId::root(),
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
