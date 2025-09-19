use csscolorparser;

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

pub mod error;

/// Can be specified as 0xRRGGBBAA
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd)]
pub struct Color(pub u32);

const GAMMA: f32 = 2.2;

impl Color {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self(
            (Self::import(red) << 24)
                | (Self::import(green) << 16)
                | (Self::import(blue) << 8)
                | Self::import(alpha),
        )
    }

    pub fn transparent() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    pub fn from_rgba(d: [f32; 4]) -> Self {
        Self::new(d[0], d[1], d[2], d[3])
    }

    pub fn from_rgb_alpha(d: [f32; 3], alpha: f32) -> Self {
        Self::new(d[0], d[1], d[2], alpha)
    }

    /// Create a new color from a hex string
    pub fn from_hex(hex: &str) -> Result<Self, csscolorparser::ParseColorError> {
        Self::from_css(hex)
    }

    /// Create a new color from a CSS string
    pub fn from_css(color: &str) -> Result<Self, csscolorparser::ParseColorError> {
        let color = csscolorparser::parse(color)?;

        Ok(Self::new(
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        ))
    }

    pub fn red(self) -> f32 {
        self.export(3)
    }

    pub fn green(self) -> f32 {
        self.export(2)
    }

    pub fn blue(self) -> f32 {
        self.export(1)
    }

    pub fn alpha(self) -> f32 {
        self.export(0)
    }

    pub fn r(self) -> f32 {
        self.red()
    }

    pub fn g(self) -> f32 {
        self.green()
    }

    pub fn b(self) -> f32 {
        self.blue()
    }

    pub fn a(self) -> f32 {
        self.alpha()
    }

    pub fn to_f32_array(self) -> [f32; 4] {
        [self.red(), self.green(), self.blue(), self.alpha()]
    }

    pub fn into_vec4_gamma(self) -> [f32; 4] {
        [
            self.red().powf(GAMMA),
            self.green().powf(GAMMA),
            self.blue().powf(GAMMA),
            self.alpha().powf(GAMMA),
        ]
    }

    fn import(value: f32) -> u32 {
        (value.clamp(0.0, 1.0) * 255.0) as u32
    }

    fn export(self, index: u32) -> f32 {
        ((self.0 >> (index << 3)) & 0xFF) as f32 / 255.0
    }
}

crate::impl_from_into_with_refs!(
    Color,
    wgpu::Color,
    |c: Color| wgpu::Color {
        r: c.red() as f64,
        g: c.green() as f64,
        b: c.blue() as f64,
        a: c.alpha() as f64,
    },
    |wc: wgpu::Color| Color::new(wc.r as f32, wc.g as f32, wc.b as f32, wc.a as f32)
);

crate::impl_from_into_with_refs!(Color, u32, |c: Color| c.0, |v: u32| Color(v));

crate::impl_from_into_with_refs!(
    Color,
    [f32; 4],
    |c: Color| c.to_f32_array(),
    |a: [f32; 4]| Color::from_rgba(a)
);

crate::impl_from_into_with_refs!(
    Color,
    [f32; 3],
    |c: Color| [c.red(), c.green(), c.blue()],
    |rgb: [f32; 3]| Color::from_rgb_alpha(rgb, 1.0)
);

crate::impl_from_into_with_refs!(
    Color,
    (f32, f32, f32),
    |c: Color| (c.red(), c.green(), c.blue()),
    |t: (f32, f32, f32)| Color::from_rgb_alpha([t.0, t.1, t.2], 1.0)
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_from_refs() {
        let rgba = [0.2, 0.4, 0.6, 0.8];
        let rgb = [0.2, 0.4, 0.6];
        let c1: Color = (&rgba).into();
        let _c2: Color = (&rgb).into();
        let _c3: Color = (&(0.2f32, 0.4f32, 0.6f32)).into();
        let _c4: Color = (&(0.2f32, 0.4f32, 0.6f32, 0.8f32)).into();

        let out_rgba: [f32; 4] = (&c1).into();
        assert_eq!(out_rgba, rgba);

        let wc = wgpu::Color::from(c1);
        let back: Color = (&wc).into();
        assert_eq!(back, Color::from(rgba));

        let u: u32 = (&back).into();
        let back2: Color = (&u).into();
        assert_eq!(back2, back);
    }
}

crate::impl_from_into_with_refs!(
    Color,
    (f32, f32, f32, f32),
    |c: Color| (c.red(), c.green(), c.blue(), c.alpha()),
    |t: (f32, f32, f32, f32)| Color::from_rgba([t.0, t.1, t.2, t.3])
);

