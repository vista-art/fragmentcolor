//! Reads each glTF image as an encoded source, from a buffer view,
//! a data URI or a file beside the glTF, with its header checked.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::SceneLoadError;
use crate::TextureOptions;
use crate::core::shader::error::describe;
use crate::texture::{SourceData, TextureSource};

/// The source of each image in the file, kept encoded.
///
/// A GLB buffer view's bytes and a data URI's bytes are copied out once, so the scene
/// keeps no vertex data with them, and an external file keeps only its path. Each header
/// is read here, so an image whose format or size can't be read fails `Scene::load`, and
/// only a corrupt body waits for the first upload, whose error names the image.
pub(super) fn read_images(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    base: Option<&Path>,
) -> Result<Vec<TextureSource>, SceneLoadError> {
    let read = |image: gltf::Image<'_>| read_image(&image, buffers, base);
    document.images().map(read).collect()
}

/// The source of one image, named after its index and the name the file gives it.
fn read_image(
    image: &gltf::Image<'_>,
    buffers: &[gltf::buffer::Data],
    base: Option<&Path>,
) -> Result<TextureSource, SceneLoadError> {
    let label = match image.name() {
        Some(name) => format!("glTF image {} ({name})", image.index()),
        None => format!("glTF image {}", image.index()),
    };
    let refuse = |reason: ImageRefusal| SceneLoadError::Invalid(format!("{label}: {reason}"));
    let data = match image.source() {
        gltf::image::Source::View { view, .. } => {
            let buffer = buffers.get(view.buffer().index());
            let range = view.offset()..view.offset() + view.length();
            let bytes = buffer.and_then(|buffer| buffer.get(range));
            let bytes = bytes.ok_or_else(|| refuse(ImageRefusal::PastBuffer))?;
            SourceData::Bytes(bytes.to_vec())
        }
        gltf::image::Source::Uri { uri, .. } => read_uri(uri, base).map_err(refuse)?,
    };
    check_header(&data).map_err(refuse)?;
    let source = TextureSource::new(Arc::new(data), TextureOptions::default());
    Ok(source.named(label))
}

/// Why a glTF image can't become a texture source.
#[derive(Debug, thiserror::Error)]
enum ImageRefusal {
    #[error("its buffer view lies past its buffer")]
    PastBuffer,
    #[error("{}", describe(.0))]
    Uri(#[from] gltf::Error),
    #[error(
        "it names the file {0}, which a glTF loaded from bytes can't reach; load the file from its path"
    )]
    Unreachable(String),
    #[cfg(not(wasm))]
    #[error("can't read {}: {source}", .path.display())]
    File {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
    #[error("its header gives it no pixels")]
    Empty,
}

/// The data a glTF image URI names: a data URI's bytes, or the path of a file the glTF refers to.
fn read_uri(uri: &str, base: Option<&Path>) -> Result<SourceData, ImageRefusal> {
    if let Some(data) = uri.strip_prefix("data:") {
        let payload = data
            .split_once(";base64,")
            .map_or(data, |(_, payload)| payload);
        let source = gltf::buffer::Source::Uri(uri);
        let read = gltf::buffer::Data::from_source(source, None);
        let mut bytes = read?.0;
        // The buffer reader pads to four bytes, and the image keeps exactly what the URI holds.
        bytes.truncate(payload.trim_end_matches('=').len() * 3 / 4);
        return Ok(SourceData::Bytes(bytes));
    }
    let Some(base) = base else {
        return Err(ImageRefusal::Unreachable(uri.to_string()));
    };
    let path = match uri
        .strip_prefix("file://")
        .or_else(|| uri.strip_prefix("file:"))
    {
        Some(path) => PathBuf::from(path),
        None => base.join(decode_uri(uri)),
    };
    Ok(SourceData::Path(path))
}

/// `uri` with each `%XX` escape decoded, as glTF writes a relative file name.
fn decode_uri(uri: &str) -> String {
    let bytes = uri.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        let digits = bytes.get(index + 1..index + 3).filter(|_| byte == b'%');
        let value = digits
            .and_then(|digits| std::str::from_utf8(digits).ok())
            .and_then(|digits| u8::from_str_radix(digits, 16).ok());
        match value {
            Some(value) => {
                decoded.push(value);
                index += 3;
            }
            None => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

/// Reads an encoded image's header, its format and its size, without decoding its pixels.
fn check_header(data: &SourceData) -> Result<(), ImageRefusal> {
    let size = match data {
        SourceData::Bytes(bytes) => image::ImageReader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()?
            .into_dimensions(),
        #[cfg(not(wasm))]
        SourceData::Path(path) => image::ImageReader::open(path)
            .map_err(|source| ImageRefusal::File {
                path: path.clone(),
                source,
            })?
            .with_guessed_format()?
            .into_dimensions(),
        _ => return Ok(()),
    };
    match size? {
        (0, _) | (_, 0) => Err(ImageRefusal::Empty),
        _ => Ok(()),
    }
}
