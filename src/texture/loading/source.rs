//! What a texture uploads from, kept apart from any device.

use super::ktx2_loader;
use crate::TextureFormat;
use crate::texture::{Mipmap, TextureData, TextureError, TextureInput, TextureOptions};
use std::path::PathBuf;
use std::sync::Arc;

/// What a texture uploads from, before any device holds it.
/// Clones share one allocation, so each upload reads the data where it lies.
#[derive(Debug, Clone)]
pub(crate) struct TextureSource {
    pub(crate) data: Arc<SourceData>,
    pub(crate) options: TextureOptions,
    /// Whether `options.format` is a Material slot's hint rather than the caller's pick.
    /// A Mipmap or KTX2 container takes a hint only where its bytes read in it.
    pub(crate) hinted: bool,
    /// What an error names the source by, such as a glTF image, when a failed upload
    /// can't say on its own which image it read.
    pub(crate) label: Option<Arc<str>>,
}

/// The forms a texture uploads from. Each one holds data.
/// `Bytes` holds an encoded image, a KTX2 container by its magic, or raw pixels when `options.size` is set.
#[derive(Debug)]
pub(crate) enum SourceData {
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
        Self {
            data,
            options,
            hinted: false,
            label: None,
        }
    }

    /// This source, which an error from its upload names by `label`.
    pub(crate) fn named(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Takes `format` as a hint where the caller picked no format. An image uploads in it,
    /// and a Mipmap or KTX2 container only where its bytes read in it, as in an sRGB twin.
    pub(crate) fn take_hint(&mut self, format: TextureFormat) {
        if self.options.format == TextureFormat::default() {
            self.options.format = format;
            self.hinted = true;
        }
    }
}

impl TextureInput {
    /// Refuses options beside a `Texture`, which keeps the options it was created with.
    /// `call` names the call that took them, as the bindings name it.
    pub(crate) fn check_options(&self, call: &str) -> Result<(), TextureError> {
        let given = self.options != TextureOptions::default();
        match given && matches!(self.data, TextureData::CloneOf(_)) {
            true => Err(TextureError::InvalidInput(format!(
                "{call}: a Texture keeps the options it was created with, so it takes none"
            ))),
            false => Ok(()),
        }
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
