#![cfg(wasm)]

use crate::error::{ErrorClass, WebError};
use js_sys::Reflect;
use lsp_doc::lsp_doc;
use wasm_bindgen::prelude::*;

use crate::platforms::web::read_unsigned;
use crate::{
    CompareFunction, Mipmap, SamplerOptions, Size, Texture, TextureData, TextureFormat, TextureId,
    TextureInput, TextureOptions,
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
        #[wasm_bindgen(unchecked_param_type = "TextureFormat")] format: &JsValue,
        #[wasm_bindgen(unchecked_optional_param_type = "ArrayLike<number> | Size")] size: Option<
            JsValue,
        >,
    ) -> Result<Mipmap, WebError> {
        let format = read_format(format)?;
        let size = size.filter(|size| !size.is_undefined() && !size.is_null());
        let size: Option<Size> = size.as_ref().map(Size::try_from).transpose()?;
        let (bytes, own) = crate::texture::js_to_texture_bytes(bytes, size, format.into())?;
        let size = size.or(own);
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
    /// requested level is out of range or not a whole number.
    #[wasm_bindgen(js_name = "level")]
    #[lsp_doc("docs/api/texture/mipmap/level.md")]
    pub fn level_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] index: &JsValue,
    ) -> Result<js_sys::Uint8Array, WebError> {
        let number = index
            .as_f64()
            .ok_or_else(|| WebError::type_error("level index must be a number"))?;
        let index = read_unsigned(number, "level index")?;
        Ok(js_sys::Uint8Array::from(self.level(index as usize)?))
    }
}

#[wasm_bindgen]
impl Texture {
    #[wasm_bindgen(js_name = "id")]
    #[lsp_doc("docs/api/texture/texture/id.md")]
    pub fn id_js(&self) -> TextureId {
        *self.id()
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
    pub fn set_sampler_options_js(&self, options: &JsValue) -> Result<(), WebError> {
        let opts = js_to_sampler_options(options)?;
        self.set_sampler_options(opts);
        Ok(())
    }

    #[wasm_bindgen(js_name = "write")]
    #[lsp_doc("docs/api/texture/texture/write.md")]
    pub fn write_js(&self, data: &JsValue) -> Result<(), WebError> {
        let object = self.object();
        let (bytes, _) =
            crate::texture::js_to_texture_bytes(data, Some(object.size()), object.format)?;
        self.write(&bytes)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "writeRegion")]
    #[lsp_doc("docs/api/texture/texture/write_region.md")]
    pub fn write_region_js(&self, data: &JsValue, region: &JsValue) -> Result<(), WebError> {
        if region.is_undefined() || region.is_null() {
            return self.write_js(data);
        }
        let region: crate::TextureRegion = region.try_into()?;
        let [width, height, depth] = region.size;
        let size = Size::new(width, height, Some(depth));
        let format = self.object().format;
        let (bytes, _) = crate::texture::js_to_texture_bytes(data, Some(size), format)?;
        self.write_region(&bytes, region)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "resize")]
    #[lsp_doc("docs/api/texture/texture/resize.md")]
    pub fn resize_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "ArrayLike<number> | Size")] size: &JsValue,
    ) -> Result<(), WebError> {
        let size: Size = size.try_into()?;
        self.resize(size)?;
        Ok(())
    }

    // A plain function, so the call takes its start before it returns the promise.
    #[wasm_bindgen(js_name = "getImage", unchecked_return_type = "Promise<Uint8Array>")]
    #[lsp_doc("docs/api/texture/texture/get_image.md")]
    pub fn get_image_js(&self) -> js_sys::Promise {
        let start = crate::renderer::Start::now(Some(&self.context));
        let texture = self.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            let object = texture.object();
            let bytes = crate::texture::read_pixels_from(&texture.context, &object, start).await?;
            Ok(js_sys::Uint8Array::from(bytes.as_slice()).into())
        })
    }
}

// Helper: parse JS values into CompareFunction. Accepts string names (case-insensitive)
// or numeric enum codes (1..=8). Returns Ok(Some(...)) if present, Ok(None) if undefined/null,
// and Err for type mismatches.
fn parse_compare(value: &JsValue) -> Result<Option<CompareFunction>, WebError> {
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
                return Err(WebError::new(
                    ErrorClass::InvalidInput,
                    "Invalid compare string; expected one of Never, Less, Equal, LessEqual, Greater, NotEqual, GreaterEqual, Always",
                ));
            }
        };
        return Ok(Some(cf));
    }
    if let Some(number) = value.as_f64() {
        let cf = match read_unsigned(number, "compare")? {
            1 => CompareFunction::Never,
            2 => CompareFunction::Less,
            3 => CompareFunction::Equal,
            4 => CompareFunction::LessEqual,
            5 => CompareFunction::Greater,
            6 => CompareFunction::NotEqual,
            7 => CompareFunction::GreaterEqual,
            8 => CompareFunction::Always,
            _ => {
                return Err(WebError::new(
                    ErrorClass::InvalidInput,
                    "Invalid numeric compare code; expected 1..=8",
                ));
            }
        };
        return Ok(Some(cf));
    }
    // Accept a CompareFunction instance if provided (from wasm-bindgen enum)
    // On wasm-bindgen, exported enums are numbers under the hood; above branch should catch it.
    Err(WebError::type_error("Unsupported compare value type"))
}

