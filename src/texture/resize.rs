//! Resizing a texture a pass draws into.

use crate::texture::{Registration, TextureError, TextureObject};
use crate::{RenderContext, Size};

/// Recreates the texture `registration` holds at `size`, unless it already has that size.
pub(crate) fn resize(
    context: &RenderContext,
    registration: &Registration,
    size: Size,
) -> Result<(), TextureError> {
    registration.replace(|current| rebuild(context, current, size))
}

/// The texture to swap in for `current` at `size`, or `None` when the size stays.
fn rebuild(
    context: &RenderContext,
    current: &TextureObject,
    size: Size,
) -> Result<Option<TextureObject>, TextureError> {
    if !current
        .usage
        .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
    {
        return Err(TextureError::FixedSize);
    }
    let layers = current.size.depth_or_array_layers;
    if let Some(depth) = size.depth.filter(|depth| *depth != layers) {
        return Err(TextureError::InvalidInput(format!(
            "resize keeps {layers} layers, got {depth}"
        )));
    }
    let extent = wgpu::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: layers,
    };
    if extent == current.size {
        return Ok(None);
    }
    let limits = context.device.limits();
    let limit = match current.inner.dimension() {
        wgpu::TextureDimension::D3 => limits.max_texture_dimension_3d,
        _ => limits.max_texture_dimension_2d,
    };
    let fits = |side: u32| (1..=limit).contains(&side);
    if !fits(extent.width) || !fits(extent.height) {
        return Err(TextureError::InvalidInput(format!(
            "can't resize to {}x{}, each side must lie in 1 to {limit}",
            extent.width, extent.height
        )));
    }
    Ok(Some(current.resized(context, extent)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texture::SamplerOptions;
    use std::sync::Arc;

    // Story: a resize swaps the registry's object for the new one, keeps the
    // sample count and the sampler options, and never revives an unregistered id.
    #[test]
    fn resize_swaps_the_registered_texture() {
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let depth = renderer
                .create_depth_texture([8, 8])
                .await
                .expect("create_depth_texture failed");
            let context = depth.context.clone();
            let options = depth.object().options.read().to_owned();

            resize(&context, &depth.registration, Size::from([16, 4])).expect("resize failed");
            let object = depth.object();
            let registered = context.get_texture(depth.id()).expect("registered");
            assert!(
                Arc::ptr_eq(&registered, &object),
                "the registry holds the new texture"
            );
            assert_eq!(object.size.width, 16);
            assert_eq!(object.inner.sample_count(), 1);
            let kept: SamplerOptions = object.options.read().to_owned();
            assert_eq!(kept.compare, options.compare, "the sampler options survive");

            renderer
                .unregister_texture(depth.id())
                .expect("unregister failed");
            resize(&context, &depth.registration, Size::from([4, 4])).expect("resize failed");
            assert!(
                context.get_texture(depth.id()).is_none(),
                "an unregistered id stays gone"
            );
        });
    }

    // Story: only a texture a pass draws into resizes, and only to a size the device takes.
    #[test]
    fn resize_refuses_what_it_cannot_do() {
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let pixels: &[u8] = &[255, 255, 255, 255];
            let source = renderer
                .create_texture((pixels, [1, 1]))
                .await
                .expect("create_texture failed");
            let context = source.context.clone();
            let refused = resize(&context, &source.registration, Size::from([2, 2]));
            assert!(matches!(refused, Err(TextureError::FixedSize)));

            let depth = renderer
                .create_depth_texture([8, 8])
                .await
                .expect("create_depth_texture failed");
            let refused = resize(&context, &depth.registration, Size::from([0, 8]));
            assert!(matches!(refused, Err(TextureError::InvalidInput(_))));
            let kept: [u32; 2] = depth.size().into();
            assert_eq!(kept, [8, 8], "a refused resize keeps the size");
        });
    }
}
