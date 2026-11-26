#![cfg(mobile)]

use lsp_doc::lsp_doc;
use std::sync::Arc;

use crate::bindings::numbers::{read_finite, read_vector};
use crate::errors::platform::mobile::FragmentColorError;
use crate::{AlphaMode, MaterialMap, Shader, UnlitMaterial};

#[uniffi::export]
impl UnlitMaterial {
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/scene/unlit_material/new.md")]
    pub fn new_mobile() -> Arc<Self> {
        Arc::new(UnlitMaterial::new())
    }

    #[uniffi::method(name = "shader")]
    #[lsp_doc("docs/api/scene/unlit_material/shader.md")]
    pub fn shader_mobile(self: Arc<Self>) -> Arc<Shader> {
        Arc::new(self.shader.clone())
    }

    #[uniffi::method(name = "baseColor")]
    #[lsp_doc("docs/api/scene/unlit_material/base_color.md")]
    pub fn base_color_mobile(
        self: Arc<Self>,
        color: Vec<f32>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let color = read_vector::<4>(color, "UnlitMaterial.baseColor")?;
        Ok(Arc::new(self.base_color(color)?))
    }

    #[uniffi::method(name = "alphaCutoff")]
    #[lsp_doc("docs/api/scene/unlit_material/alpha_cutoff.md")]
    pub fn alpha_cutoff_mobile(
        self: Arc<Self>,
        value: f32,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let value = read_finite(value, "UnlitMaterial.alphaCutoff: value")?;
        Ok(Arc::new(self.alpha_cutoff(value)?))
    }

    #[uniffi::method(name = "alphaMode")]
    #[lsp_doc("docs/api/scene/unlit_material/alpha_mode.md")]
    pub fn alpha_mode_mobile(self: Arc<Self>, mode: AlphaMode) -> Arc<Self> {
        Arc::new(self.alpha_mode(mode))
    }

    #[uniffi::method(name = "doubleSided")]
    #[lsp_doc("docs/api/scene/unlit_material/double_sided.md")]
    pub fn double_sided_mobile(self: Arc<Self>, value: bool) -> Arc<Self> {
        Arc::new(self.double_sided(value))
    }

    #[uniffi::method(name = "baseColorMap")]
    #[lsp_doc("docs/api/scene/unlit_material/base_color_map.md")]
    pub fn base_color_map_mobile(self: Arc<Self>) -> Arc<MaterialMap> {
        Arc::new(self.base_color_map())
    }
}
