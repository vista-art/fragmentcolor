#![cfg(wasm)]

use crate::{Shader, ShaderError};

impl Shader {
    pub async fn fetch(url: &str) -> Result<Self, ShaderError> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::{JsFuture, future_to_promise};
        use web_sys::Request;
        use web_sys::RequestInit;
        use web_sys::RequestMode;
        use web_sys::Response;

        let opts = RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(url, &opts).expect("failed to create request");
        let window = web_sys::window().expect("no global `window` exists");
        let resp_promise = window.fetch_with_request(&request);
        let resp_value = future_to_promise(JsFuture::from(resp_promise));

        let resp: Response = resp_value.dyn_into().expect("not a Response");

        let jsvalue = JsFuture::from(resp.text().expect("failed to read response"))
            .await
            .expect("failed to read response");

        let body = jsvalue.as_string().expect("response not a string");

        Self::new(&body)
    }
}
