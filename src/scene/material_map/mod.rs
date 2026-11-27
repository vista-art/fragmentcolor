//! `MaterialMap`: one texture map of a material, and the UV set and transform it samples through.
//!
//! Every map samples one vertex UV set through its own `KHR_texture_transform`.
//! The PBR shader keeps them in `material.uv`, an array indexed in `MapSlot` order.

use lsp_doc::lsp_doc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use std::sync::Arc;

use crate::core::UniformInput;
use crate::core::shader::uniforms::texture_maps::MapTexture;
use crate::scene::check_finite;
use crate::scene::material::{skip_missing, write_values};
use crate::texture::{TextureData, TextureError, TextureMeta, TextureSource};
use crate::{MaterialError, Shader, TextureId, TextureInput, UniformData};

mod platform;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/scene/material_map/material_map.md")]
pub struct MaterialMap {
    shader: Shader,
    slot: MapSlot,
    /// The type of the material this map belongs to, as errors name the call.
    owner: &'static str,
}

crate::impl_fc_kind!(MaterialMap, "MaterialMap");
crate::impl_js_bridge!(MaterialMap, crate::PassError);

/// Another map, whose texture the map that takes it samples too.
impl From<&MaterialMap> for TextureData {
    fn from(map: &MaterialMap) -> Self {
        TextureData::Map(map.clone())
    }
}

/// Another map, whose texture the map that takes it samples too.
impl From<&MaterialMap> for TextureInput {
    fn from(map: &MaterialMap) -> Self {
        TextureData::from(map).into()
    }
}

impl MaterialMap {
    /// The handle to the map in `slot` of the `owner` material that draws with `shader`.
    pub(crate) fn new(shader: Shader, slot: MapSlot, owner: &'static str) -> Self {
        Self {
            shader,
            slot,
            owner,
        }
    }

    #[lsp_doc("docs/api/scene/material_map/texture.md")]
    pub fn texture(&self, source: impl Into<TextureInput>) -> Result<Self, MaterialError> {
        self.take_input(source.into())?;
        Ok(self.clone())
    }

    #[lsp_doc("docs/api/scene/material_map/uv.md")]
    pub fn uv(&self, set: u32) -> Result<Self, MaterialError> {
        self.check_uv("uv")?;
        let set = UvSet::try_from(set).map_err(|_| MaterialError::UvSet {
            call: self.name_call("uv"),
            set,
        })?;
        write_values(&self.shader, vec![self.slot.build_set(set)])?;
        Ok(self.clone())
    }

    #[lsp_doc("docs/api/scene/material_map/transform.md")]
    pub fn transform(
        &self,
        offset: [f32; 2],
        scale: [f32; 2],
        rotation: f32,
    ) -> Result<Self, MaterialError> {
        self.check_uv("transform")?;
        let transform = UvTransform::new(offset, scale, rotation);
        let transform = transform.check(&self.name_call("transform"))?;
        write_values(&self.shader, self.slot.build_transform(transform).into())?;
        Ok(self.clone())
    }
}

impl MaterialMap {
    /// The slot this map fills.
    pub(crate) fn slot(&self) -> MapSlot {
        self.slot
    }

    /// The name of `method` on this map, as errors name the call.
    fn name_call(&self, method: &str) -> String {
        format!("{}::{}().{method}", self.owner, self.slot.pick_key())
    }

    /// Sets the texture from `input`: a `Texture` binds as it is, another map shares
    /// its texture, and every other input is a source each Renderer uploads.
    fn take_input(&self, input: TextureInput) -> Result<(), MaterialError> {
        let call = self.name_call("texture");
        input.check_options(&call)?;
        let TextureInput { data, options } = input;
        match data {
            TextureData::CloneOf(texture) => self.bind_id(*texture.id()),
            TextureData::Map(map) => self.share(&map),
            data => {
                let input = TextureInput { data, options };
                let source = TextureSource::try_from(input).map_err(|error| error.name(&call))?;
                self.set_map(source)
            }
        }
    }

    /// Makes this map sample the texture `other` samples, which is a map's or an explicit
    /// `Texture`. A map with no texture of its own leaves nothing to share.
    fn share(&self, other: &MaterialMap) -> Result<(), MaterialError> {
        let object = &other.shader.object;
        let key = other.slot.pick_key();
        if let Some(texture) = object.find_map(key) {
            return self.share_texture(texture);
        }
        let bound = object.bound_textures.get(key).map(|bound| bound.clone());
        let Some(registration) = bound else {
            let message = format!(
                "{}: {}::{key}() has no texture to share",
                self.name_call("texture"),
                other.owner
            );
            return Err(TextureError::InvalidInput(message).into());
        };
        self.bind_id(*registration.id())
    }

    /// Makes this map sample `texture`, another map's, under its id when this slot reads the
    /// source in the same color space, and from a copy of its own in this slot's space otherwise.
    fn share_texture(&self, texture: Arc<MapTexture>) -> Result<(), MaterialError> {
        match texture.read_source().swap_hint(self.slot.pick_format()) {
            Some(source) => self.set_map(source),
            None => {
                let key = self.slot.pick_key();
                skip_missing(key, self.shader.object.share_map(key, texture))
            }
        }
    }

    /// Refuses `method`, `uv` or `transform`, on a map that samples no mesh UVs.
    fn check_uv(&self, method: &str) -> Result<(), MaterialError> {
        if self.slot.reads_uv() {
            return Ok(());
        }
        Err(MaterialError::NoUv {
            call: self.name_call(method),
        })
    }

