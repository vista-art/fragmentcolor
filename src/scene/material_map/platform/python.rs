#![cfg(python)]

use lsp_doc::lsp_doc;
use pyo3::prelude::*;

use crate::MaterialMap;
use crate::bindings::python::{PyChannels, PyFloat, PyUnsigned};
use crate::core::{TextureKeywords, py_to_texture_spec};
use crate::texture::{PySamplerOptions, PyTextureSource, PyTextureUsage, refuse_options};
use crate::{PySize, TextureData, TextureFormat};

/// What `MaterialMap.texture` takes from Python: another map, or any `create_texture` source.
#[derive(FromPyObject)]
pub enum PyMapInput<'py> {
    #[pyo3(annotation = "MaterialMap")]
    Map(MaterialMap),
    Source(PyTextureSource<'py>),
}

#[pymethods]
impl MaterialMap {
    #[pyo3(
        name = "texture",
        signature = (input, size=None, format=None, mipmaps=None, sampler=None, usage=None)
    )]
    #[lsp_doc("docs/api/scene/material_map/texture.md")]
    pub fn texture_py<'py>(
        &self,
        input: PyMapInput<'py>,
        size: Option<PySize<'py>>,
        format: Option<TextureFormat>,
        mipmaps: Option<bool>,
        sampler: Option<PySamplerOptions<'py>>,
        usage: Option<PyTextureUsage<'py>>,
    ) -> PyResult<Self> {
        let keywords = TextureKeywords::new(size, format, mipmaps, sampler, usage);
        let call = "MaterialMap.texture";
        let source = match input {
            PyMapInput::Map(_) if !keywords.is_empty() => return Err(refuse_options(call).into()),
            PyMapInput::Map(map) => TextureData::Map(map).into(),
            PyMapInput::Source(source) => py_to_texture_spec(source, keywords, call)?,
        };
        Ok(self.texture(source)?)
    }

    #[pyo3(name = "uv")]
    #[lsp_doc("docs/api/scene/material_map/uv.md")]
    pub fn uv_py(&self, set: PyUnsigned<'_>) -> PyResult<Self> {
        Ok(self.uv(set.read("MaterialMap.uv: set")?)?)
    }

    #[pyo3(name = "transform")]
    #[lsp_doc("docs/api/scene/material_map/transform.md")]
    pub fn transform_py(
        &self,
        offset: PyChannels<'_>,
        scale: PyChannels<'_>,
        rotation: PyFloat<'_>,
    ) -> PyResult<Self> {
        let offset = offset.read_array("MaterialMap.transform: offset")?;
        let scale = scale.read_array("MaterialMap.transform: scale")?;
        let rotation = rotation.read_finite("MaterialMap.transform: rotation")?;
        Ok(self.transform(offset, scale, rotation)?)
    }
}
