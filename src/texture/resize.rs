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

/// Checks that a texture `width` × `height` fits the device, each side in 1 to `limit`,
/// the device's largest side for the texture's dimension.
pub(crate) fn check_size(width: u32, height: u32, limit: u32) -> Result<(), TextureError> {
    let fits = |side: u32| (1..=limit).contains(&side);
    if fits(width) && fits(height) {
        return Ok(());
    }
    Err(TextureError::InvalidInput(format!(
        "can't use a size of {width}x{height}, each side must lie in 1 to {limit}"
    )))
}

/// Checks a texture's size against `limits` for its `dimension`. A 2D texture
/// takes each side in 1 to the 2D limit and 1 to the array limit of layers,
/// and a 3D texture takes every side, its depth too, in 1 to the 3D limit.
pub(crate) fn check_extent(
    limits: &wgpu::Limits,
    size: wgpu::Extent3d,
    dimension: wgpu::TextureDimension,
) -> Result<(), TextureError> {
    let (side, layers) = match dimension {
        wgpu::TextureDimension::D3 => (
            limits.max_texture_dimension_3d,
            limits.max_texture_dimension_3d,
        ),
        _ => (
            limits.max_texture_dimension_2d,
            limits.max_texture_array_layers,
        ),
    };
    check_size(size.width, size.height, side)?;
    let depth = size.depth_or_array_layers;
    if (1..=layers).contains(&depth) {
        return Ok(());
    }
    Err(TextureError::InvalidInput(format!(
        "can't use a depth of {depth}, it must lie in 1 to {layers}"
    )))
}

/// Checks a size for a texture of one layer, such as a target or a depth attachment:
/// a depth of 1 or none, and each side in 1 to `limit`.
pub(crate) fn check_plane(size: Size, limit: u32) -> Result<(), TextureError> {
    if let Some(depth) = size.depth.filter(|depth| *depth != 1) {
        return Err(TextureError::InvalidInput(format!(
            "a target or depth texture has one layer, got {depth}"
        )));
    }
    check_size(size.width, size.height, limit)
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
    check_size(extent.width, extent.height, limit)?;
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

    // Story: a size fits when each side lies in 1 to the limit.
    #[test]
    fn check_size_takes_sides_from_one_to_the_limit() {
        assert!(check_size(1, 8192, 8192).is_ok());
        assert!(matches!(
            check_size(0, 8, 8192),
            Err(TextureError::InvalidInput(_))
        ));
        assert!(matches!(
            check_size(8, 8193, 8192),
            Err(TextureError::InvalidInput(_))
        ));
    }

    // Story: a 2D texture takes layers up to the array limit, and a 3D one takes
    // every side up to the 3D limit.
    #[test]
    fn check_extent_takes_layers_within_the_limits() {
        let limits = wgpu::Limits::downlevel_defaults();
        let size = |width, height, depth_or_array_layers| wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers,
        };
        let (flat, deep) = (wgpu::TextureDimension::D2, wgpu::TextureDimension::D3);
        assert!(check_extent(&limits, size(8, 8, 1), flat).is_ok());
        assert!(check_extent(&limits, size(8, 8, 6), flat).is_ok());
        assert!(check_extent(&limits, size(8, 8, 0), flat).is_err());
        assert!(check_extent(&limits, size(8, 8, u32::MAX), flat).is_err());
        assert!(check_extent(&limits, size(0, 8, 1), flat).is_err());
        assert!(check_extent(&limits, size(8, 8, 8), deep).is_ok());
        assert!(check_extent(&limits, size(8, 8, u32::MAX), deep).is_err());
    }

    // Story: a plane has one layer, and its sides pass the same check.
    #[test]
    fn check_plane_takes_one_layer() {
        assert!(check_plane(Size::new(8, 8, None), 8192).is_ok());
        assert!(check_plane(Size::new(8, 8, Some(1)), 8192).is_ok());
        assert!(check_plane(Size::new(8, 8, Some(0)), 8192).is_err());
        assert!(check_plane(Size::new(8, 8, Some(2)), 8192).is_err());
        assert!(check_plane(Size::new(0, 8, None), 8192).is_err());
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
