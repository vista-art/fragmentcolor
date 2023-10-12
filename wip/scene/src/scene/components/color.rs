use csscolorparser;
use palette::{
    rgb::{FromHexError, LinSrgb, LinSrgba},
    WithAlpha,
};
use std::str::FromStr;
use wgpu::ComputePipelineDescriptor;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Color {
    /// Create a new color
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Create a new color from a hex string
    pub fn from_hex(hex: &str) -> Result<Self, csscolorparser::ColorParseError> {
        Self::parse(hex)?;
    }

    /// Create a new color from a CSS string
    pub fn parse(color: &str) -> Result<Self, csscolorparser::ColorParseError> {
        let color = csscolorparser::parse(color)?;

        Ok(Self {
            red: color.r,
            green: color.g,
            blue: color.b,
            alpha: color.a,
        })
    }

    pub fn into_components(self) -> [f32; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorUniform {
    pub color: [f32; 4],
}

impl From<Color> for ColorUniform {
    fn from(color: Color) -> Self {
        Self {
            color: color.into_components(),
        }
    }
}

impl ColorUniform {
    pub fn new(color: Color) -> Self {
        Self {
            color: color.into_components(),
        }
    }

    pub fn update(&mut self, color: Color) {
        self.color = color.into_components();
    }
}
