use csscolorparser;
use serde::{Deserialize, Serialize};

/// Can be specified as 0xRRGGBBAA
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd, Deserialize)]
pub struct Color(pub u32);

impl Serialize for Color {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            self.red() as u8,
            self.green() as u8,
            self.blue() as u8,
            self.alpha() as u8
        ))
    }
}

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

impl From<Color> for wgpu::Color {
    fn from(c: Color) -> Self {
        Self {
            r: c.red() as f64,
            g: c.green() as f64,
            b: c.blue() as f64,
            a: c.alpha() as f64,
        }
    }
}

impl From<wgpu::Color> for Color {
    fn from(c: wgpu::Color) -> Self {
        Self::new(c.r as f32, c.g as f32, c.b as f32, c.a as f32)
    }
}

impl From<Color> for u32 {
    fn from(c: Color) -> Self {
        c.0
    }
}

impl From<u32> for Color {
    fn from(c: u32) -> Self {
        Self(c)
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        c.to_f32_array()
    }
}

impl From<[f32; 4]> for Color {
    fn from(c: [f32; 4]) -> Self {
        Self::from_rgba(c)
    }
}
