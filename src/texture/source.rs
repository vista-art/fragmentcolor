//! What a texture uploads from, kept apart from any device.

use super::{Mipmap, TextureData, TextureError, TextureInput, TextureOptions, ktx2_loader};
use std::path::PathBuf;
use std::sync::Arc;

/// What a texture uploads from, before any device holds it.
/// Clones share one allocation, so each upload reads the data where it lies.
#[derive(Debug, Clone)]
pub(crate) struct TextureSource {
    pub(crate) data: Arc<SourceData>,
    pub(crate) options: TextureOptions,
}

/// The forms a texture uploads from. Each one holds data.
#[derive(Debug)]
pub(crate) enum SourceData {
    /// An encoded image, a KTX2 container by its magic, or raw pixels when `options.size` is set.
    Bytes(Vec<u8>),
    Ktx2Bytes(Vec<u8>),
    Path(PathBuf),
    Ktx2Path(PathBuf),
    Url(String),
    Ktx2Url(String),
    Prepared(Mipmap),
}

impl TextureSource {
    /// A source that reads `data`, which other sources may share, with `options`.
    pub(crate) fn new(data: Arc<SourceData>, options: TextureOptions) -> Self {
        Self { data, options }
    }
}

impl SourceData {
    /// Whether the data brings its own size, as every form but raw pixel bytes does.
    pub(crate) fn is_sized(&self) -> bool {
        match self {
            SourceData::Bytes(bytes) => ktx2_loader::is_ktx2(bytes),
            SourceData::Ktx2Bytes(_)
            | SourceData::Path(_)
            | SourceData::Ktx2Path(_)
            | SourceData::Url(_)
            | SourceData::Ktx2Url(_)
            | SourceData::Prepared(_) => true,
        }
    }
}

impl TryFrom<TextureInput> for TextureSource {
    type Error = TextureError;

    /// Moves the input's data into one shared allocation, with no copy.
    fn try_from(input: TextureInput) -> Result<Self, Self::Error> {
        let data = match input.data {
            TextureData::Bytes(bytes) => SourceData::Bytes(bytes),
            TextureData::Ktx2Bytes(bytes) => SourceData::Ktx2Bytes(bytes),
            TextureData::Path(path) => SourceData::Path(path),
            TextureData::Ktx2Path(path) => SourceData::Ktx2Path(path),
            TextureData::Url(url) => SourceData::Url(url),
            TextureData::Ktx2Url(url) => SourceData::Ktx2Url(url),
            TextureData::Prepared(chain) => SourceData::Prepared(chain),
            TextureData::Empty => {
                return Err(TextureError::InvalidInput(
                    "a texture source needs data: bytes, a path, a URL or a Mipmap".into(),
                ));
            }
            TextureData::CloneOf(_) => {
                return Err(TextureError::InvalidInput(
                    "a Texture already lives on a Renderer's device, so it's no source to upload"
                        .into(),
                ));
            }
        };
        Ok(Self::new(Arc::new(data), input.options))
    }
}
