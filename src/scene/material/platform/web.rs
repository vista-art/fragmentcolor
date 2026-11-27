#![cfg(wasm)]

use wasm_bindgen::prelude::*;

use crate::bindings::web::describe;
use crate::errors::WebError;
use crate::{Material, PbrMaterial, Shader, ToonMaterial, UnlitMaterial};

/// The material a JS value holds: a `PbrMaterial`, an `UnlitMaterial`, a `ToonMaterial` or a `Shader`,
/// as `label` takes it.
pub(crate) fn read_material(value: &JsValue, label: &str) -> Result<Material, WebError> {
    if let Ok(material) = PbrMaterial::try_from(value) {
        return Ok(material.into());
    }
    if let Ok(material) = UnlitMaterial::try_from(value) {
        return Ok(material.into());
    }
    if let Ok(material) = ToonMaterial::try_from(value) {
        return Ok(material.into());
    }
    if let Ok(shader) = Shader::try_from(value) {
        return Ok(shader.into());
    }
    Err(WebError::type_error(&format!(
        "{label}: expected a PbrMaterial, an UnlitMaterial, a ToonMaterial or a Shader, got {}",
        describe(value)
    )))
}

/// The object a JS caller holds for `material`: the material object or the `Shader` itself.
pub(crate) fn write_material(material: &Material) -> JsValue {
    match material {
        Material::Pbr(material) => material.clone().into(),
        Material::Unlit(material) => material.clone().into(),
        Material::Toon(material) => material.clone().into(),
        Material::Custom(shader) => shader.clone().into(),
    }
}
