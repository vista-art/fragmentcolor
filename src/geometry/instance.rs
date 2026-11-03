//! `Instance`: the named properties a shader reads per instance.

use super::vertex::Vertex;
use super::vertex_data::VertexData;
use lsp_doc::lsp_doc;
use std::collections::HashMap;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Clone, Debug, Default)]
#[lsp_doc("docs/api/geometry/instance/instance.md")]
pub struct Instance {
    pub(crate) properties: HashMap<String, VertexData>,
    pub(crate) prop_locations: HashMap<String, u32>,
    pub(crate) next_location: u32,
}

crate::impl_fc_kind!(Instance, "Instance");

impl Instance {
    #[lsp_doc("docs/api/geometry/instance/new.md")]
    pub fn new() -> Self {
        Self::default()
    }

    #[lsp_doc("docs/api/geometry/instance/set.md")]
    #[must_use = "set returns the updated Instance"]
    pub fn set(mut self, key: &str, value: impl Into<VertexData>) -> Self {
        let k = key.to_string();
        if !self.prop_locations.contains_key(&k) {
            self.prop_locations.insert(k.clone(), self.next_location);
            self.next_location = self.next_location.saturating_add(1);
        }
        self.properties.insert(k, value.into());
        self
    }
}

/// An instance from named values, set in key order so attribute locations stay stable.
impl From<HashMap<String, VertexData>> for Instance {
    fn from(values: HashMap<String, VertexData>) -> Self {
        let mut entries: Vec<(String, VertexData)> = values.into_iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
            .into_iter()
            .fold(Instance::new(), |instance, (key, value)| {
                instance.set(&key, value)
            })
    }
}

impl From<Vertex> for Instance {
    fn from(v: Vertex) -> Self {
        v.create_instance()
    }
}
impl From<&Vertex> for Instance {
    fn from(v: &Vertex) -> Self {
        v.create_instance()
    }
}
