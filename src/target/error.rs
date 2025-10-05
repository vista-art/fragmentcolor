use thiserror::Error;

#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Failed to create a window handle: {0}")]
    WindowHandleError(String),
    #[error("Failed to create a display handle: {0}")]
    DisplayHandleError(String),
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
    #[error("Display Error: {0}")]
    Error(String),
}

#[cfg(wasm)]
impl From<wasm_bindgen::JsValue> for DisplayError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        let error_string = if let Some(s) = value.as_string() {
            s
        } else {
            format!("{:?}", value)
        };
        DisplayError::Error(error_string)
    }
}

#[cfg(wasm)]
impl From<DisplayError> for wasm_bindgen::JsValue {
    fn from(error: DisplayError) -> Self {
        wasm_bindgen::JsValue::from_str(&error.to_string())
    }
}
