#![cfg(wasm)]

/// Install a panic hook and console logger when running in WASM so browser console shows
/// readable errors instead of a generic "unreachable" trap.
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);
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
