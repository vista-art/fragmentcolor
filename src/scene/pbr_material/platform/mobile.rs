#![cfg(mobile)]

use lsp_doc::lsp_doc;
use std::sync::Arc;

use crate::bindings::numbers::{read_finite, read_vector};
use crate::errors::platform::mobile::FragmentColorError;
use crate::{AlphaMode, MaterialMap, PbrMaterial, Shader};

#[uniffi::export]
impl PbrMaterial {
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/scene/pbr_material/new.md")]
    pub fn new_mobile() -> Arc<Self> {
        Arc::new(PbrMaterial::new())
    }

    #[uniffi::method(name = "shader")]
    #[lsp_doc("docs/api/scene/pbr_material/shader.md")]
    pub fn shader_mobile(self: Arc<Self>) -> Arc<Shader> {
        Arc::new(self.shader.clone())
    }

    #[uniffi::method(name = "baseColor")]
    #[lsp_doc("docs/api/scene/pbr_material/base_color.md")]
    pub fn base_color_mobile(
        self: Arc<Self>,
        color: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let color = read_vector::<4>(color, "PbrMaterial.baseColor")?;
        Ok(Arc::new(self.base_color(color)?))
    }

    #[uniffi::method(name = "metallic")]
    #[lsp_doc("docs/api/scene/pbr_material/metallic.md")]
    pub fn metallic_mobile(self: Arc<Self>, value: f32) -> Result<Arc<Self>, FragmentColorError> {
        let value = read_finite(value, "PbrMaterial.metallic: value")?;
        Ok(Arc::new(self.metallic(value)?))
    }

    #[uniffi::method(name = "roughness")]
    #[lsp_doc("docs/api/scene/pbr_material/roughness.md")]
    pub fn roughness_mobile(self: Arc<Self>, value: f32) -> Result<Arc<Self>, FragmentColorError> {
        let value = read_finite(value, "PbrMaterial.roughness: value")?;
        Ok(Arc::new(self.roughness(value)?))
    }

    #[uniffi::method(name = "normalScale")]
    #[lsp_doc("docs/api/scene/pbr_material/normal_scale.md")]
    pub fn normal_scale_mobile(
        self: Arc<Self>,
        value: f32,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let value = read_finite(value, "PbrMaterial.normalScale: value")?;
        Ok(Arc::new(self.normal_scale(value)?))
    }

    #[uniffi::method(name = "occlusionStrength")]
    #[lsp_doc("docs/api/scene/pbr_material/occlusion_strength.md")]
    pub fn occlusion_strength_mobile(
        self: Arc<Self>,
        value: f32,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let value = read_finite(value, "PbrMaterial.occlusionStrength: value")?;
        Ok(Arc::new(self.occlusion_strength(value)?))
    }

    #[uniffi::method(name = "emissive")]
    #[lsp_doc("docs/api/scene/pbr_material/emissive.md")]
    pub fn emissive_mobile(
        self: Arc<Self>,
        factor: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let factor = read_vector::<3>(factor, "PbrMaterial.emissive")?;
        Ok(Arc::new(self.emissive(factor)?))
    }

    #[uniffi::method(name = "alphaCutoff")]
    #[lsp_doc("docs/api/scene/pbr_material/alpha_cutoff.md")]
    pub fn alpha_cutoff_mobile(
        self: Arc<Self>,
        value: f32,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let value = read_finite(value, "PbrMaterial.alphaCutoff: value")?;
        Ok(Arc::new(self.alpha_cutoff(value)?))
    }

    #[uniffi::method(name = "uvTransform")]
    #[lsp_doc("docs/api/scene/pbr_material/uv_transform.md")]
    pub fn uv_transform_mobile(
        self: Arc<Self>,
        offset: Vec<f32>,
        scale: Vec<f32>,
        rotation: f32,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let offset = read_vector::<2>(offset, "PbrMaterial.uvTransform: offset")?;
        let scale = read_vector::<2>(scale, "PbrMaterial.uvTransform: scale")?;
        let rotation = read_finite(rotation, "PbrMaterial.uvTransform: rotation")?;
        Ok(Arc::new(self.uv_transform(offset, scale, rotation)?))
    }

    #[uniffi::method(name = "alphaMode")]
    #[lsp_doc("docs/api/scene/pbr_material/alpha_mode.md")]
    pub fn alpha_mode_mobile(self: Arc<Self>, mode: AlphaMode) -> Arc<Self> {
        Arc::new(self.alpha_mode(mode))
    }

    #[uniffi::method(name = "doubleSided")]
    #[lsp_doc("docs/api/scene/pbr_material/double_sided.md")]
    pub fn double_sided_mobile(self: Arc<Self>, value: bool) -> Arc<Self> {
        Arc::new(self.double_sided(value))
    }

    #[uniffi::method(name = "baseColorMap")]
    #[lsp_doc("docs/api/scene/pbr_material/base_color_map.md")]
    pub fn base_color_map_mobile(self: Arc<Self>) -> Arc<MaterialMap> {
        Arc::new(self.base_color_map())
    }

    #[uniffi::method(name = "metallicRoughnessMap")]
    #[lsp_doc("docs/api/scene/pbr_material/metallic_roughness_map.md")]
    pub fn metallic_roughness_map_mobile(self: Arc<Self>) -> Arc<MaterialMap> {
        Arc::new(self.metallic_roughness_map())
    }

    #[uniffi::method(name = "normalMap")]
    #[lsp_doc("docs/api/scene/pbr_material/normal_map.md")]
    pub fn normal_map_mobile(self: Arc<Self>) -> Arc<MaterialMap> {
        Arc::new(self.normal_map())
    }

    #[uniffi::method(name = "occlusionMap")]
    #[lsp_doc("docs/api/scene/pbr_material/occlusion_map.md")]
    pub fn occlusion_map_mobile(self: Arc<Self>) -> Arc<MaterialMap> {
        Arc::new(self.occlusion_map())
    }

    #[uniffi::method(name = "emissiveMap")]
    #[lsp_doc("docs/api/scene/pbr_material/emissive_map.md")]
    pub fn emissive_map_mobile(self: Arc<Self>) -> Arc<MaterialMap> {
        Arc::new(self.emissive_map())
    }
}
