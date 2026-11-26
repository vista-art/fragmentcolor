#![cfg(wasm)]

use wasm_bindgen::prelude::*;

use crate::bindings::web::{describe, read_unsigned};
use crate::errors::{ErrorClass, WebError};
use crate::{AlphaMode, Material, PbrMaterial, Shader};

/// The material a JS value holds: a `PbrMaterial` or a `Shader`, as `label` takes it.
pub(crate) fn read_material(value: &JsValue, label: &str) -> Result<Material, WebError> {
    if let Ok(material) = PbrMaterial::try_from(value) {
        return Ok(material.into());
    }
    if let Ok(shader) = Shader::try_from(value) {
        return Ok(shader.into());
    }
    Err(WebError::type_error(&format!(
        "{label}: expected a PbrMaterial or a Shader, got {}",
        describe(value)
    )))
}

/// The object a JS caller holds for `material`: the `PbrMaterial` or the `Shader` itself.
pub(crate) fn write_material(material: &Material) -> JsValue {
    match material {
        Material::Pbr(material) => material.clone().into(),
        Material::Custom(shader) => shader.clone().into(),
    }
}

/// Maps an `AlphaMode` from JS, a number in variant order, to the mode.
/// Anything else is an error, which wasm-bindgen would read as `Opaque`.
pub(crate) fn read_alpha_mode(value: &JsValue) -> Result<AlphaMode, WebError> {
    let code = value.as_f64().ok_or_else(|| {
        WebError::type_error(&format!(
            "mode must be an AlphaMode, got {}",
            describe(value)
        ))
    })?;
    match read_unsigned(code, "mode")? {
        0 => Ok(AlphaMode::Opaque),
        1 => Ok(AlphaMode::Mask),
        2 => Ok(AlphaMode::Blend),
        _ => Err(WebError::new(
            ErrorClass::InvalidInput,
            &format!("{code} is not an AlphaMode"),
        )),
    }
}
