use crate::{Pass, PassObject, Renderable};
use lsp_doc::lsp_doc;
use std::sync::Arc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

mod features;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
#[derive(Debug, Default, Clone)]
#[lsp_doc("docs/api/core/frame/frame.md")]
pub struct Frame {
    pub(crate) passes: Vec<Arc<PassObject>>,
    _dependencies: Vec<(usize, usize)>, // @TODO implement directed acyclic graph
}

impl Frame {
    #[lsp_doc("docs/api/core/frame/new.md")]
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            _dependencies: Vec::new(),
        }
    }

    #[lsp_doc("docs/api/core/frame/add_pass.md")]
    pub fn add_pass(&mut self, pass: &Pass) {
        self.passes.push(pass.object.clone());
    }
}

impl Renderable for Frame {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.passes.iter().map(|p| p.as_ref())
    }
}

#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for Frame {
    type Error = crate::error::ShaderError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::Reflect;
        use wasm_bindgen::convert::RefFromWasmAbi;

        let key = wasm_bindgen::JsValue::from_str("__wbg_ptr");
        let ptr = Reflect::get(value, &key).map_err(|_| {
            crate::error::ShaderError::WasmError("Missing __wbg_ptr on Frame".into())
        })?;
        let id = ptr.as_f64().ok_or_else(|| {
            crate::error::ShaderError::WasmError("Invalid __wbg_ptr for Frame".into())
        })? as u32;
        let anchor: <Frame as RefFromWasmAbi>::Anchor =
            unsafe { <Frame as RefFromWasmAbi>::ref_from_abi(id) };
        Ok(anchor.clone())
    }
}
