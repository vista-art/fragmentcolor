use anyhow::Result;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::color::hex_to_rgba;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Serialize, Deserialize, Clone)]
pub struct Gaze {
    pub radius: f32,
    pub border: f32,
    pub color: String,
    pub alpha: f32,
}

impl Default for Gaze {
    fn default() -> Self {
        Self {
            radius: 0.2,
            border: 0.05,
            color: "#ff0000ff".to_string(),
            alpha: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GazeUniform {
    position: [f32; 2],
    radius: f32,
    border: f32,
    color: [f32; 4],
}

impl GazeUniform {
    pub fn new(gaze: Gaze) -> Result<Self> {
        let color = hex_to_rgba(&gaze.color)?;

        Ok(Self {
            position: [0.0, 0.0],
            radius: gaze.radius,
            border: gaze.border,
            color: [color.red, color.green, color.blue, color.alpha * gaze.alpha],
        })
    }

    pub fn set_position(&mut self, position: [f32; 2]) {
        self.position = position;
    }
}
