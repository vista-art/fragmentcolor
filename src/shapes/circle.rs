use crate::renderer::{Renderable, RenderableTrait, Uniform, UniformOperations};
use crate::uniform;
use palette::rgb::LinSrgba;
use smart_default::SmartDefault;
use std::sync::{Arc, RwLock};

pub struct CircleOptions {
    pub radius: f32,
    pub border_size: f32,
    pub border_color: LinSrgba,
    pub color: LinSrgba,
    pub alpha: f32,
}

#[derive(Debug, SmartDefault, Clone)]
pub struct Circle {
    #[default(cgmath::Point2::new(0.0, 0.0))]
    pub position: cgmath::Point2<f32>,
    pub radius: f32,
    pub border_size: f32,
    pub border_color: LinSrgba,
    pub color: LinSrgba,
    pub alpha: f32,
    uniform: Arc<RwLock<Uniform>>,
}

impl RenderableTrait for Circle {
    fn label(&self) -> String {
        "Circle".to_string()
    }

    fn uniform(&self) -> Arc<RwLock<Uniform>> {
        self.uniform.clone()
    }

    fn buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let uniform = self.uniform.read().unwrap();
        let buffer = uniform.buffer(device);
        buffer
    }

    fn update(&self) {
        let any_renderable = Renderable::from(self.to_owned());

        self.uniform.write().unwrap().update(&any_renderable);
    }
}

impl Circle {
    pub fn new(options: CircleOptions) -> Self {
        let circle = Self {
            position: cgmath::Point2::new(0.0, 0.0),
            radius: options.radius,
            border_size: options.border_size,
            border_color: options.border_color,
            color: options.color,
            alpha: options.alpha,
            uniform: Arc::new(RwLock::new(Uniform::Circle(CircleUniform::default()))),
        };

        circle
    }

    pub fn set_position(&mut self, position: cgmath::Point2<f32>) {
        self.position = position;
    }
}

uniform!(CircleUniform for Circle {
    position: [f32; 2],
    radius: f32,
    border: f32,
    color: [f32; 4],
});

impl CircleUniform {
    pub fn update(&mut self, renderable: &Renderable) {
        use log::info;

        match renderable {
            Renderable::Circle(circle) => {
                info!("Inside CircleUniform Update: {:?}", circle.position);
                self.position = circle.position.into();
                self.radius = circle.radius;
                self.border = circle.border_size;
                self.color = [
                    circle.color.red,
                    circle.color.green,
                    circle.color.blue,
                    circle.color.alpha * circle.alpha,
                ];
            }
        }
    }
}
