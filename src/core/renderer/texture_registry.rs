//! The textures a renderer holds by `TextureId`, and how one leaves the registry.

use crate::core::renderer::Renderer;

pub(crate) fn unregister_texture(
    r: &Renderer,
    texture_id: crate::texture::TextureId,
) -> Result<(), crate::core::renderer::RendererError> {
    let context = if let Some(ctx) = r.context.read().as_ref() {
        ctx.clone()
    } else {
        return Err(crate::core::renderer::RendererError::NoContext);
    };

    // Remove from registry; return NotFound if absent
    let removed = context.textures.remove(&texture_id);
    if removed.is_none() {
        return Err(crate::core::renderer::RendererError::TextureNotFoundError(
            texture_id,
        ));
    }

    Ok(())
}
