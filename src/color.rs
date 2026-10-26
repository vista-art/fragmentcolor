#[cfg(wasm)]
use wasm_bindgen::prelude::*;

pub mod error;
use error::ColorError;

/// A linear RGBA color with one float per channel.
/// `From<u32>` reads `0xRRGGBBAA` bytes, and `from_css` reads a CSS color.
/// Values past 0 to 1 survive: a float target clears to them, and a unorm target clamps them on the GPU.
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass(from_py_object))]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Color([f32; 4]);

const GAMMA: f32 = 2.2;

/// Channel names in the order a color stores them.
const CHANNELS: [&str; 4] = ["red", "green", "blue", "alpha"];

impl Color {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self([red, green, blue, alpha])
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

    pub fn from_rgba(channels: [f32; 4]) -> Self {
        Self(channels)
    }

    pub fn from_rgb_alpha(rgb: [f32; 3], alpha: f32) -> Self {
        Self::new(rgb[0], rgb[1], rgb[2], alpha)
    }

    /// Create a new color from a hex string
    pub fn from_hex(hex: &str) -> Result<Self, csscolorparser::ParseColorError> {
        Self::from_css(hex)
    }

    /// Create a new color from a CSS string
    pub fn from_css(color: &str) -> Result<Self, csscolorparser::ParseColorError> {
        let color = csscolorparser::parse(color)?;
        Ok(Self::new(color.r, color.g, color.b, color.a))
    }

    pub fn red(self) -> f32 {
        self.0[0]
    }

    pub fn green(self) -> f32 {
        self.0[1]
    }

    pub fn blue(self) -> f32 {
        self.0[2]
    }

    pub fn alpha(self) -> f32 {
        self.0[3]
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
        self.0
    }

    pub fn into_vec4_gamma(self) -> [f32; 4] {
        self.0.map(|channel| channel.powf(GAMMA))
    }

