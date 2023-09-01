use crate::{
    controllers::Controller,
    events::VipEvent,
    renderer::{color::hex_to_rgba, AnyRenderable, Renderables},
    shapes::{Circle, CircleOptions},
};
use palette::rgb::LinSrgba;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg_attr(wasm, wasm_bindgen(getter_with_clone))]
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

// NotNan is needed around floats to make this enum
// hashable, which is required for the event manager
// to find the correct controller to send events to.
#[derive(Debug, Clone, Copy)]
pub enum GazeEvent {
    ChangePosition { x: f32, y: f32 },
}

#[derive(Debug, SmartDefault)]
pub struct Gaze {
    #[default(cgmath::Point2::new(0.0, 0.0))]
    position: cgmath::Point2<f32>,
    renderables: Renderables,
    should_update: bool,
}

impl Controller<VipEvent> for Gaze {
    fn handle(&mut self, event: VipEvent) {
        match event {
            VipEvent::Gaze(event) => match event {
                GazeEvent::ChangePosition { x, y } => {
                    self.should_update = true;
                    self.set_position(x, y)
                }
            },
        }
    }

    fn renderables(&self) -> &Renderables {
        &self.renderables
    }

    fn should_update(&self) -> bool {
        self.should_update
    }
}

impl Gaze {
    pub fn new(options: GazeOptions) -> Self {
        let circle = Circle::new(CircleOptions {
            radius: options.radius.unwrap_or_default(),
            border_size: options.border.unwrap_or_default(),
            border_color: hex_to_rgba(&options.color.unwrap_or_default()).unwrap_or_default(),
            color: LinSrgba::new(0.0, 0.0, 0.0, 0.0),
            alpha: 1.0,
        });

        Self {
            renderables: vec![AnyRenderable::Circle(circle)],
            ..Default::default()
        }
    }

    fn set_position(&mut self, x: f32, y: f32) {
        use log::info;

        self.position = cgmath::Point2::new(x, y);

        for renderable in self.renderables.iter_mut() {
            // @TODO refactor as enums
            match renderable {
                AnyRenderable::Circle(circle) => {
                    circle.set_position(self.position);
                    info!("from gaze controller, set circle pos: x: {}, y: {}", &x, &y);
                }
            }
        }
    }
}
