use crate::{Pass, PassObject, Renderable};
use lsp_doc::lsp_doc;
use std::sync::Arc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

mod features;

pub mod error;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
#[derive(Debug, Default, Clone)]
#[lsp_doc("docs/api/core/frame/frame.md")]
pub struct Frame {
    pub(crate) passes: Vec<Arc<PassObject>>,
    _dependencies: Vec<(usize, usize)>, // @TODO implement directed acyclic graph
}

#[cfg(wasm)]
crate::impl_tryfrom_owned_via_ref!(
    Frame,
    wasm_bindgen::JsValue,
    crate::frame::error::FrameError
);

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
    type Error = crate::frame::error::FrameError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::Reflect;
        use wasm_bindgen::convert::RefFromWasmAbi;

        let key = wasm_bindgen::JsValue::from_str("__wbg_ptr");
        let ptr = Reflect::get(value, &key).map_err(|_| {
            crate::frame::error::FrameError::Error("Missing __wbg_ptr on Frame".into())
        })?;
        let id = ptr.as_f64().ok_or_else(|| {
            crate::frame::error::FrameError::Error("Invalid __wbg_ptr for Frame".into())
        })? as u32;
        let anchor: <Frame as RefFromWasmAbi>::Anchor =
            unsafe { <Frame as RefFromWasmAbi>::ref_from_abi(id) };
        Ok(anchor.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: A new frame starts empty, then collects passes in order, and exposes them via Renderable.
    #[test]
    fn collects_passes_and_exposes_renderable_view() {
        // Arrange
        let mut frame = Frame::new();
        let p1 = Pass::new("p1");
        let p2 = Pass::new("p2");

        // Act
        frame.add_pass(&p1);
        frame.add_pass(&p2);

        // Assert: internal storage preserves order
        assert_eq!(frame.passes.len(), 2);
        // Assert: Renderable view yields two pass objects
        let v = frame.passes();
        let count = v.into_iter().count();
        assert_eq!(count, 2);
    }
}
