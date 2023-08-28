use crate::renderer::{Renderable, Uniform};
use crate::uniform;
use palette::rgb::LinSrgba;
use smart_default::SmartDefault;
use std::cell::RefCell;
use std::sync::Arc;

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
    uniform: Arc<RefCell<CircleUniform>>,
    // figure how to clone these
    // buffer: Box<wgpu::Buffer>,
    // bind_group: Box<wgpu::BindGroup>,
    // bind_group_layout: Box<wgpu::BindGroupLayout>,
}

impl Renderable for Circle {
    type U = CircleUniform;

    fn label(&self) -> String {
        "Circle".to_string()
    }

    fn uniform(&self) -> Arc<RefCell<CircleUniform>> {
        self.uniform.clone()
    }

    fn update(&self) {
        self.uniform.borrow_mut().update(self);
    }

    fn buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let uniform = self.uniform.borrow();
        let buffer = uniform.buffer(device);
        buffer
    }

    // fn bind_group(&self, device: &wgpu::Device) -> &wgpu::BindGroup {
    //     todo!()
    // }

    // fn bind_group_layout(&self, device: &wgpu::Device) -> &wgpu::BindGroupLayout {
    //     todo!()
    // }
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
            uniform: Arc::new(RefCell::new(CircleUniform::default())),
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
    pub fn update(&mut self, circle: &Circle) {
        println!("updating circle uniform from concrete implementation");

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
