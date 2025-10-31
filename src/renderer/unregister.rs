use crate::renderer::Renderer;

pub(crate) fn unregister_texture(
    r: &Renderer,
    texture_id: crate::texture::TextureId,
) -> Result<(), crate::renderer::error::RendererError> {
    let context = if let Some(ctx) = r.context.read().as_ref() {
        ctx.clone()
    } else {
        return Err(crate::renderer::error::RendererError::NoContext);
    };

    // Remove from registry; return NotFound if absent
    let removed = context.textures.remove(&texture_id);
    if removed.is_none() {
        return Err(crate::renderer::error::RendererError::TextureNotFoundError(
            texture_id,
        ));
    }

    Ok(())
}