pub(crate) fn js_to_sampler_options(value: &JsValue) -> Result<SamplerOptions, WebError> {
    if !value.is_object() {
        return Err(WebError::type_error(
            "sampler options must be an object { repeatX?, repeatY?, smooth?, compare? }",
        ));
    }
    // Each flag reads its camelCase key, then its snake_case key, which wins.
    let flag = |names: &[&str], default: bool| -> Result<bool, WebError> {
        let mut flag = default;
        for name in names {
            if let Some(field) = read_field(value, name) {
                let message = format!("{name} must be a boolean");
                flag = field
                    .as_bool()
                    .ok_or_else(|| WebError::type_error(&message))?;
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

/// Reads a texture source and its options, as `createTexture` and the Material setters take them.
/// An `ImageData` or canvas brings its own size and RGBA pixels, so the options may repeat
/// that size and pick an 8-bit RGBA or BGRA format, which stores blue first.
pub(crate) fn read_texture_input(
    source: &JsValue,
    options: Option<&JsValue>,
) -> Result<TextureInput, WebError> {
    let mut input = TextureInput::try_from(source)?;
    if let Some(options) = options.filter(|options| !options.is_undefined() && !options.is_null()) {
        let source_size = input.options.size;
        input.options = read_texture_options(options)?;
        if let (Some(own), TextureData::Bytes(pixels)) = (source_size, &mut input.data) {
            let format = input.options.format.into();
            crate::texture::check_pixels(own, input.options.size, format)?;
            crate::texture::swap_bgra(pixels, format);
        }
        input.options.size = input.options.size.or(source_size);
    }
    Ok(input)
}

/// The keys of a texture options object.
const OPTION_KEYS: [&str; 5] = ["size", "format", "mipmaps", "sampler", "usage"];

/// Reads texture options, and throws for a bare size that fails to read, such as `[1.5, 2]`.
/// The options are a `TextureFormat`, a bare size such as `[w, h]` or `{ width, height }`, or an
/// object `{ size?, format?, mipmaps?, sampler?, usage? }`, which reads as options whenever it holds
/// one of those keys and then refuses a `width`, `height` or `depth`.
fn read_texture_options(value: &JsValue) -> Result<TextureOptions, WebError> {
    if value.as_f64().is_some() {
        return Ok(TextureOptions::from(read_format(value)?));
    }
    let given = |keys: &[&str]| keys.iter().any(|key| read_field(value, key).is_some());
    let options = value.is_object() && given(&OPTION_KEYS);
    if options && given(&["width", "height", "depth"]) {
        return Err(WebError::new(
            ErrorClass::InvalidInput,
            "texture options take the size under `size`, as in { size: [w, h] }",
        ));
    }
    let list = js_sys::Array::is_array(value) || js_sys::ArrayBuffer::is_view(value);
    if !options && (list || given(&["width", "height", "depth"])) {
        return Ok(TextureOptions::from(Size::try_from(value)?));
    }
    if !value.is_object() {
        return Err(WebError::type_error(
            "texture options must be a size, a TextureFormat, or an options object",
        ));
    }
    let mut options = TextureOptions::default();
    if let Some(size) = read_field(value, "size") {
        options.size = Some(Size::try_from(&size)?);
    }
    if let Some(format) = read_field(value, "format") {
        options.format = read_format(&format)?;
    }
    if let Some(mipmaps) = read_field(value, "mipmaps") {
        options.mipmaps = mipmaps
            .as_bool()
            .ok_or_else(|| WebError::type_error("options.mipmaps must be a boolean"))?;
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
pub(crate) fn read_usage(value: &JsValue) -> Result<u32, WebError> {
    let bits = match value.as_f64() {
        Some(bits) => bits,
        None => read_field(value, "bits")
            .and_then(|bits| bits.as_f64())
            .ok_or_else(|| WebError::type_error("usage must be a TextureUsage or a number"))?,
    };
    read_unsigned(bits, "usage")
}

/// A property of a JS object, or `None` when it is missing, `undefined` or `null`.
fn read_field(object: &JsValue, name: &str) -> Option<JsValue> {
    Reflect::get(object, &name.into())
        .ok()
        .filter(|value| !value.is_undefined() && !value.is_null())
}

/// Maps a `TextureFormat` from JS, a number in the variant order of
/// `src/texture/format.rs`, to the format. Anything else is an error,
/// where wasm-bindgen would read `undefined`, `NaN` or a fraction as a code.
pub(crate) fn read_format(value: &JsValue) -> Result<TextureFormat, WebError> {
    let code = value.as_f64().ok_or_else(|| {
        WebError::type_error(&format!("format must be a TextureFormat, got {value:?}"))
    })?;
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
        _ => {
            return Err(WebError::new(
                ErrorClass::InvalidInput,
                &format!("{code} is not a TextureFormat"),
            ));
        }
    };
    Ok(format)
}
