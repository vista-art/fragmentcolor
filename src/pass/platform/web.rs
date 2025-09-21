#![cfg(wasm)]

use crate::PassInput;
use crate::{Color, Pass, Shader};
use lsp_doc::lsp_doc;
use wasm_bindgen::JsValue;
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
    pub fn load_previous_js(&self) {
        *self.object.input.write() = PassInput::load();
    }

    #[wasm_bindgen(js_name = "getInput")]
    pub fn get_input_js(&self) -> PassInput {
        self.object.get_input()
    }

    #[wasm_bindgen(js_name = "addShader")]
    pub fn add_shader_js(&self, shader: &Shader) {
        self.object.add_shader(shader);
    }

    #[wasm_bindgen(js_name = "addMesh")]
    #[lsp_doc("docs/api/core/pass/add_mesh.md")]
    pub fn add_mesh_js(&self, mesh: &crate::mesh::Mesh) {
        self.add_mesh(mesh)
    }

    #[wasm_bindgen(js_name = "setClearColor")]
    pub fn set_clear_color_js(&self, color: &JsValue) -> Result<(), JsError> {
        let color: Color = color
            .try_into()
            .map_err(|e: crate::color::error::ColorError| JsError::new(&format!("{e}")))?;
        self.object.set_clear_color(color);
        Ok(())
    }
}
