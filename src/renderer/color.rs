use palette::{
    rgb::{FromHexError, LinSrgb, LinSrgba},
    WithAlpha,
};
use std::str::FromStr;

pub fn hex_to_rgba(hex: &str) -> Result<LinSrgba<f32>, FromHexError> {
    let hex = hex.strip_prefix('#').map_or(hex, |stripped| stripped);

    match hex.len() {
        3 | 6 => {
            let color = LinSrgb::from_str(hex)?.into_format().with_alpha(1.0);

            Ok(color)
        }
        8 => {
            let alpha = u8::from_str_radix(&hex[6..], 16)? as f32 / 255.0;
            let color = LinSrgb::from_str(&hex[..6])?
                .into_format()
                .with_alpha(alpha);

            Ok(color)
        }
        _ => Err("invalid hex code format".into()),
    }
}
