#![cfg(wasm)]

use crate::error::WebError;

/// Install a panic hook and console logger when running in WASM so browser console shows
/// readable errors instead of a generic "unreachable" trap.
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);
    // Pre-grow WASM linear memory to reduce mid-frame memory.grow events
    const PREALLOC_BYTES: usize = 64 * 1024 * 1024; // 64 MiB
    let prealloc = vec![0u8; PREALLOC_BYTES];
    drop(prealloc);
}

/// Allow raising/lowering log level at runtime from JS (e.g., for verbose healthchecks)
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_log_level(
    #[wasm_bindgen(unchecked_param_type = "string")] level: &wasm_bindgen::JsValue,
) -> Result<(), WebError> {
    use log::LevelFilter;
    let level = read_text(level, "set_log_level: level")?;
    let lvl = match level.to_ascii_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" | "warning" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };
    log::set_max_level(lvl);
    Ok(())
}

/// Resolves after one turn of the browser event loop (a zero-delay timer).
/// Lets GPU fences advance between device polls, which the WebGL2 backend
/// needs before a buffer map callback can fire.
pub(crate) async fn yield_to_event_loop() {
    use wasm_bindgen::JsCast;
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let callback: &js_sys::Function = resolve.unchecked_ref();
        let scheduled = web_sys::window()
            .map(|window| {
                window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(callback, 0)
                    .is_ok()
            })
            .unwrap_or(false);
        if !scheduled {
            let _ = callback.call0(&wasm_bindgen::JsValue::NULL);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// Reads a JS number as an unsigned 32-bit integer, such as an index or a count.
/// Negative, fractional, NaN and out-of-range values fail with an error that names `what`.
pub(crate) fn read_unsigned(value: f64, what: &str) -> Result<u32, WebError> {
    Ok(super::read_unsigned(value, what)?)
}

/// A number argument. Any other value is a `TypeError` that names `label`, as Python raises one.
pub(crate) fn read_number(value: &wasm_bindgen::JsValue, label: &str) -> Result<f64, WebError> {
    value.as_f64().ok_or_else(|| {
        WebError::type_error(&format!("{label} must be a number, got {value:?}"))
    })
}

/// A number argument for a 32-bit float, such as a roughness or an angle, read as [`read_number`] reads it.
pub(crate) fn read_float(value: &wasm_bindgen::JsValue, label: &str) -> Result<f32, WebError> {
    Ok(read_number(value, label)? as f32)
}

/// A count or index: a `TypeError` for a value that isn't a number, as [`read_number`] gives,
/// and an `InvalidInputError` for a negative, fractional or out-of-range one, as Python raises.
pub(crate) fn read_count(value: &wasm_bindgen::JsValue, label: &str) -> Result<u32, WebError> {
    read_unsigned(read_number(value, label)?, label)
}

/// A string argument. Any other value is a `TypeError` that names `label`.
pub(crate) fn read_text(value: &wasm_bindgen::JsValue, label: &str) -> Result<String, WebError> {
    value.as_string().ok_or_else(|| {
        WebError::type_error(&format!("{label} must be a string, got {value:?}"))
    })
}

/// A boolean argument. Any other value is a `TypeError` that names `label`, so `'yes'` is no `true`.
pub(crate) fn read_flag(value: &wasm_bindgen::JsValue, label: &str) -> Result<bool, WebError> {
    value.as_bool().ok_or_else(|| {
        WebError::type_error(&format!("{label} must be a boolean, got {value:?}"))
    })
}

/// An optional argument, `None` for `undefined` or `null`.
pub(crate) fn read_given(value: Option<wasm_bindgen::JsValue>) -> Option<wasm_bindgen::JsValue> {
    value.filter(|value| !value.is_undefined() && !value.is_null())
}

/// The numbers of a plain or typed array, or `None` for a value that isn't one.
/// An item that isn't a number is a `TypeError` that names `label` and the item.
pub(crate) fn read_list(
    value: &wasm_bindgen::JsValue,
    label: &str,
) -> Result<Option<Vec<f64>>, WebError> {
    use js_sys::{Array, ArrayBuffer};
    if !(Array::is_array(value) || ArrayBuffer::is_view(value)) {
        return Ok(None);
    }
    let items = Array::from(value)
        .iter()
        .enumerate()
        .map(|(index, item)| read_number(&item, &format!("{label}: item {index}")))
        .collect::<Result<Vec<f64>, WebError>>()?;
    Ok(Some(items))
}

/// The numbers of a float vector or matrix from JavaScript: an array or a `Float32Array`, as AGENTS.md
/// rule 6 lists. Any other value, or an item that isn't a number, fails with an error that names `label`.
pub(crate) fn read_vector_items(
    value: &wasm_bindgen::JsValue,
    label: &str,
) -> Result<Vec<f64>, WebError> {
    use js_sys::{Array, Float32Array};
    use wasm_bindgen::JsCast;
    if !(Array::is_array(value) || value.is_instance_of::<Float32Array>()) {
        return Err(WebError::type_error(&format!(
            "{label} takes an array or a Float32Array of numbers, got {value:?}"
        )));
    }
    Array::from(value)
        .iter()
        .enumerate()
        .map(|(index, item)| {
            item.as_f64().ok_or_else(|| {
                WebError::type_error(&format!(
                    "{label}: item {index} must be a number, got {item:?}"
                ))
            })
        })
        .collect()
}

/// The numbers of a float vector of any length, each checked by [`super::vector::read_floats`].
pub(crate) fn read_floats(
    value: &wasm_bindgen::JsValue,
    label: &str,
) -> Result<Vec<f32>, WebError> {
    let items = read_vector_items(value, label)?;
    Ok(super::vector::read_floats(items, label)?)
}

/// Exactly `N` numbers of a float vector, such as a position, each checked by [`super::vector::read_floats`].
pub(crate) fn read_vector<const N: usize>(
    value: &wasm_bindgen::JsValue,
    label: &str,
) -> Result<[f32; N], WebError> {
    let items = read_vector_items(value, label)?;
    Ok(super::vector::read_vector::<N>(items, label)?)
}

/// A color from `[r, g, b]` or `[r, g, b, a]`, each channel checked by [`super::vector::read_floats`].
pub(crate) fn read_color(
    value: &wasm_bindgen::JsValue,
    label: &str,
) -> Result<crate::Color, WebError> {
    let items = read_vector_items(value, label)?;
    Ok(super::vector::read_color(items, label)?)
}

/// Reads a plain array of numbers or a numeric typed array.
/// Returns `None` for any other value, or when an item is not a number.
pub(crate) fn read_numbers(value: &wasm_bindgen::JsValue) -> Option<Vec<f64>> {
    use wasm_bindgen::JsCast;
    let is_list = js_sys::Array::is_array(value)
        || (js_sys::ArrayBuffer::is_view(value) && !value.is_instance_of::<js_sys::DataView>());
    if !is_list {
        return None;
    }
    js_sys::Array::from(value)
        .iter()
        .map(|item| item.as_f64())
        .collect()
}

/// Reads an array whose items are each an array or typed array of numbers,
/// such as the columns of a matrix. Returns `None` for any other value.
pub(crate) fn read_rows(value: &wasm_bindgen::JsValue) -> Option<Vec<Vec<f64>>> {
    if !js_sys::Array::is_array(value) {
        return None;
    }
    js_sys::Array::from(value)
        .iter()
        .map(|row| read_numbers(&row))
        .collect()
}

/// Reads the bytes of an `ArrayBuffer`, or the bytes a view such as a `DataView` or
/// a typed array covers. Returns `None` for any other value.
pub(crate) fn read_bytes(value: &wasm_bindgen::JsValue) -> Option<Vec<u8>> {
    use js_sys::{ArrayBuffer, Reflect, Uint8Array};
    use wasm_bindgen::JsCast;
    if let Some(buffer) = value.dyn_ref::<ArrayBuffer>() {
        return Some(Uint8Array::new(buffer).to_vec());
    }
    if !ArrayBuffer::is_view(value) {
        return None;
    }
    let field = |name: &str| Reflect::get(value, &name.into()).ok();
    let buffer = field("buffer")?;
    let offset = field("byteOffset")?.as_f64()? as u32;
    let length = field("byteLength")?.as_f64()? as u32;
    Some(Uint8Array::new_with_byte_offset_and_length(&buffer, offset, length).to_vec())
}
