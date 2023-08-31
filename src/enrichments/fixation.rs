use serde::{Deserialize, Serialize};

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg_attr(wasm, wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FixationOptions {
    pub radius: f32,
    pub border: f32,
    pub color: String,
    pub alpha: f32,
}
