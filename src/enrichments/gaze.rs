use crate::renderer::Renderable;
//use crate::enrichments::Enrichment;
use crate::shapes::circle::Circle;
use crate::{renderer::color::hex_to_rgba, shapes::CircleOptions};
use palette::rgb::LinSrgba;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GazeOptions {
    pub radius: Option<f32>,
    pub border: Option<f32>,
    pub color: Option<String>,
    pub alpha: Option<f32>,
}

impl Default for GazeOptions {
    fn default() -> Self {
        Self {
            radius: Some(0.2),
            border: Some(0.05),
            color: Some("#ff000088".to_string()),
            alpha: Some(1.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GazeEvent {
    ChangePosition { x: u32, y: u32 },
    ChangeNormalizedPosition { x: f32, y: f32 },
}

#[derive(Debug, SmartDefault)]
pub struct Gaze {
    #[default(cgmath::Point2::new(0.0, 0.0))]
    position: cgmath::Point2<f32>,
    circle: Circle,
}

impl Gaze {
    pub fn new(options: GazeOptions) -> Self {
        Self {
            circle: Circle::new(CircleOptions {
                radius: options.radius.unwrap_or_default(),
                border_size: options.border.unwrap_or_default(),
                border_color: hex_to_rgba(&options.color.unwrap_or_default()).unwrap_or_default(),
                color: LinSrgba::new(0.0, 0.0, 0.0, 0.0),
                alpha: 1.0,
            }),
            ..Default::default()
        }
    }

    pub fn handle(&mut self, event: GazeEvent) {
        match event {
            GazeEvent::ChangePosition { x, y } => self.set_position(x, y),
            GazeEvent::ChangeNormalizedPosition { x, y } => self.set_normalized_position(x, y),
        }
    }

    fn set_position(&mut self, _x: u32, _y: u32) {
        todo!("not yet implemented. TODO: inject screen resolution here somehow, or make it available for querying")
    }

    fn set_normalized_position(&mut self, x: f32, y: f32) {
        self.position = cgmath::Point2::new(x, y);
        self.circle.set_position(self.position);
        self.circle.update();
    }

    pub fn renderables(&self) -> Vec<Circle> {
        vec![self.circle.clone()]
    }

    pub fn renderable(&self) -> Circle {
        self.circle.clone()
    }
}
