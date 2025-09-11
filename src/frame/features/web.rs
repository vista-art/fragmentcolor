#![cfg(wasm)]

use crate::{Frame, Pass};
use lsp_doc::lsp_doc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Frame {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/core/frame/new.md")]
    pub fn new_js() -> Self {
        Self::new()
    }

    #[wasm_bindgen(js_name = "addPass")]
    #[lsp_doc("docs/api/core/frame/add_pass.md")]
    pub fn add_pass_js(&mut self, pass: &Pass) {
        self.add_pass(pass)
    }
}
