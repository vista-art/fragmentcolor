//! Cross-target URL fetching helpers.
//!
//! - **Web**: always uses `web_sys::fetch` (the browser's built-in). The
//!   `network` Cargo feature does not affect wasm builds.
//! - **Desktop (Linux / macOS / Windows) with `--features network`**: uses
//!   `ureq` with `native-tls` (system TLS stack).
//! - **Desktop without `network`, and mobile (iOS / Android)**: the fetch
//!   helpers are still exported so call sites compile uniformly, but they
//!   return `NetworkError::feature_disabled()` at runtime. Embedded
//!   registry slugs (the `shaders-*` features) keep working without
//!   network.

#[cfg(wasm)]
use wasm_bindgen_test::__rt::wasm_bindgen::JsValue;

/// Single error type for HTTP fetches across every target. `Display` carries
/// a human-readable message; `From<ureq::Error>` is provided when the
/// `network` feature is enabled so call sites can use `?` directly.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct NetworkError(pub String);

impl NetworkError {
    /// Message used by every native fetch helper when the `network` feature
    /// is not compiled in. Exposed as a constant so callers that want to
    /// distinguish "feature disabled" from a real network failure can
    /// substring-match without hard-coding the prose.
    pub const FEATURE_DISABLED: &'static str = "fragmentcolor was built without the `network` feature; URL fetching is unavailable. \
         Use `Shader::new(<source>)` / `Shader::new(<file path>)` / Shader::new(<registry slug>) instead, \
         or rebuild the library manually with `cargo build --features network`.";

    pub fn feature_disabled() -> Self {
        NetworkError(Self::FEATURE_DISABLED.to_string())
    }
}

#[cfg(all(not(wasm), feature = "network"))]
impl From<ureq::Error> for NetworkError {
    fn from(e: ureq::Error) -> Self {
        NetworkError(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Native HTTP agent (only when `network` feature is on, on desktop targets).
// ---------------------------------------------------------------------------
//
// ureq 3.x defaults to Rustls when no provider is configured; we set
// `TlsProvider::NativeTls` explicitly so the dep tree picks the system TLS
// stack instead of `ring`. Lazy-init via `OnceLock` so the agent is shared
// across calls.
#[cfg(all(
    not(wasm),
    feature = "network",
    any(target_os = "linux", target_os = "macos", target_os = "windows")
))]
fn agent() -> &'static ureq::Agent {
    use std::sync::OnceLock;
    use ureq::config::Config;
    use ureq::tls::{TlsConfig, TlsProvider};
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        Config::builder()
            .tls_config(
                TlsConfig::builder()
                    .provider(TlsProvider::NativeTls)
                    .build(),
            )
            .build()
            .into()
    })
}

#[cfg(all(
    not(wasm),
    feature = "network",
    any(target_os = "linux", target_os = "macos", target_os = "windows")
))]
pub async fn fetch_text(url: &str) -> Result<String, NetworkError> {
    let resp = agent().get(url).call()?;
    let out = resp
        .into_body()
        .read_to_string()
        .map_err(|e| NetworkError(e.to_string()))?;
    Ok(out)
}

#[cfg(all(
    not(wasm),
    feature = "network",
    any(target_os = "linux", target_os = "macos", target_os = "windows")
))]
pub async fn fetch_bytes(url: &str) -> Result<Vec<u8>, NetworkError> {
    let resp = agent().get(url).call()?;
    let out = resp
        .into_body()
        .read_to_vec()
        .map_err(|e| NetworkError(e.to_string()))?;
    Ok(out)
}

// ---------------------------------------------------------------------------
// Native fallback: feature disabled, or mobile (iOS / Android always lacks
// ureq). Returns a clear typed error so callers can surface "rebuild with
// --features network" without having to special-case the build configuration
// themselves.
// ---------------------------------------------------------------------------
#[cfg(all(
    not(wasm),
    not(all(
        feature = "network",
        any(target_os = "linux", target_os = "macos", target_os = "windows")
    ))
))]
pub async fn fetch_text(_url: &str) -> Result<String, NetworkError> {
    Err(NetworkError::feature_disabled())
}

#[cfg(all(
    not(wasm),
    not(all(
        feature = "network",
        any(target_os = "linux", target_os = "macos", target_os = "windows")
    ))
))]
pub async fn fetch_bytes(_url: &str) -> Result<Vec<u8>, NetworkError> {
    Err(NetworkError::feature_disabled())
}

// ---------------------------------------------------------------------------
// Web (wasm32): always uses the browser's native `fetch`. No Cargo feature
// dependency, no extra deps to bundle.
// ---------------------------------------------------------------------------
#[cfg(wasm)]
pub async fn fetch_text(url: &str) -> Result<String, JsValue> {
    use wasm_bindgen::{JsCast, JsError};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response, window};

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let req = Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let win = window().ok_or_else(|| "no window".to_string())?;
    let resp_val = JsFuture::from(win.fetch_with_request(&req)).await?;
    let resp: Response = resp_val.dyn_into()?;
    let text = JsFuture::from(resp.text().map_err(|e| format!("{e:?}"))?).await?;

    let out = text
        .as_string()
        .ok_or_else(|| JsError::new("response text is not a string"))?;

    Ok(out)
}

#[cfg(wasm)]
pub async fn fetch_bytes(url: &str) -> Result<Vec<u8>, JsValue> {
    use js_sys::Uint8Array;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response, window};

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let req = Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let win = window().ok_or_else(|| "no window".to_string())?;
    let resp_val = JsFuture::from(win.fetch_with_request(&req))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let resp: Response = resp_val
        .dyn_into()
        .map_err(|_| "bad response".to_string())?;
    let buf = JsFuture::from(resp.array_buffer().map_err(|e| format!("{e:?}"))?)
        .await
        .map_err(|e| format!("{e:?}"))?;
    let u8 = Uint8Array::new(&buf);
    let mut out = vec![0u8; u8.length() as usize];
    u8.copy_to(&mut out[..]);

    Ok(out)
}

#[cfg(test)]
#[cfg(not(wasm))]
mod tests {
    use super::*;

    // Story: every native fetch surfaces a `NetworkError`. With `--features
    // network` an unreachable host produces a connect failure; without it
    // every URL errors with FEATURE_DISABLED. Both satisfy `is_err`.
    #[test]
    fn returns_error_for_unreachable_or_disabled() {
        pollster::block_on(async move {
            let url = "http://127.0.0.1:1/";

            let t = fetch_text(url).await;
            let b = fetch_bytes(url).await;

            assert!(t.is_err(), "fetch_text should error");
            assert!(b.is_err(), "fetch_bytes should error");
        });
    }
}
