use std::sync::{Arc, RwLock};

use crate::{
    controllers::Controller,
    events::VipEvent,
    renderer::{color::hex_to_rgba, Renderable, RenderableRefs},
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
    pub name: Option<String>,
    pub radius: Option<f32>,
    pub border: Option<f32>,
    pub color: Option<String>,
    pub alpha: Option<f32>,
}

impl Default for GazeOptions {
    fn default() -> Self {
        Self {
            name: Some("gaze".to_string()),
            radius: Some(0.05),
            border: Some(0.005),
            color: Some("#ff000080".to_string()),
            alpha: Some(1.0),
        }
    }
}

#[derive(Debug, SmartDefault)]
pub struct Gaze {
    #[default("gaze".to_string())]
    name: String,
    #[default(cgmath::Point2::new(0.0, 0.0))]
    position: cgmath::Point2<f32>,
    renderables: RenderableRefs,
}

impl Controller<VipEvent> for Gaze {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle(&mut self, event: VipEvent) {
        let VipEvent {
            controller,
            event,
            args,
        } = event;

        if controller == self.name {
            use log::info;
            match event.as_str() {
                "set_position" => {
                    let [x, y] = args.as_slice() else { return };
                    info!("from gaze controller, set position: x: {}, y: {}", &x, &y);

                    let x = x.parse().expect("Failed to parse x coordinate");
                    let y = y.parse().expect("Failed to parse y coordinate");

                    self.set_position(x, y);
                }
                _ => (),
            }
        }
    }

    fn renderables(&self) -> &RenderableRefs {
        self.renderables.as_ref()
    }
}

impl Gaze {
    pub fn new(options: GazeOptions) -> Self {
        let circle = Circle::new(CircleOptions {
            radius: options.radius.unwrap_or_default(),
            border_size: options.border.unwrap_or_default(),
            border_color: LinSrgba::new(0.0, 0.0, 0.0, 0.0),
            color: hex_to_rgba(&options.color.unwrap_or_default()).unwrap_or_default(),
            alpha: 1.0,
        });

        Self {
            renderables: vec![Arc::new(RwLock::new(Renderable::Circle(circle)))],
            ..Default::default()
        }
    }

    fn set_position(&mut self, x: f32, y: f32) {
        use log::info;

        self.position = cgmath::Point2::new(x, y);

        let renderables = self.renderables.clone();

        for renderable in renderables {
            let mut renderable = &mut *renderable.write().unwrap();

            match &mut renderable {
                Renderable::Circle(ref mut circle) => {
                    circle.set_position(self.position);
                    info!("from gaze controller, set circle pos: x: {}, y: {}", &x, &y);
                }
            }
        }
    }
}
