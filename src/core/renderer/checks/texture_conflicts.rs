//! Refuses a pass that writes a texture it also reads, in one usage scope.

use crate::PassError;
use crate::texture::{TextureClass, TextureStorageAccess};
use std::collections::HashMap;

/// How a pass uses a texture. Reads share it, and a write needs it alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TextureUse {
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

impl TextureUse {
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
pub(super) struct UsageScope<'pass> {
    pass: &'pass str,
    uses: HashMap<wgpu::Texture, (TextureUse, String)>,
}

impl<'pass> UsageScope<'pass> {
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
        self.insert(texture, TextureUse::Write(writer), user.into())
    }

    /// Records that the binding `name` uses `texture`.
    pub(super) fn bind(
        &mut self,
        texture: &wgpu::Texture,
        access: TextureUse,
        name: &str,
    ) -> Result<(), PassError> {
        let user = match access {
            TextureUse::Read => format!("'{name}' reads"),
            TextureUse::Write(_) => format!("'{name}' writes"),
        };
        self.insert(texture, access, user)
    }

    /// Keeps the first use of `texture`, and refuses a second one it can't share.
    fn insert(
        &mut self,
        texture: &wgpu::Texture,
        access: TextureUse,
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
