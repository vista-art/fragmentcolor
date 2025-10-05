#![cfg(wasm)]

use lsp_doc::lsp_doc;
use wasm_bindgen::prelude::*;

use crate::{CompareFunction, SamplerOptions, Size, Texture};

#[wasm_bindgen]
impl Texture {
    #[wasm_bindgen(js_name = "size")]
    #[lsp_doc("docs/api/core/texture/size.md")]
    pub fn size_js(&self) -> Size {
        self.size()
    }

    #[wasm_bindgen(js_name = "aspect")]
    #[lsp_doc("docs/api/core/texture/aspect.md")]
    pub fn aspect_js(&self) -> f32 {
        self.aspect()
    }

    #[wasm_bindgen(js_name = "setSamplerOptions")]
    #[lsp_doc("docs/api/core/texture/set_sampler_options.md")]
    pub fn set_sampler_options_js(&self, options: &JsValue) -> Result<(), JsError> {
        let opts = js_to_sampler_options(options)?;
        self.set_sampler_options(opts);
        Ok(())
    }
}

// Helper: parse JS values into CompareFunction. Accepts string names (case-insensitive)
// or numeric enum codes (1..=8). Returns Ok(Some(...)) if present, Ok(None) if undefined/null,
// and Err for type mismatches.
fn parse_compare(value: &JsValue) -> Result<Option<CompareFunction>, JsError> {
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    if let Some(s) = value.as_string() {
        let name = s.to_ascii_lowercase();
        let cf = match name.as_str() {
            "!" | "never" => CompareFunction::Never,
            "<" | "less" => CompareFunction::Less,
            "=" | "equal" => CompareFunction::Equal,
            "<=" | "le" | "lessequal" | "less_equal" => CompareFunction::LessEqual,
            ">" | "greater" => CompareFunction::Greater,
            "!=" | "ne" | "notequal" | "not_equal" | "different" => CompareFunction::NotEqual,
            ">=" | "ge" | "greaterequal" | "greater_equal" => CompareFunction::GreaterEqual,
            "*" | "always" => CompareFunction::Always,
            _ => {
                return Err(JsError::new(
                    "Invalid compare string; expected one of Never, Less, Equal, LessEqual, Greater, NotEqual, GreaterEqual, Always",
                ));
            }
        };
        return Ok(Some(cf));
    }
    if let Some(n) = value.as_f64() {
        let code = n as i32;
        let cf = match code {
            1 => CompareFunction::Never,
            2 => CompareFunction::Less,
            3 => CompareFunction::Equal,
            4 => CompareFunction::LessEqual,
            5 => CompareFunction::Greater,
            6 => CompareFunction::NotEqual,
            7 => CompareFunction::GreaterEqual,
            8 => CompareFunction::Always,
            _ => return Err(JsError::new("Invalid numeric compare code; expected 1..=8")),
        };
        return Ok(Some(cf));
    }
    // Accept a CompareFunction instance if provided (from wasm-bindgen enum)
    // On wasm-bindgen, exported enums are numbers under the hood; above branch should catch it.
    Err(JsError::new("Unsupported compare value type"))
}

pub(crate) fn js_to_sampler_options(value: &JsValue) -> Result<SamplerOptions, JsError> {
    use js_sys::Reflect;

    if value.is_object() {
        let mut opts = SamplerOptions::default();
        // repeatX / repeat_x
        if let Ok(v) = Reflect::get(value, &JsValue::from_str("repeatX")) {
            if !v.is_undefined() && !v.is_null() {
                opts.repeat_x = v
                    .as_bool()
                    .ok_or_else(|| JsError::new("repeatX must be a boolean"))?;
            }
        }
        if let Ok(v) = Reflect::get(value, &JsValue::from_str("repeat_x")) {
            if !v.is_undefined() && !v.is_null() {
                opts.repeat_x = v
                    .as_bool()
                    .ok_or_else(|| JsError::new("repeat_x must be a boolean"))?;
            }
        }
        // repeatY / repeat_y
        if let Ok(v) = Reflect::get(value, &JsValue::from_str("repeatY")) {
            if !v.is_undefined() && !v.is_null() {
                opts.repeat_y = v
                    .as_bool()
                    .ok_or_else(|| JsError::new("repeatY must be a boolean"))?;
            }
        }
        if let Ok(v) = Reflect::get(value, &JsValue::from_str("repeat_y")) {
            if !v.is_undefined() && !v.is_null() {
                opts.repeat_y = v
                    .as_bool()
                    .ok_or_else(|| JsError::new("repeat_y must be a boolean"))?;
            }
        }
        // smooth
        if let Ok(v) = Reflect::get(value, &JsValue::from_str("smooth")) {
            if !v.is_undefined() && !v.is_null() {
                opts.smooth = v
                    .as_bool()
                    .ok_or_else(|| JsError::new("smooth must be a boolean"))?;
            }
        }
        // compare (string or number or null/undefined)
        if let Ok(v) = Reflect::get(value, &JsValue::from_str("compare")) {
            opts.compare = parse_compare(&v)?;
        }
        return Ok(opts);
    }

    Err(JsError::new(
        "setSamplerOptions expects an object { repeatX?, repeatY?, smooth?, compare? }",
    ))
}
