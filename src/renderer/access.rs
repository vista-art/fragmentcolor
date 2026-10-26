//! The textures one pass uses, and the uses that can't share a texture.
//!
//! wgpu tracks each texture's uses in a usage scope: a whole render pass, or one
//! compute dispatch. Reads share a texture, and a write needs it alone, so a pass
//! that samples the texture it draws into is refused. These rules mirror wgpu's.

use crate::pass::PassError;
use crate::texture::{TextureClass, TextureStorageAccess};
use std::collections::HashMap;

/// How a pass uses a texture. Reads share it, and a write needs it alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Access {
    Read,
    Write(Writer),
}

/// What writes a texture. Two different writers conflict, like a write and a read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Writer {
    /// The pass draws into it.
    Color,
    /// The pass draws depth into it.
    Depth,
    /// A storage binding with this access stores into it.
    Storage(TextureStorageAccess),
}

impl Access {
    /// The use a texture binding of `class` makes, or `None` for an external texture.
    pub(super) fn from_class(class: &TextureClass) -> Option<Self> {
        match class {
            TextureClass::Sampled { .. } | TextureClass::Depth { .. } => Some(Self::Read),
            TextureClass::Storage { access, .. } if access.store || access.atomic => {
                Some(Self::Write(Writer::Storage(*access)))
            }
            TextureClass::Storage { .. } => Some(Self::Read),
            TextureClass::External() => None,
        }
    }
}

/// The textures one usage scope of a pass touches, each with its first use.
pub(super) struct Accesses<'pass> {
    pass: &'pass str,
    uses: HashMap<wgpu::Texture, (Access, String)>,
}

impl<'pass> Accesses<'pass> {
    /// An empty scope of the pass named `pass`, which its errors name.
    pub(super) fn new(pass: &'pass str) -> Self {
        Self {
            pass,
            uses: HashMap::new(),
        }
    }

    /// Records that the pass draws color or depth into `texture`.
    pub(super) fn attach(
        &mut self,
        texture: &wgpu::Texture,
        writer: Writer,
    ) -> Result<(), PassError> {
        let user = match writer {
            Writer::Depth => "the pass draws depth into",
            _ => "the pass draws into",
        };
        self.insert(texture, Access::Write(writer), user.into())
    }

    /// Records that the binding `name` uses `texture`.
    pub(super) fn bind(
        &mut self,
        texture: &wgpu::Texture,
        access: Access,
        name: &str,
    ) -> Result<(), PassError> {
        let user = match access {
            Access::Read => format!("'{name}' reads"),
            Access::Write(_) => format!("'{name}' writes"),
        };
        self.insert(texture, access, user)
    }

    /// Keeps the first use of `texture`, and refuses a second one it can't share.
    fn insert(
        &mut self,
        texture: &wgpu::Texture,
        access: Access,
        user: String,
    ) -> Result<(), PassError> {
        match self.uses.get(texture) {
            None => {
                self.uses.insert(texture.clone(), (access, user));
                Ok(())
            }
            Some((first, _)) if *first == access => Ok(()),
            Some((_, earlier)) => Err(PassError::TextureConflict {
                pass: self.pass.to_string(),
                detail: format!("{user} a texture that {earlier}; use it in a later pass"),
            }),
        }
    }
}
