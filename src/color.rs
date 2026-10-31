#[cfg(wasm)]
use wasm_bindgen::prelude::*;

pub mod error;
use error::{ColorError, CssFailure};

/// A linear RGBA color with one float per channel.
///
/// A color is a list in every language, `[r, g, b]` with an opaque alpha or `[r, g, b, a]`,
/// and `from_css` reads a CSS color. Values past 0 to 1 survive: a float target clears to them,
/// and a unorm target clamps them on the GPU.
///
/// ```
/// let color: fragmentcolor::Color = [1.0, 0.5, 0.0].into();
/// assert_eq!(color.to_f32_array(), [1.0, 0.5, 0.0, 1.0]);
/// ```
///
/// A tuple or a packed `u32` is not a color:
///
/// ```compile_fail
/// let color: fragmentcolor::Color = (1.0, 0.5, 0.0).into();
/// ```
///
/// ```compile_fail
/// let color = fragmentcolor::Color::from(0xFF8000FFu32);
/// ```
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass(from_py_object))]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Color([f32; 4]);

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

    /// A color from hex text such as `#ff8000`,
    /// or `ColorError::Css` with the parser's reason.
    pub fn from_hex(hex: &str) -> Result<Self, ColorError> {
        Self::from_css(hex)
    }

    /// A color from CSS text such as `#ff8000` or `rgb(255 128 0)`,
    /// or `ColorError::Css` with the text and the parser's reason.
    pub fn from_css(text: &str) -> Result<Self, ColorError> {
        let color = parse_css(text)?;
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

    /// The color as wgpu clears to it, each channel widened to `f64`.
    pub(crate) fn to_wgpu(self) -> wgpu::Color {
        let [r, g, b, a] = self.0.map(f64::from);
        wgpu::Color { r, g, b, a }
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

/// The parsed CSS color, or the failure that keeps the text.
fn parse_css(text: &str) -> Result<csscolorparser::Color, CssFailure<'_>> {
    match csscolorparser::parse(text) {
        Ok(color) => Ok(color),
        Err(error) => Err(CssFailure { text, error }),
    }
}

/// Reads `[r, g, b]` with an opaque alpha, or `[r, g, b, a]`, as a binding passes them.
impl TryFrom<&[f32]> for Color {
    type Error = ColorError;

    fn try_from(channels: &[f32]) -> Result<Self, Self::Error> {
        match *channels {
            [red, green, blue] => Ok(Self::new(red, green, blue, 1.0)),
            [red, green, blue, alpha] => Ok(Self::new(red, green, blue, alpha)),
            _ => Err(ColorError::Invalid(format!(
                "a color takes 3 or 4 channels, got {}",
                channels.len()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_from_refs() {
        let rgba = [0.2, 0.4, 0.6, 0.8];
        let rgb = [0.2, 0.4, 0.6];
        let c1: Color = (&rgba).into();
        let c2: Color = (&rgb).into();
        assert_eq!(c2, Color::new(0.2, 0.4, 0.6, 1.0));

        let out_rgba: [f32; 4] = (&c1).into();
        assert_eq!(out_rgba, rgba);
    }

    // Story: a color keeps values past 0 to 1, which a float target clears to,
    // and hands wgpu exactly the numbers it was given.
    #[test]
    fn keeps_every_channel_exactly() {
        let color = Color::from([4.0, -1.0, 0.25, 1.0]);
        assert_eq!(color.to_f32_array(), [4.0, -1.0, 0.25, 1.0]);
        let clear = color.to_wgpu();
        assert_eq!([clear.r, clear.g, clear.b, clear.a], [4.0, -1.0, 0.25, 1.0]);
        assert_eq!(Color::from([0.3, 0.6, 0.9]).green(), 0.6);
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

    // Story: CSS text reads as a color, and text that isn't one
    // fails with a `ColorError` that quotes it.
    #[test]
    fn reads_css_colors() {
        let orange = Color::from_css("#ff8000").map(Color::to_f32_array).ok();
        let [red, green, blue, alpha] = orange.unwrap_or_default();
        assert_eq!([red, blue, alpha], [1.0, 0.0, 1.0]);
        assert!((green - 128.0 / 255.0).abs() < 1e-6, "{green}");
        assert_eq!(Color::from_hex("#000").ok(), Some(Color::black()));
        let error = Color::from_css("not a color").err();
        assert!(
            matches!(&error, Some(ColorError::Css { text, .. }) if text == "not a color"),
            "{error:?}"
        );
        let message = error.map(|error| error.to_string()).unwrap_or_default();
        assert!(
            message.starts_with("\"not a color\" is not a CSS color: "),
            "{message}"
        );
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
