#![cfg(mobile)]

use lsp_doc::lsp_doc;
use std::sync::Arc;

use crate::Shader;
use crate::bindings::numbers::read_vector;
use crate::error::platform::mobile::FragmentColorError;
use crate::texture::{Texture, TextureInput, TextureInputMobile, TextureOptions};
use crate::{AlphaMode, Material, MaterialError, TextureSlot};

#[uniffi::export]
impl Material {
    #[uniffi::constructor(name = "pbr")]
    #[lsp_doc("docs/api/scene/material/pbr.md")]
    pub fn pbr_mobile() -> Arc<Self> {
        Arc::new(Material::pbr())
    }

    #[uniffi::constructor(name = "custom")]
    #[lsp_doc("docs/api/scene/material/custom.md")]
    pub fn custom_mobile(shader: Arc<Shader>) -> Arc<Self> {
        Arc::new(Material::custom((*shader).clone()))
    }

    #[uniffi::method(name = "shader")]
    #[lsp_doc("docs/api/scene/material/shader.md")]
    pub fn shader_mobile(self: Arc<Self>) -> Arc<Shader> {
        Arc::new(self.shader.clone())
    }

    #[uniffi::method(name = "baseColor")]
    #[lsp_doc("docs/api/scene/material/base_color.md")]
    pub fn base_color_mobile(
        self: Arc<Self>,
        color: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let arr = read_vector::<4>(color, "Material.baseColor")?;
        Ok(Arc::new(self.base_color(arr)))
    }

    #[uniffi::method(name = "metallic")]
    #[lsp_doc("docs/api/scene/material/metallic.md")]
    pub fn metallic_mobile(self: Arc<Self>, value: f32) -> Arc<Self> {
        Arc::new(self.metallic(value))
    }

    #[uniffi::method(name = "roughness")]
    #[lsp_doc("docs/api/scene/material/roughness.md")]
    pub fn roughness_mobile(self: Arc<Self>, value: f32) -> Arc<Self> {
        Arc::new(self.roughness(value))
    }

    #[uniffi::method(name = "normalScale")]
    #[lsp_doc("docs/api/scene/material/normal_scale.md")]
    pub fn normal_scale_mobile(self: Arc<Self>, value: f32) -> Arc<Self> {
        Arc::new(self.normal_scale(value))
    }

    #[uniffi::method(name = "occlusionStrength")]
    #[lsp_doc("docs/api/scene/material/occlusion_strength.md")]
    pub fn occlusion_strength_mobile(self: Arc<Self>, value: f32) -> Arc<Self> {
        Arc::new(self.occlusion_strength(value))
    }

    #[uniffi::method(name = "emissive")]
    #[lsp_doc("docs/api/scene/material/emissive.md")]
    pub fn emissive_mobile(
        self: Arc<Self>,
        factor: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let arr = read_vector::<3>(factor, "Material.emissive")?;
        Ok(Arc::new(self.emissive(arr)))
    }

    #[uniffi::method(name = "alphaCutoff")]
    #[lsp_doc("docs/api/scene/material/alpha_cutoff.md")]
    pub fn alpha_cutoff_mobile(self: Arc<Self>, value: f32) -> Arc<Self> {
        Arc::new(self.alpha_cutoff(value))
    }

    #[uniffi::method(name = "uvTransform")]
    #[lsp_doc("docs/api/scene/material/uv_transform.md")]
    pub fn uv_transform_mobile(
        self: Arc<Self>,
        offset: Vec<f32>,
        scale: Vec<f32>,
        rotation: f32,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let o = read_vector::<2>(offset, "Material.uvTransform: offset")?;
        let s = read_vector::<2>(scale, "Material.uvTransform: scale")?;
        Ok(Arc::new(self.uv_transform(o, s, rotation)))
    }

    #[uniffi::method(name = "textureUv")]
    #[lsp_doc("docs/api/scene/material/texture_uv.md")]
    pub fn texture_uv_mobile(
        self: Arc<Self>,
        slot: TextureSlot,
        set: u32,
    ) -> Result<Arc<Self>, FragmentColorError> {
        Ok(Arc::new(self.texture_uv(slot, set)?))
    }

    #[uniffi::method(name = "textureTransform")]
    #[lsp_doc("docs/api/scene/material/texture_transform.md")]
    pub fn texture_transform_mobile(
        self: Arc<Self>,
        slot: TextureSlot,
        offset: Vec<f32>,
        scale: Vec<f32>,
        rotation: f32,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let o = read_vector::<2>(offset, "Material.textureTransform: offset")?;
        let s = read_vector::<2>(scale, "Material.textureTransform: scale")?;
        Ok(Arc::new(self.texture_transform(slot, o, s, rotation)))
    }

    #[uniffi::method(name = "alphaMode")]
    #[lsp_doc("docs/api/scene/material/alpha_mode.md")]
    pub fn alpha_mode_mobile(self: Arc<Self>, mode: AlphaMode) -> Arc<Self> {
        Arc::new(self.alpha_mode(mode))
    }

    #[uniffi::method(name = "doubleSided")]
    #[lsp_doc("docs/api/scene/material/double_sided.md")]
    pub fn double_sided_mobile(self: Arc<Self>, value: bool) -> Arc<Self> {
        Arc::new(self.double_sided(value))
    }

