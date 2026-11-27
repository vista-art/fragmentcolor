//! Turns a path, bytes, a URL or a KTX2 file into a texture, off the caller's thread.

pub(super) mod decode;
pub(crate) mod ktx2_loader;
mod source;
pub(crate) mod worker;

pub(crate) use source::{SourceData, TextureSource};
