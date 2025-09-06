#![cfg(wasm)]

use crate::{Color, Pass, PassInput, Shader};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Pass {
    #[wasm_bindgen(constructor)]
    pub fn new_js(name: &str) -> Self {
        Self::new(name)
    }

    #[wasm_bindgen(js_name = "compute")]
    pub fn compute_js(name: &str) -> Self {
        Self::compute(name)
    }

    #[wasm_bindgen(js_name = "fromShader")]
    pub fn from_shader_js(name: &str, shader: &Shader) -> Self {
        Self::from_shader(name, shader)
    }

    #[wasm_bindgen(js_name = "loadPrevious")]
    pub fn load_previous_py(&self) {
        *self.object.input.write() = PassInput::load();
    }

    #[wasm_bindgen(js_name = "getInput")]
    pub fn get_input_py(&self) -> PassInput {
        self.object.get_input()
    }

    #[wasm_bindgen(js_name = "addShader")]
    pub fn add_shader_py(&self, shader: &Shader) {
        self.object.add_shader(shader);
    }

    #[wasm_bindgen(js_name = "setClearColor")]
    pub fn set_clear_color_py(&self, color: Color) {
        self.object.set_clear_color(color);
    }
}
