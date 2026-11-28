#![cfg(mobile)]

use lsp_doc::lsp_doc;
use std::sync::Arc;

use crate::bindings::numbers::{read_finite, read_vector};
use crate::errors::platform::mobile::FragmentColorError;
use crate::{AlphaMode, MaterialMap, Shader, ToonMaterial};

#[uniffi::export]
impl ToonMaterial {
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/scene/toon_material/new.md")]
    pub fn new_mobile() -> Arc<Self> {
        Arc::new(ToonMaterial::new())
    }

    #[uniffi::method(name = "shader")]
    #[lsp_doc("docs/api/scene/toon_material/shader.md")]
    pub fn shader_mobile(self: Arc<Self>) -> Arc<Shader> {
        Arc::new(self.shader.clone())
    }

    #[uniffi::method(name = "baseColor")]
    #[lsp_doc("docs/api/scene/toon_material/base_color.md")]
    pub fn base_color_mobile(
        self: Arc<Self>,
        color: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let color = read_vector::<4>(color, "ToonMaterial.baseColor")?;
        Ok(Arc::new(self.base_color(color)?))
    }

    #[uniffi::method(name = "bands")]
    #[lsp_doc("docs/api/scene/toon_material/bands.md")]
    pub fn bands_mobile(self: Arc<Self>, count: u32) -> Result<Arc<Self>, FragmentColorError> {
        Ok(Arc::new(self.bands(count)?))
    }

    #[uniffi::method(name = "highlight")]
    #[lsp_doc("docs/api/scene/toon_material/highlight.md")]
    pub fn highlight_mobile(
        self: Arc<Self>,
        color: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let color = read_vector::<3>(color, "ToonMaterial.highlight")?;
        Ok(Arc::new(self.highlight(color)?))
    }

    #[uniffi::method(name = "highlightSize")]
    #[lsp_doc("docs/api/scene/toon_material/highlight_size.md")]
    pub fn highlight_size_mobile(
        self: Arc<Self>,
        value: f32,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let value = read_finite(value, "ToonMaterial.highlightSize: value")?;
        Ok(Arc::new(self.highlight_size(value)?))
    }

    #[uniffi::method(name = "rim")]
    #[lsp_doc("docs/api/scene/toon_material/rim.md")]
    pub fn rim_mobile(self: Arc<Self>, color: Vec<f32>) -> Result<Arc<Self>, FragmentColorError> {
        let color = read_vector::<3>(color, "ToonMaterial.rim")?;
        Ok(Arc::new(self.rim(color)?))
    }

    #[uniffi::method(name = "rimWidth")]
    #[lsp_doc("docs/api/scene/toon_material/rim_width.md")]
    pub fn rim_width_mobile(self: Arc<Self>, value: f32) -> Result<Arc<Self>, FragmentColorError> {
        let value = read_finite(value, "ToonMaterial.rimWidth: value")?;
        Ok(Arc::new(self.rim_width(value)?))
    }

    #[uniffi::method(name = "alphaCutoff")]
    #[lsp_doc("docs/api/scene/toon_material/alpha_cutoff.md")]
    pub fn alpha_cutoff_mobile(
        self: Arc<Self>,
        value: f32,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let value = read_finite(value, "ToonMaterial.alphaCutoff: value")?;
        Ok(Arc::new(self.alpha_cutoff(value)?))
    }

    #[uniffi::method(name = "alphaMode")]
    #[lsp_doc("docs/api/scene/toon_material/alpha_mode.md")]
    pub fn alpha_mode_mobile(self: Arc<Self>, mode: AlphaMode) -> Arc<Self> {
        Arc::new(self.alpha_mode(mode))
    }

    #[uniffi::method(name = "doubleSided")]
    #[lsp_doc("docs/api/scene/toon_material/double_sided.md")]
    pub fn double_sided_mobile(self: Arc<Self>, value: bool) -> Arc<Self> {
        Arc::new(self.double_sided(value))
    }

    #[uniffi::method(name = "baseColorMap")]
    #[lsp_doc("docs/api/scene/toon_material/base_color_map.md")]
    pub fn base_color_map_mobile(self: Arc<Self>) -> Arc<MaterialMap> {
        Arc::new(self.base_color_map())
    }

    #[uniffi::method(name = "rampMap")]
    #[lsp_doc("docs/api/scene/toon_material/ramp_map.md")]
    pub fn ramp_map_mobile(self: Arc<Self>) -> Arc<MaterialMap> {
        Arc::new(self.ramp_map())
    }
}
