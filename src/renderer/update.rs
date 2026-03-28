use crate::renderer::Renderer;

pub(crate) fn update_texture(
    r: &Renderer,
    texture_id: crate::texture::TextureId,
    data: &[u8],
    options: crate::texture::TextureWriteOptions,
) -> Result<(), crate::renderer::error::RendererError> {
    let context = if let Some(ctx) = r.context.read().as_ref() {
        ctx.clone()
    } else {
        return Err(crate::renderer::error::RendererError::NoContext);
    };

    let tex = context.get_texture(&texture_id).ok_or(
        crate::renderer::error::RendererError::TextureNotFoundError(texture_id),
    )?;

    // Build a public Texture wrapper to reuse Texture::write_with
    let handle = crate::texture::Texture::new(context, tex, texture_id);
    handle.write_with(data, options)?;

    Ok(())
}
