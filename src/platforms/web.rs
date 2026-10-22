#![cfg(wasm)]

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
pub fn set_log_level(level: &str) {
    use log::LevelFilter;
    let lvl = match level.to_ascii_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" | "warning" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };
    log::set_max_level(lvl);
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
pub(crate) fn read_unsigned(value: f64, what: &str) -> Result<u32, wasm_bindgen::JsError> {
    let whole = value.is_finite() && value.fract() == 0.0;
    if whole && (0.0..=u32::MAX as f64).contains(&value) {
        return Ok(value as u32);
    }
    Err(wasm_bindgen::JsError::new(&format!(
        "{what} must be a whole number from 0 to {}, got {value}",
        u32::MAX
    )))
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
