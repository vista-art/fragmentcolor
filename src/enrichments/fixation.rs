use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FixationOptions {
    pub radius: f32,
    pub border: f32,
    pub color: String,
    pub alpha: f32,
}