#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for Color {
    type Error = crate::color::error::ColorError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::{Array, Float32Array, Int32Array, Reflect, Uint32Array};
        use wasm_bindgen::JsCast;

        // Helper to normalize [r,g,b,a?] numbers; if any component > 1.0, assume 0..255 space
        fn normalize_rgba(mut c: [f32; 4], has_alpha: bool) -> [f32; 4] {
            let mut maxc = c[0].abs().max(c[1].abs()).max(c[2].abs());
            if has_alpha {
                maxc = maxc.max(c[3].abs());
            }
            if maxc > 1.0 {
                for i in 0..(if has_alpha { 4 } else { 3 }) {
                    c[i] = (c[i] / 255.0).clamp(0.0, 1.0);
                }
            } else {
                for i in 0..(if has_alpha { 4 } else { 3 }) {
                    c[i] = c[i].clamp(0.0, 1.0);
                }
            }
            c
        }

        // Strings: CSS/hex
        if let Some(s) = value.as_string() {
            return Color::from_css(&s)
                .map_err(|e| crate::color::error::ColorError::TypeMismatch(e.to_string()));
        }

        // Typed arrays
        if let Some(arr) = value.dyn_ref::<Float32Array>() {
            let len = arr.length();
            if len == 3 {
                let mut buf = [0.0f32; 3];
                arr.copy_to(&mut buf);
                let c = normalize_rgba([buf[0], buf[1], buf[2], 1.0], false);
                return Ok(Color::from(c));
            }
            if len == 4 {
                let mut buf = [0.0f32; 4];
                arr.copy_to(&mut buf);
                let c = normalize_rgba(buf, true);
                return Ok(Color::from(c));
            }
        }
        if let Some(arr) = value.dyn_ref::<Int32Array>() {
            let len = arr.length();
            if len == 3 {
                let mut buf = [0i32; 3];
                arr.copy_to(&mut buf);
                let c = normalize_rgba([buf[0] as f32, buf[1] as f32, buf[2] as f32, 255.0], true);
                return Ok(Color::from([c[0], c[1], c[2], c[3]]));
            }
            if len == 4 {
                let mut buf = [0i32; 4];
                arr.copy_to(&mut buf);
                let c = normalize_rgba(
                    [buf[0] as f32, buf[1] as f32, buf[2] as f32, buf[3] as f32],
                    true,
                );
                return Ok(Color::from([c[0], c[1], c[2], c[3]]));
            }
        }
        if let Some(arr) = value.dyn_ref::<Uint32Array>() {
            let len = arr.length();
            if len == 3 {
                let mut buf = [0u32; 3];
                arr.copy_to(&mut buf);
                let c = normalize_rgba([buf[0] as f32, buf[1] as f32, buf[2] as f32, 255.0], true);
                return Ok(Color::from([c[0], c[1], c[2], c[3]]));
            }
            if len == 4 {
                let mut buf = [0u32; 4];
                arr.copy_to(&mut buf);
                let c = normalize_rgba(
                    [buf[0] as f32, buf[1] as f32, buf[2] as f32, buf[3] as f32],
                    true,
                );
                return Ok(Color::from([c[0], c[1], c[2], c[3]]));
            }
        }

        // Plain JS arrays
        if let Some(arr) = value.dyn_ref::<Array>() {
            let len = arr.length();
            let num_at = |i: u32| arr.get(i).as_f64().unwrap_or(0.0) as f32;
            if len == 3 {
                let c = normalize_rgba([num_at(0), num_at(1), num_at(2), 1.0], false);
                return Ok(Color::from([c[0], c[1], c[2], c[3]]));
            }
            if len == 4 {
                let c = normalize_rgba([num_at(0), num_at(1), num_at(2), num_at(3)], true);
                return Ok(Color::from([c[0], c[1], c[2], c[3]]));
            }
        }

        // Object with { r, g, b, a? }
        if value.is_object() {
            let r = Reflect::get(&value, &wasm_bindgen::JsValue::from_str("r"))
                .ok()
                .and_then(|v| v.as_f64());
            let g = Reflect::get(&value, &wasm_bindgen::JsValue::from_str("g"))
                .ok()
                .and_then(|v| v.as_f64());
            let b = Reflect::get(&value, &wasm_bindgen::JsValue::from_str("b"))
                .ok()
                .and_then(|v| v.as_f64());
            if let (Some(r), Some(g), Some(b)) = (r, g, b) {
                let a = Reflect::get(&value, &wasm_bindgen::JsValue::from_str("a"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0);
                let c = normalize_rgba([r as f32, g as f32, b as f32, a as f32], true);
                return Ok(Color::from([c[0], c[1], c[2], c[3]]));
            }
        }

        Err(crate::color::error::ColorError::TypeMismatch(
            "Cannot convert JavaScript value to Color (expected [r,g,b] or [r,g,b,a], CSS string, or {r,g,b,a})".into(),
        ))
    }
}

#[cfg(wasm)]
crate::impl_tryfrom_owned_via_ref!(
    Color,
    wasm_bindgen::JsValue,
    crate::color::error::ColorError
);