    /// The color, or an error naming the first channel that is NaN or infinite.
    pub(crate) fn check_finite(self) -> Result<Self, ColorError> {
        for (channel, value) in CHANNELS.into_iter().zip(self.0) {
            if !value.is_finite() {
                return Err(ColorError::NotFinite { channel, value });
            }
        }
        Ok(self)
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

/// Reads `0xRRGGBBAA`, one byte per channel.
impl From<u32> for Color {
    fn from(bytes: u32) -> Self {
        let channel = |shift: u32| ((bytes >> shift) & 0xFF) as f32 / 255.0;
        Self::new(channel(24), channel(16), channel(8), channel(0))
    }
}

impl From<&u32> for Color {
    fn from(bytes: &u32) -> Self {
        Self::from(*bytes)
    }
}

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

crate::impl_from_into_with_refs!(
    Color,
    (f32, f32, f32, f32),
    |c: Color| (c.red(), c.green(), c.blue(), c.alpha()),
    |t: (f32, f32, f32, f32)| Color::from_rgba([t.0, t.1, t.2, t.3])
);

/// Reads `[r, g, b]` with an opaque alpha, or `[r, g, b, a]`, as a binding passes them.
impl TryFrom<&[f32]> for Color {
    type Error = ColorError;

    fn try_from(channels: &[f32]) -> Result<Self, Self::Error> {
        match *channels {
            [red, green, blue] => Ok(Self::new(red, green, blue, 1.0)),
            [red, green, blue, alpha] => Ok(Self::new(red, green, blue, alpha)),
            _ => Err(ColorError::TypeMismatch(format!(
                "a color takes 3 or 4 channels, got {}",
                channels.len()
            ))),
        }
    }
}

/// Reads a CSS string, an `{ r, g, b, a }` object, or `[r, g, b]` and `[r, g, b, a]`
/// as a plain array or a `Float32Array`. Numbers keep their value, and a missing alpha is 1.
#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for Color {
    type Error = ColorError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::{Array, Float32Array, Reflect};

        if let Some(css) = value.as_string() {
            return Ok(Color::from_css(&css)?);
        }

        if Array::is_array(value) || value.is_instance_of::<Float32Array>() {
            let items = Array::from(value);
            if !(3..=4).contains(&items.length()) {
                return Err(ColorError::TypeMismatch(format!(
                    "a color takes 3 or 4 channels, got {}",
                    items.length()
                )));
            }
            let channels = items
                .iter()
                .zip(CHANNELS)
                .map(|(item, channel)| read_channel(channel, &item))
                .collect::<Result<Vec<f32>, ColorError>>()?;
            return Color::try_from(channels.as_slice());
        }

        if value.is_object() {
            let field = |name: &str| {
                Reflect::get(value, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
            };
            let mut channels = Vec::with_capacity(4);
            for (name, channel) in ["r", "g", "b"].into_iter().zip(CHANNELS) {
                channels.push(read_channel(channel, &field(name))?);
            }
            let alpha = field("a");
            if !alpha.is_undefined() {
                channels.push(read_channel("alpha", &alpha)?);
            }
            return Color::try_from(channels.as_slice());
        }

        Err(ColorError::TypeMismatch(
            "a color takes [r, g, b], [r, g, b, a], a CSS string, or { r, g, b, a }".into(),
        ))
    }
}

/// One channel from JavaScript, which must be a number.
#[cfg(wasm)]
fn read_channel(channel: &str, item: &JsValue) -> Result<f32, ColorError> {
    item.as_f64().map(|number| number as f32).ok_or_else(|| {
        ColorError::TypeMismatch(format!(
            "the {channel} channel must be a number, got {item:?}"
        ))
    })
}

#[cfg(wasm)]
crate::impl_tryfrom_owned_via_ref!(Color, wasm_bindgen::JsValue, ColorError);

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
    }

    // Story: a color keeps values past 0 to 1, which a float target clears to,
    // and hands wgpu exactly the numbers it was given.
    #[test]
    fn keeps_every_channel_exactly() {
        let color = Color::from([4.0, -1.0, 0.25, 1.0]);
        assert_eq!(color.to_f32_array(), [4.0, -1.0, 0.25, 1.0]);
        let clear = wgpu::Color::from(color);
        assert_eq!([clear.r, clear.g, clear.b, clear.a], [4.0, -1.0, 0.25, 1.0]);
        assert_eq!(Color::from([0.3, 0.6, 0.9]).green(), 0.6);
    }

    // Story: 0xRRGGBBAA reads one byte per channel.
    #[test]
    fn reads_bytes_from_a_u32() {
        let color = Color::from(0xFF8000FFu32);
        assert_eq!(color.red(), 1.0);
        assert_eq!(color.green(), 128.0 / 255.0);
        assert_eq!(color.blue(), 0.0);
        assert_eq!(color.alpha(), 1.0);
    }

    // Story: a binding's list of 3 or 4 channels becomes a color, and any
    // other length fails.
    #[test]
    fn reads_three_or_four_channels() {
        let rgb = Color::try_from([0.5, 2.0, 0.0].as_slice()).ok();
        assert_eq!(rgb, Some(Color::new(0.5, 2.0, 0.0, 1.0)));
        let rgba = Color::try_from([0.5, 2.0, 0.0, 0.5].as_slice()).ok();
        assert_eq!(rgba, Some(Color::new(0.5, 2.0, 0.0, 0.5)));
        assert!(Color::try_from([0.5, 2.0].as_slice()).is_err());
        assert!(Color::try_from([0.0; 5].as_slice()).is_err());
    }

    // Story: NaN or infinity in any channel fails and names the channel.
    #[test]
    fn refuses_channels_that_are_not_finite() {
        assert!(Color::from([4.0, -1.0, 0.0, 1.0]).check_finite().is_ok());
        let message = Color::from([0.0, f32::NAN, 0.0, 1.0])
            .check_finite()
            .err()
            .map(|error| error.to_string());
        assert_eq!(
            message.as_deref(),
            Some("the green channel must be a finite number, got NaN")
        );
        assert!(
            Color::from([0.0, 0.0, 0.0, f32::INFINITY])
                .check_finite()
                .is_err()
        );
    }
}
