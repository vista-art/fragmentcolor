#![cfg(wasm)]

use lsp_doc::lsp_doc;
use wasm_bindgen::prelude::*;

use crate::{CompareFunction, Mipmap, SamplerOptions, Size, Texture, TextureFormat, TextureId};

#[wasm_bindgen]
impl Mipmap {
    /// Build a chain from `bytes` for `format`. If `size` is undefined / null,
    /// `bytes` is decoded as an image (PNG / JPEG / etc.); if `size` is
    /// provided, `bytes` is treated as raw pixel data already laid out for
    /// `format` at `size`. Pure CPU work — call from a Web Worker (or the
    /// main thread) and pass the result to `renderer.createTexture(chain)`
    /// for the GPU upload.
    ///
    /// `size` accepts the same shapes the rest of the JS API does:
    /// `[w, h]`, `[w, h, d]`, a typed array, or a `Size` object — see
    /// `Size::try_from(&JsValue)` for the full list. Forcing callers to
    /// construct a wasm-bindgen `Size` instance (which has no JS-side
    /// constructor anyway) would break parity with `createStorageTexture`,
    /// `createTextureTarget`, and friends.
    #[wasm_bindgen(js_name = "build")]
    #[lsp_doc("docs/api/texture/mipmap/build.md")]
    pub fn build_js(
        bytes: &JsValue,
        format: TextureFormat,
        size: &JsValue,
    ) -> Result<Mipmap, JsError> {
        let bytes = crate::texture::js_to_texture_bytes(bytes)?;
        let size: Option<Size> = if size.is_undefined() || size.is_null() {
            None
        } else {
            Some(size.try_into()?)
        };
        let input = crate::TextureInput {
            data: crate::TextureData::Bytes(bytes),
            options: crate::TextureOptions {
                size,
                format,
                ..Default::default()
            },
        };
        Ok(Self::build(input)?)
    }

    #[wasm_bindgen(js_name = "format")]
    #[lsp_doc("docs/api/texture/mipmap/format.md")]
    pub fn format_js(&self) -> TextureFormat {
        self.format.into()
    }

    #[wasm_bindgen(js_name = "size")]
    #[lsp_doc("docs/api/texture/mipmap/size.md")]
    pub fn size_js(&self) -> Size {
        let (w, h) = self.size();
        Size::from([w, h])
    }

    #[wasm_bindgen(js_name = "count")]
    #[lsp_doc("docs/api/texture/mipmap/count.md")]
    pub fn count_js(&self) -> u32 {
        self.count() as u32
    }

    /// Return the bytes for a single mip level as a `Uint8Array`. Use
    /// `count()` to discover the valid range. Returns an error if the
    /// requested level is out of range.
    #[wasm_bindgen(js_name = "level")]
    #[lsp_doc("docs/api/texture/mipmap/levels.md")]
    pub fn level_js(&self, index: u32) -> Result<js_sys::Uint8Array, JsError> {
        let levels = self.levels();
        let idx = index as usize;
        if idx >= levels.len() {
            return Err(JsError::new(&format!(
                "level {} out of range; chain has {} levels",
                idx,
                levels.len()
            )));
        }
        Ok(js_sys::Uint8Array::from(levels[idx].as_slice()))
    }
}

#[wasm_bindgen]
impl Texture {
    #[wasm_bindgen(js_name = "id")]
    #[lsp_doc("docs/api/texture/texture/id.md")]
    pub fn id_js(&self) -> TextureId {
        self.id
    }

    #[wasm_bindgen(js_name = "size")]
    #[lsp_doc("docs/api/texture/texture/size.md")]
    pub fn size_js(&self) -> Size {
        self.size()
    }

    #[wasm_bindgen(js_name = "aspect")]
    #[lsp_doc("docs/api/texture/texture/aspect.md")]
    pub fn aspect_js(&self) -> f32 {
        self.aspect()
    }

    #[wasm_bindgen(js_name = "setSamplerOptions")]
    #[lsp_doc("docs/api/texture/texture/set_sampler_options.md")]
    pub fn set_sampler_options_js(&self, options: &JsValue) -> Result<(), JsError> {
        let opts = js_to_sampler_options(options)?;
        self.set_sampler_options(opts);
        Ok(())
    }

    #[wasm_bindgen(js_name = "write")]
    #[lsp_doc("docs/api/texture/texture/write.md")]
    pub fn write_js(&self, data: &JsValue) -> Result<(), JsError> {
        let bytes = crate::texture::js_to_texture_bytes(data)?;
        self.write(&bytes)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "writeRegion")]
    #[lsp_doc("docs/api/texture/texture/write_region.md")]
    pub fn write_region_js(&self, data: &JsValue, region: &JsValue) -> Result<(), JsError> {
        let bytes = crate::texture::js_to_texture_bytes(data)?;
        let r: crate::TextureRegion = region.try_into()?;
        self.write_region(&bytes, r)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "getImage")]
    #[lsp_doc("docs/api/texture/texture/get_image.md")]
    pub async fn get_image_js(&self) -> Result<js_sys::Uint8Array, JsError> {
        let bytes = self
            .get_image()
            .await
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(js_sys::Uint8Array::from(bytes.as_slice()))
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
