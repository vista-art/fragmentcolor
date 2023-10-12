use crate::renderer::{Renderable, RenderableTrait, Uniform, UniformOperations};
use crate::uniform;
use palette::rgb::LinSrgba;
use smart_default::SmartDefault;
use std::sync::{Arc, RwLock};

pub struct Circle {
    pub radius: components::Radius,
    pub border: components::Border,
    pub color: components::Color,
}

impl Circle {}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CircleUniform {
    position: [f32; 2],
    radius: f32,
    border: f32,
    color: [f32; 4],
}

#[derive(Debug, SmartDefault, Clone)]
pub struct Circle {
    #[default(cgmath::Point2::new(0.0, 0.0))]
    pub position: cgmath::Point2<f32>,
    pub radius: f32,
    pub border_size: f32,
    pub color: crate::components::Color,
    pub alpha: f32,
    should_update: bool,
}

impl Circle {
    pub fn new(options: CircleOptions) -> Self {
        let circle = Self {
            position: cgmath::Point2::new(0.0, 0.0),
            radius: options.radius,
            border_size: options.border_size,
            color: options.color,
            alpha: options.alpha,
            should_update: true,
        };

        circle
    }

    pub fn add_to_scene(&self, scene: &mut crate::scene::Scene) {
        scene.add_renderable(self);
    }

    pub fn set_position(&mut self, position: cgmath::Point2<f32>) {
        self.position = position;
        self.should_update = true;
    }
}

impl CircleUniform {
    pub fn update(&mut self, circle: &Circle) {
        use log::info;
        info!("Inside CircleUniform Update: {:?}", circle.position);

        self.position = circle.position.into();
        self.radius = circle.radius;
        self.border = circle.border_size;
        self.color = [
            circle.color.red,
            circle.color.green,
            circle.color.blue,
            circle.color.alpha,
        ];
    }
}
