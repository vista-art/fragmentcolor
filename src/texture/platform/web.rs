#![cfg(wasm)]

use js_sys::Reflect;
use lsp_doc::lsp_doc;
use wasm_bindgen::prelude::*;

use crate::platforms::web::read_unsigned;
use crate::{
    CompareFunction, Mipmap, SamplerOptions, Size, Texture, TextureFormat, TextureId, TextureInput,
    TextureOptions,
};

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

    #[wasm_bindgen(js_name = "levels", unchecked_return_type = "Uint8Array[]")]
    #[lsp_doc("docs/api/texture/mipmap/levels.md")]
    pub fn levels_js(&self) -> js_sys::Array {
        let level = |bytes: &Vec<u8>| js_sys::Uint8Array::from(bytes.as_slice());
        self.levels().iter().map(level).collect()
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
    if !value.is_object() {
        return Err(JsError::new(
            "sampler options must be an object { repeatX?, repeatY?, smooth?, compare? }",
        ));
    }
    // Each flag reads its camelCase key, then its snake_case key, which wins.
    let flag = |names: &[&str], default: bool| -> Result<bool, JsError> {
        let mut flag = default;
        for name in names {
            if let Some(field) = read_field(value, name) {
                let message = format!("{name} must be a boolean");
                flag = field.as_bool().ok_or_else(|| JsError::new(&message))?;
            }
        }
        Ok(flag)
    };
    let defaults = SamplerOptions::default();
    let compare = read_field(value, "compare").unwrap_or(JsValue::UNDEFINED);
    Ok(SamplerOptions {
        repeat_x: flag(&["repeatX", "repeat_x"], defaults.repeat_x)?,
        repeat_y: flag(&["repeatY", "repeat_y"], defaults.repeat_y)?,
        smooth: flag(&["smooth"], defaults.smooth)?,
        compare: parse_compare(&compare)?,
    })
}

/// Reads a texture source and its options, as `createTexture` and the Material
/// texture setters take them. An `ImageData` or canvas source carries its own
/// size, which applies when the options don't set one.
pub(crate) fn read_texture_input(
    source: &JsValue,
    options: Option<&JsValue>,
) -> Result<TextureInput, JsError> {
    let mut input = TextureInput::try_from(source)?;
    if let Some(options) = options.filter(|options| !options.is_undefined() && !options.is_null()) {
        let source_size = input.options.size;
        input.options = read_texture_options(options)?;
        input.options.size = input.options.size.or(source_size);
    }
    Ok(input)
}

/// Reads texture options: a bare `TextureFormat`, a bare size such as `[w, h]`,
/// or an object `{ size?, format?, mipmaps?, sampler?, usage? }`.
fn read_texture_options(value: &JsValue) -> Result<TextureOptions, JsError> {
    if let Some(code) = value.as_f64() {
        return Ok(TextureOptions::from(read_format(code)?));
    }
    if let Ok(size) = Size::try_from(value) {
        return Ok(TextureOptions::from(size));
    }
    if !value.is_object() {
        return Err(JsError::new(
            "texture options must be a size, a TextureFormat, or an options object",
        ));
    }
    let mut options = TextureOptions::default();
    if let Some(size) = read_field(value, "size") {
        options.size = Some(Size::try_from(&size)?);
    }
    if let Some(format) = read_field(value, "format") {
        let code = format
            .as_f64()
            .ok_or_else(|| JsError::new("options.format must be a TextureFormat"))?;
        options.format = read_format(code)?;
    }
    if let Some(mipmaps) = read_field(value, "mipmaps") {
        options.mipmaps = mipmaps
            .as_bool()
            .ok_or_else(|| JsError::new("options.mipmaps must be a boolean"))?;
    }
    if let Some(sampler) = read_field(value, "sampler") {
        options.sampler = js_to_sampler_options(&sampler)?;
    }
    if let Some(usage) = read_field(value, "usage") {
        options.usage = Some(read_usage(&usage)?);
    }
    Ok(options)
}

/// Reads a `TextureUsage` or its bits as a number.
pub(crate) fn read_usage(value: &JsValue) -> Result<u32, JsError> {
    let bits = match value.as_f64() {
        Some(bits) => bits,
        None => read_field(value, "bits")
            .and_then(|bits| bits.as_f64())
            .ok_or_else(|| JsError::new("usage must be a TextureUsage or a number"))?,
    };
    read_unsigned(bits, "usage")
}

/// A property of a JS object, or `None` when it is missing, `undefined` or `null`.
fn read_field(object: &JsValue, name: &str) -> Option<JsValue> {
    Reflect::get(object, &name.into())
        .ok()
        .filter(|value| !value.is_undefined() && !value.is_null())
}

/// Maps a `TextureFormat` code from JS to the format. The codes follow the
/// variant order in `src/texture/format.rs`.
fn read_format(code: f64) -> Result<TextureFormat, JsError> {
    let format = match read_unsigned(code, "format")? {
        0 => TextureFormat::R8Unorm,
        1 => TextureFormat::Rg8Unorm,
        2 => TextureFormat::R16Unorm,
        3 => TextureFormat::Rg16Unorm,
        4 => TextureFormat::Rgba8Unorm,
        5 => TextureFormat::Rgba8UnormSrgb,
        6 => TextureFormat::Bgra8Unorm,
        7 => TextureFormat::Rgba16Unorm,
        8 => TextureFormat::Rgba16Float,
        9 => TextureFormat::Rgba32Float,
        10 => TextureFormat::Rgba32Uint,
        11 => TextureFormat::Rgba32Sint,
        12 => TextureFormat::Depth32Float,
        13 => TextureFormat::Rgba,
        14 => TextureFormat::Bgra,
        15 => TextureFormat::Lab,
        16 => TextureFormat::L8,
        _ => return Err(JsError::new(&format!("{code} is not a TextureFormat"))),
    };
    Ok(format)
}