    /// Sets the UV set and the transform this map samples with, or neither.
    pub(crate) fn set_uv(&self, set: UvSet, transform: UvTransform) -> Result<(), MaterialError> {
        let mut values = vec![self.slot.build_set(set)];
        values.extend(self.slot.build_transform(transform));
        write_values(&self.shader, values)
    }

    /// Binds the explicit texture `id` names to this map as it is, so the slot holds the id
    /// and no source. The caller holds the texture, so its registration stays found.
    fn bind_id(&self, id: TextureId) -> Result<(), MaterialError> {
        let key = self.slot.pick_key();
        let meta = TextureMeta::from(id);
        skip_missing(key, self.shader.object.set(key, UniformData::Texture(meta)))
    }

    /// Sets this map to `source`, which takes the slot's format as a hint when it picked none.
    /// A Mipmap or KTX2 container whose bytes don't read in it, such as a float one, keeps its own.
    pub(crate) fn set_map(&self, mut source: TextureSource) -> Result<(), MaterialError> {
        source.take_hint(self.slot.pick_format());
        let key = self.slot.pick_key();
        skip_missing(key, self.shader.object.set_map(key, source))
    }
}

/// The maps of the built-in shaders. The PBR maps come in the order the `material.uv` arrays
/// keep them, and the unlit and toon shaders have the first one. The toon shader's ramp,
/// last, reads the light level in place of the mesh's UVs, so it has no `material.uv` entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum MapSlot {
    BaseColor,
    MetallicRoughness,
    Normal,
    Occlusion,
    Emissive,
    Ramp,
}

impl MapSlot {
    /// The five maps of the PBR shader.
    pub(crate) const ALL: [Self; 5] = [
        Self::BaseColor,
        Self::MetallicRoughness,
        Self::Normal,
        Self::Occlusion,
        Self::Emissive,
    ];

    /// The uniform the built-in shaders read this slot's map from,
    /// which is also the name of the material call that returns the map.
    pub(crate) fn pick_key(self) -> &'static str {
        match self {
            Self::BaseColor => "base_color_map",
            Self::MetallicRoughness => "metallic_roughness_map",
            Self::Normal => "normal_map",
            Self::Occlusion => "occlusion_map",
            Self::Emissive => "emissive_map",
            Self::Ramp => "ramp_map",
        }
    }

    /// Whether the map samples the mesh's UVs, which every map but the toon ramp does.
    fn reads_uv(self) -> bool {
        self != Self::Ramp
    }

    /// The format a map in this slot takes when the caller picks none,
    /// sRGB for the three color maps and linear for the three that hold data.
    pub(crate) fn pick_format(self) -> crate::TextureFormat {
        match self {
            Self::BaseColor | Self::Emissive | Self::Ramp => crate::TextureFormat::Rgba8UnormSrgb,
            Self::MetallicRoughness | Self::Normal | Self::Occlusion => {
                crate::TextureFormat::Rgba8Unorm
            }
        }
    }

    /// The uniform key of one field of this slot's `material.uv` entry.
    fn format_key(self, field: &str) -> String {
        format!("material.uv[{}].{field}", self as usize)
    }

    /// The three writes of `transform` into this slot's `material.uv` entry.
    pub(crate) fn build_transform(self, transform: UvTransform) -> [(String, UniformInput); 3] {
        [
            (
                self.format_key("offset"),
                UniformInput::from(transform.offset.as_slice()),
            ),
            (
                self.format_key("scale"),
                UniformInput::from(transform.scale.as_slice()),
            ),
            (
                self.format_key("rotation"),
                UniformInput::from([transform.rotation].as_slice()),
            ),
        ]
    }

    /// The write of `set` into this slot's `material.uv` entry.
    fn build_set(self, set: UvSet) -> (String, UniformInput) {
        let number = UniformInput::Number(u32::from(set).into());
        (self.format_key("uv_set"), number)
    }
}

/// The vertex UV set a map samples: `Vertex::UV0` or `Vertex::UV1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum UvSet {
    #[default]
    First,
    Second,
}

/// The number glTF gives the set, the `n` of `TEXCOORD_<n>`.
impl From<UvSet> for u32 {
    fn from(set: UvSet) -> Self {
        match set {
            UvSet::First => 0,
            UvSet::Second => 1,
        }
    }
}

/// The set glTF numbers `index`, which `TEXCOORD_<index>` names. Any other number is
/// returned as the error, for the caller to name.
impl TryFrom<u32> for UvSet {
    type Error = u32;

    fn try_from(index: u32) -> Result<Self, u32> {
        match index {
            0 => Ok(Self::First),
            1 => Ok(Self::Second),
            other => Err(other),
        }
    }
}

/// A `KHR_texture_transform`, which scales a UV, then rotates it, then offsets it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct UvTransform {
    pub(crate) offset: [f32; 2],
    pub(crate) scale: [f32; 2],
    pub(crate) rotation: f32,
}

impl UvTransform {
    /// A transform from its three parts, which `check` refuses unless each number is finite.
    pub(crate) fn new(offset: [f32; 2], scale: [f32; 2], rotation: f32) -> Self {
        Self {
            offset,
            scale,
            rotation,
        }
    }

    /// Refuses a transform with a number that isn't finite, naming the part after `call`.
    pub(crate) fn check(self, call: &str) -> Result<Self, MaterialError> {
        check_finite(call, Some("offset"), &self.offset)?;
        check_finite(call, Some("scale"), &self.scale)?;
        check_finite(call, Some("rotation"), &[self.rotation])?;
        Ok(self)
    }
}

impl Default for UvTransform {
    fn default() -> Self {
        Self {
            offset: [0.0, 0.0],
            scale: [1.0, 1.0],
            rotation: 0.0,
        }
    }
}

#[cfg(test)]
mod tests;
