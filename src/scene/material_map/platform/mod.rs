#[cfg(mobile)]
pub(crate) mod mobile;
#[cfg(python)]
pub(crate) mod python;
#[cfg(wasm)]
pub(crate) mod web;

#[cfg(any(python, wasm))]
use crate::texture::TextureError;

/// The refusal of options beside a `MaterialMap`, which shares its texture as it is.
#[cfg(any(python, wasm))]
fn refuse_options(call: &str) -> TextureError {
    TextureError::InvalidInput(format!(
        "{call}: a MaterialMap shares its texture as it is, so it takes no options"
    ))
}