    #[uniffi::method(name = "baseColorTexture")]
    #[lsp_doc("docs/api/scene/material/base_color_texture.md")]
    pub fn base_color_texture_mobile(self: Arc<Self>, texture: Arc<Texture>) -> Arc<Self> {
        Arc::new(self.base_color_texture(&*texture))
    }

    #[uniffi::method(name = "metallicRoughnessTexture")]
    #[lsp_doc("docs/api/scene/material/metallic_roughness_texture.md")]
    pub fn metallic_roughness_texture_mobile(self: Arc<Self>, texture: Arc<Texture>) -> Arc<Self> {
        Arc::new(self.metallic_roughness_texture(&*texture))
    }

    #[uniffi::method(name = "normalTexture")]
    #[lsp_doc("docs/api/scene/material/normal_texture.md")]
    pub fn normal_texture_mobile(self: Arc<Self>, texture: Arc<Texture>) -> Arc<Self> {
        Arc::new(self.normal_texture(&*texture))
    }

    #[uniffi::method(name = "occlusionTexture")]
    #[lsp_doc("docs/api/scene/material/occlusion_texture.md")]
    pub fn occlusion_texture_mobile(self: Arc<Self>, texture: Arc<Texture>) -> Arc<Self> {
        Arc::new(self.occlusion_texture(&*texture))
    }

    #[uniffi::method(name = "emissiveTexture")]
    #[lsp_doc("docs/api/scene/material/emissive_texture.md")]
    pub fn emissive_texture_mobile(self: Arc<Self>, texture: Arc<Texture>) -> Arc<Self> {
        Arc::new(self.emissive_texture(&*texture))
    }

    /// The texture input form of `baseColorTexture`, uploaded when the scene loads.
    /// Swift and Kotlin reach it through the `baseColorTexture` overloads, which match `createTexture`.
    #[uniffi::method(name = "baseColorSource", default(options = None))]
    #[lsp_doc("docs/api/scene/material/hidden/base_color_source_mobile.md")]
    pub fn base_color_source_mobile(
        self: Arc<Self>,
        input: TextureInputMobile,
        options: Option<TextureOptions>,
    ) -> Arc<Self> {
        Arc::new(self.base_color_texture(texture_input(input, options)))
    }

    /// The texture input form of `metallicRoughnessTexture`, uploaded when the scene loads.
    /// Swift and Kotlin reach it through the `metallicRoughnessTexture` overloads, which match `createTexture`.
    #[uniffi::method(name = "metallicRoughnessSource", default(options = None))]
    #[lsp_doc("docs/api/scene/material/hidden/metallic_roughness_source_mobile.md")]
    pub fn metallic_roughness_source_mobile(
        self: Arc<Self>,
        input: TextureInputMobile,
        options: Option<TextureOptions>,
    ) -> Arc<Self> {
        Arc::new(self.metallic_roughness_texture(texture_input(input, options)))
    }

    /// The texture input form of `normalTexture`, uploaded when the scene loads.
    /// Swift and Kotlin reach it through the `normalTexture` overloads, which match `createTexture`.
    #[uniffi::method(name = "normalSource", default(options = None))]
    #[lsp_doc("docs/api/scene/material/hidden/normal_source_mobile.md")]
    pub fn normal_source_mobile(
        self: Arc<Self>,
        input: TextureInputMobile,
        options: Option<TextureOptions>,
    ) -> Arc<Self> {
        Arc::new(self.normal_texture(texture_input(input, options)))
    }

    /// The texture input form of `occlusionTexture`, uploaded when the scene loads.
    /// Swift and Kotlin reach it through the `occlusionTexture` overloads, which match `createTexture`.
    #[uniffi::method(name = "occlusionSource", default(options = None))]
    #[lsp_doc("docs/api/scene/material/hidden/occlusion_source_mobile.md")]
    pub fn occlusion_source_mobile(
        self: Arc<Self>,
        input: TextureInputMobile,
        options: Option<TextureOptions>,
    ) -> Arc<Self> {
        Arc::new(self.occlusion_texture(texture_input(input, options)))
    }

    /// The texture input form of `emissiveTexture`, uploaded when the scene loads.
    /// Swift and Kotlin reach it through the `emissiveTexture` overloads, which match `createTexture`.
    #[uniffi::method(name = "emissiveSource", default(options = None))]
    #[lsp_doc("docs/api/scene/material/hidden/emissive_source_mobile.md")]
    pub fn emissive_source_mobile(
        self: Arc<Self>,
        input: TextureInputMobile,
        options: Option<TextureOptions>,
    ) -> Arc<Self> {
        Arc::new(self.emissive_texture(texture_input(input, options)))
    }
}

/// A value the material can't use reaches Swift and Kotlin as a render error.
impl From<MaterialError> for FragmentColorError {
    fn from(error: MaterialError) -> Self {
        FragmentColorError::Render(error.to_string())
    }
}

/// A crate texture input from the mobile input and its optional options.
fn texture_input(input: TextureInputMobile, options: Option<TextureOptions>) -> TextureInput {
    TextureInput {
        data: input.into(),
        options: options.unwrap_or_default(),
    }
}
