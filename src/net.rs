//! Cross-target URL fetching helpers
//! Returns errors as Strings so call-sites can convert to their local error types.

#[cfg(wasm)]
use wasm_bindgen_test::__rt::wasm_bindgen::JsValue;

#[cfg(not(wasm))]
pub async fn fetch_text(url: &str) -> Result<String, ureq::Error> {
    let resp = ureq::get(url).call()?;
    let out = resp.into_body().read_to_string()?;
    Ok(out)
}

#[cfg(not(wasm))]
pub async fn fetch_bytes(url: &str) -> Result<Vec<u8>, ureq::Error> {
    let resp = ureq::get(url).call()?;
    let out = resp.into_body().read_to_vec()?;
    Ok(out)
}

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

    // Story: Attempting to fetch from an unroutable local port should yield a network error
    // for both text and bytes helpers.
    #[test]
    fn returns_error_on_unreachable_host() {
        pollster::block_on(async move {
            // Arrange
            let url = "http://127.0.0.1:1/";

            // Act
            let t = fetch_text(url).await;
            let b = fetch_bytes(url).await;

            // Assert
            assert!(t.is_err(), "fetch_text should error on unreachable host");
            assert!(b.is_err(), "fetch_bytes should error on unreachable host");
        });
    }
}
