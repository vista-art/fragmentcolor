use gloo_utils::format::JsValueSerdeExt;
use plr::{components::*, Object};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = Circle)]
pub struct JsCircle {
    inner: Object<Shape>,
}

#[wasm_bindgen(js_class = Circle)]
impl JsCircle {
    #[wasm_bindgen(constructor)]
    pub fn new(options: JsValue) -> Self {
        let options: CircleOptions = options.into_serde().unwrap();
        Self {
            inner: Circle::new(options),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn radius(&self) -> f32 {
        self.inner.radius()
    }

    #[wasm_bindgen(setter)]
    pub fn set_radius(&mut self, radius: f32) {
        self.inner.set_radius(radius);
    }

    #[wasm_bindgen(getter)]
    pub fn color(&self) -> JsValue {
        JsValue::from_serde(&self.inner.color()).unwrap()
    }

    #[wasm_bindgen(setter)]
    pub fn set_color(&mut self, color: JsValue) -> Result<(), JsValue> {
        let color: String = color.into_serde().unwrap();
        Ok(self.inner.set_color(Color::from_css(&color)?))
    }

    #[wasm_bindgen(getter)]
    pub fn border(&self) -> f32 {
        self.inner.border()
    }

    #[wasm_bindgen(setter)]
    pub fn set_border(&mut self, border: f32) {
        self.inner.set_border(border);
    }
}
