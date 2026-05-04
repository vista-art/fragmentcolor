//! KTX2 container loader.
//!
//! Parses a KTX2 byte slice via the `ktx2` crate, validates that the contents
//! match the subset FragmentColor supports today (single-layer, single-face,
//! 2D, no supercompression), maps the Vulkan format code to a
//! [`wgpu::TextureFormat`], and uploads each pre-baked mip level directly. We
//! deliberately skip the CPU `imageops::resize` chain we use for JPEG/PNG
//! sources — the encoder already picked the format and built the chain.
//!
//! Cube maps, array textures, 3D textures, and Basis Universal supercompression
//! are out of scope for this pass and will fail with a descriptive error so the
//! caller sees them clearly rather than getting silently-wrong output.

use ktx2::Format as VkFormat;

use crate::RenderContext;
use crate::texture::error::TextureError;
use crate::texture::{
    SamplerOptions, TextureObject, bytes_per_pixel, create_default_sampler, mip_level_count_for,
};
use parking_lot::RwLock;

/// Map a KTX2 (Vulkan) format code to the equivalent `wgpu::TextureFormat`.
///
/// Returns `None` for formats FragmentColor doesn't decode today; callers turn
/// that into a [`TextureError`] explaining which format showed up.
///
/// The supported set covers the formats RemixBrush and similar consumers ship:
/// uncompressed 8-bit / 16-bit RGBA, BC1–7 (desktop), ETC2 (Android),
/// ASTC 4×4 / 8×8 (mobile + WebGPU). Anything else falls through to `None`
/// for now — extending the table is a one-line change per format.
pub(crate) fn vk_format_to_wgpu(format: VkFormat) -> Option<wgpu::TextureFormat> {
    use wgpu::{AstcBlock, AstcChannel, TextureFormat};

    let astc = |block: AstcBlock, channel: AstcChannel| TextureFormat::Astc { block, channel };

    Some(match format {
        // Uncompressed scalars and small vectors
        VkFormat::R8_UNORM => TextureFormat::R8Unorm,
        VkFormat::R8G8_UNORM => TextureFormat::Rg8Unorm,
        VkFormat::R16_UNORM => TextureFormat::R16Unorm,
        VkFormat::R16G16_UNORM => TextureFormat::Rg16Unorm,

        // Uncompressed RGBA8 / RGBA16F (most common KTX2 uncompressed payloads)
        VkFormat::R8G8B8A8_UNORM => TextureFormat::Rgba8Unorm,
        VkFormat::R8G8B8A8_SRGB => TextureFormat::Rgba8UnormSrgb,
        VkFormat::B8G8R8A8_UNORM => TextureFormat::Bgra8Unorm,
        VkFormat::B8G8R8A8_SRGB => TextureFormat::Bgra8UnormSrgb,
        VkFormat::R16G16B16A16_UNORM => TextureFormat::Rgba16Unorm,
        VkFormat::R16G16B16A16_SFLOAT => TextureFormat::Rgba16Float,

        // BC1-7 (desktop; needs TEXTURE_COMPRESSION_BC)
        VkFormat::BC1_RGB_UNORM_BLOCK | VkFormat::BC1_RGBA_UNORM_BLOCK => {
            TextureFormat::Bc1RgbaUnorm
        }
        VkFormat::BC1_RGB_SRGB_BLOCK | VkFormat::BC1_RGBA_SRGB_BLOCK => {
            TextureFormat::Bc1RgbaUnormSrgb
        }
        VkFormat::BC2_UNORM_BLOCK => TextureFormat::Bc2RgbaUnorm,
        VkFormat::BC2_SRGB_BLOCK => TextureFormat::Bc2RgbaUnormSrgb,
        VkFormat::BC3_UNORM_BLOCK => TextureFormat::Bc3RgbaUnorm,
        VkFormat::BC3_SRGB_BLOCK => TextureFormat::Bc3RgbaUnormSrgb,
        VkFormat::BC4_UNORM_BLOCK => TextureFormat::Bc4RUnorm,
        VkFormat::BC4_SNORM_BLOCK => TextureFormat::Bc4RSnorm,
        VkFormat::BC5_UNORM_BLOCK => TextureFormat::Bc5RgUnorm,
        VkFormat::BC5_SNORM_BLOCK => TextureFormat::Bc5RgSnorm,
        VkFormat::BC6H_UFLOAT_BLOCK => TextureFormat::Bc6hRgbUfloat,
        VkFormat::BC6H_SFLOAT_BLOCK => TextureFormat::Bc6hRgbFloat,
        VkFormat::BC7_UNORM_BLOCK => TextureFormat::Bc7RgbaUnorm,
        VkFormat::BC7_SRGB_BLOCK => TextureFormat::Bc7RgbaUnormSrgb,

        // ETC2 (Android-class GPUs; needs TEXTURE_COMPRESSION_ETC2)
        VkFormat::ETC2_R8G8B8_UNORM_BLOCK => TextureFormat::Etc2Rgb8Unorm,
        VkFormat::ETC2_R8G8B8_SRGB_BLOCK => TextureFormat::Etc2Rgb8UnormSrgb,
        VkFormat::ETC2_R8G8B8A1_UNORM_BLOCK => TextureFormat::Etc2Rgb8A1Unorm,
        VkFormat::ETC2_R8G8B8A1_SRGB_BLOCK => TextureFormat::Etc2Rgb8A1UnormSrgb,
        VkFormat::ETC2_R8G8B8A8_UNORM_BLOCK => TextureFormat::Etc2Rgba8Unorm,
        VkFormat::ETC2_R8G8B8A8_SRGB_BLOCK => TextureFormat::Etc2Rgba8UnormSrgb,

        // ASTC 4×4 and 8×8 (mobile + WebGPU; needs TEXTURE_COMPRESSION_ASTC)
        VkFormat::ASTC_4x4_UNORM_BLOCK => astc(AstcBlock::B4x4, AstcChannel::Unorm),
        VkFormat::ASTC_4x4_SRGB_BLOCK => astc(AstcBlock::B4x4, AstcChannel::UnormSrgb),
        VkFormat::ASTC_8x8_UNORM_BLOCK => astc(AstcBlock::B8x8, AstcChannel::Unorm),
        VkFormat::ASTC_8x8_SRGB_BLOCK => astc(AstcBlock::B8x8, AstcChannel::UnormSrgb),

        _ => return None,
    })
}

/// Decode a KTX2 byte slice into a fully-uploaded [`TextureObject`].
///
/// Validation rules (any violation returns `TextureError::CreateTextureError`):
/// - `format` must be present (no Basis Universal — VK_FORMAT_UNDEFINED — yet).
/// - `supercompression_scheme` must be `None` (no zstd/zlib/Basis transcoding yet).
/// - `face_count` must be 1 (cube maps unsupported).
/// - `layer_count` must be 0 or 1 (array textures unsupported).
/// - `pixel_depth` must be 0 or 1 (3D textures unsupported).
/// - The format must map to a `wgpu::TextureFormat` we know about.
/// - The runtime device must actually support the chosen format. We honor the
///   adapter's advertised features (BC / ETC2 / ASTC are opportunistically
///   requested at device creation), but a desktop adapter without ASTC, for
///   example, will reject ASTC textures here with a clear error rather than
///   crashing inside wgpu validation.
///
/// Mipmap chain: every level present in the file is uploaded as-is. We do not
/// generate additional mips on top of the file's chain.
pub(crate) fn from_ktx2_bytes(
    context: &RenderContext,
    bytes: &[u8],
) -> Result<TextureObject, TextureError> {
    let reader = ktx2::Reader::new(bytes)
        .map_err(|e| TextureError::CreateTextureError(format!("Invalid KTX2 file: {:?}", e)))?;
    let header = reader.header();

    if header.supercompression_scheme.is_some() {
        return Err(TextureError::CreateTextureError(format!(
            "KTX2 supercompression {:?} is not supported (decompress upstream or use a non-supercompressed file)",
            header.supercompression_scheme
        )));
    }
    if header.face_count != 1 {
        return Err(TextureError::CreateTextureError(format!(
            "KTX2 cube maps (face_count = {}) are not supported yet",
            header.face_count
        )));
    }
    if header.layer_count > 1 {
        return Err(TextureError::CreateTextureError(format!(
            "KTX2 array textures (layer_count = {}) are not supported yet",
            header.layer_count
        )));
    }
    if header.pixel_depth > 1 {
        return Err(TextureError::CreateTextureError(format!(
            "KTX2 3D textures (pixel_depth = {}) are not supported yet",
            header.pixel_depth
        )));
    }

    let vk_format = header.format.ok_or_else(|| {
        TextureError::CreateTextureError(
            "KTX2 file has VK_FORMAT_UNDEFINED (Basis Universal supercompressed payloads are not transcoded yet)".into(),
        )
    })?;
    let wgpu_format = vk_format_to_wgpu(vk_format).ok_or_else(|| {
        TextureError::CreateTextureError(format!(
            "KTX2 format {:?} is not supported yet — extend vk_format_to_wgpu to add it",
            vk_format
        ))
    })?;

    // Verify the device actually supports this format (the relevant compression
    // feature may not have been granted by the adapter). Without this guard,
    // wgpu would surface a hard-to-trace validation error inside texture creation.
    let required_features = wgpu_format.required_features();
    let device_features = context.device.features();
    if !device_features.contains(required_features) {
        return Err(TextureError::CreateTextureError(format!(
            "KTX2 format {:?} requires GPU features {:?}, but the active adapter only advertises {:?}",
            vk_format,
            required_features,
            device_features & required_features
        )));
    }

    let width = header.pixel_width;
    let height = header.pixel_height.max(1);
    let level_count = header.level_count.max(1);

    if width == 0 {
        return Err(TextureError::CreateTextureError(
            "KTX2 file has zero width".into(),
        ));
    }

    // Honor `level_count = 0` (file says "regenerate the chain"): we treat that
    // as "the file ships level 0 only" and synthesize the rest if the format is
    // mip-friendly. For now we just upload what's there — if we want resampling
    // here later, dispatch to `write_image_levels` like the JPEG/PNG path.
    let stored_levels = level_count.min(mip_level_count_for(width, height));

    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
    crate::texture::check_format(context.device.features(), wgpu_format, usage)?;
    let descriptor = wgpu::TextureDescriptor {
        label: Some("KTX2 Source Texture"),
        size,
        mip_level_count: stored_levels,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu_format,
        view_formats: &[],
        usage,
    };
    let texture = context.device.create_texture(&descriptor);

    let (block_w, block_h) = wgpu_format.block_dimensions();
    let block_bytes = wgpu_format.block_copy_size(None).ok_or_else(|| {
        TextureError::CreateTextureError(format!(
            "KTX2 format {:?} has no defined block copy size — likely a depth/multi-aspect format that doesn't belong in a source texture",
            vk_format
        ))
    })?;
    let bpp_fallback = bytes_per_pixel(wgpu_format);

    for (level_index, level) in reader.levels().take(stored_levels as usize).enumerate() {
        let mip = level_index as u32;
        let level_w = (width >> mip).max(1);
        let level_h = (height >> mip).max(1);

        let blocks_in_row = level_w.div_ceil(block_w);
        let blocks_in_col = level_h.div_ceil(block_h);
        let bytes_per_row = if block_w == 1 && block_h == 1 {
            // Uncompressed: bytes_per_row is bpp * width.
            bpp_fallback * level_w
        } else {
            block_bytes * blocks_in_row
        };
        let expected_len = (bytes_per_row as u64) * (blocks_in_col as u64);
        if (level.data.len() as u64) < expected_len {
            return Err(TextureError::CreateTextureError(format!(
                "KTX2 level {} is {} bytes but the format expects at least {}",
                level_index,
                level.data.len(),
                expected_len
            )));
        }

        context.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: mip,
                origin: wgpu::Origin3d::ZERO,
            },
            &level.data[..expected_len as usize],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(blocks_in_col * block_h),
            },
            wgpu::Extent3d {
                width: level_w,
                height: level_h,
                depth_or_array_layers: 1,
            },
        );
    }

    let sampler = create_default_sampler(&context.device);
    Ok(TextureObject {
        inner: texture,
        size,
        sampler: RwLock::new(sampler),
        options: RwLock::new(SamplerOptions::default()),
        format: wgpu_format,
        usage,
    })
}

#[cfg(test)]
pub(crate) mod test_helpers {
    //! Minimal in-memory KTX2 builder for tests. Only constructs the subset
    //! the loader actually accepts (single layer, single face, 2D, no
    //! supercompression). The DFD is built with `ktx2::dfd::Basic::from_format`,
    //! which produces a spec-compliant block for any format the crate knows.

    use ktx2::dfd::{Basic, Block};
    use ktx2::{Format as VkFormat, Header, Index};

    pub struct Ktx2Test {
        pub format: VkFormat,
        pub width: u32,
        pub height: u32,
        /// Mip levels, ordered largest first (level 0). Each entry is the raw,
        /// uncompressed bytes for that level.
        pub levels: Vec<Vec<u8>>,
    }

    impl Ktx2Test {
        pub fn build(self) -> Vec<u8> {
            let (basic, type_size) = Basic::from_format(self.format).expect("supported format");
            let dfd_block = Block::Basic(basic);
            let dfd_block_bytes = dfd_block.to_vec();
            let dfd_total_size_field: u32 = (4 + dfd_block_bytes.len()) as u32;

            let level_count = self.levels.len() as u32;

            // Layout (offsets in bytes):
            //   header (80) | level index (24 * N) | DFD (4 + block) | KVD (0) | level data
            let level_index_size = 24usize * level_count as usize;
            let header_end = 80 + level_index_size;
            let dfd_offset = header_end;
            let dfd_byte_length = dfd_total_size_field;
            let dfd_end = dfd_offset + dfd_byte_length as usize;
            let kvd_offset = dfd_end;
            let kvd_byte_length: u32 = 0;
            let level_data_start = kvd_offset + kvd_byte_length as usize;

            // Pack levels in spec order (level N-1 first on disk, level 0 last) — this
            // matches what every well-formed KTX2 file we'd actually load looks like.
            // The reader returns levels in level-0-first order regardless.
            let mut level_offsets = Vec::with_capacity(level_count as usize);
            let mut cursor = level_data_start;
            for level in self.levels.iter().rev() {
                level_offsets.push((cursor, level.len()));
                cursor += level.len();
            }
            // Reverse so index 0 corresponds to mip level 0 (largest).
            level_offsets.reverse();
            let total_size = cursor;

            let header = Header {
                format: Some(self.format),
                type_size,
                pixel_width: self.width,
                pixel_height: self.height,
                pixel_depth: 0,
                layer_count: 0,
                face_count: 1,
                level_count,
                supercompression_scheme: None,
                index: Index {
                    dfd_byte_offset: dfd_offset as u32,
                    dfd_byte_length,
                    kvd_byte_offset: if kvd_byte_length > 0 {
                        kvd_offset as u32
                    } else {
                        0
                    },
                    kvd_byte_length,
                    sgd_byte_offset: 0,
                    sgd_byte_length: 0,
                },
            };

            let mut out = vec![0u8; total_size];
            out[..80].copy_from_slice(&header.as_bytes());

            for (i, (offset, length)) in level_offsets.iter().enumerate() {
                let entry_offset = 80 + i * 24;
                out[entry_offset..entry_offset + 8]
                    .copy_from_slice(&(*offset as u64).to_le_bytes());
                out[entry_offset + 8..entry_offset + 16]
                    .copy_from_slice(&(*length as u64).to_le_bytes());
                out[entry_offset + 16..entry_offset + 24]
                    .copy_from_slice(&(*length as u64).to_le_bytes());
            }

            // DFD: 4-byte total length field + block bytes.
            out[dfd_offset..dfd_offset + 4].copy_from_slice(&dfd_total_size_field.to_le_bytes());
            out[dfd_offset + 4..dfd_offset + 4 + dfd_block_bytes.len()]
                .copy_from_slice(&dfd_block_bytes);

            // Level data
            for (level, (offset, length)) in self.levels.iter().zip(level_offsets.iter()) {
                out[*offset..*offset + *length].copy_from_slice(level);
            }

            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helpers::Ktx2Test;

    // Story: a synthetic uncompressed-RGBA8 KTX2 with one mip level loads with
    // the right format, dimensions, and mip count. The default RGBA8 path
    // doesn't need any compression features so it works on every adapter.
    #[test]
    fn loads_uncompressed_rgba8_single_level() {
        pollster::block_on(async move {
            let pixels: Vec<u8> = (0..(4 * 4 * 4)).map(|i| i as u8).collect();
            let bytes = Ktx2Test {
                format: VkFormat::R8G8B8A8_UNORM,
                width: 4,
                height: 4,
                levels: vec![pixels],
            }
            .build();

            let renderer = crate::Renderer::new();
            let tex = renderer
                .create_texture(crate::texture::TextureData::Ktx2Bytes(bytes))
                .await
                .expect("load uncompressed KTX2");
            assert_eq!(tex.object.format, wgpu::TextureFormat::Rgba8Unorm);
            assert_eq!(tex.object.inner.mip_level_count(), 1);
            assert_eq!(tex.size().width, 4);
            assert_eq!(tex.size().height, 4);
        });
    }

    // Story: pre-baked mip chain in the file is honored as-is — three levels in,
    // three levels out, and we never call our CPU resize path.
    #[test]
    fn honors_prebaked_mip_chain() {
        pollster::block_on(async move {
            let l0: Vec<u8> = vec![0xAA; 4 * 4 * 4];
            let l1: Vec<u8> = vec![0xBB; 2 * 2 * 4];
            let l2: Vec<u8> = vec![0xCC; 4];
            let bytes = Ktx2Test {
                format: VkFormat::R8G8B8A8_SRGB,
                width: 4,
                height: 4,
                levels: vec![l0, l1, l2],
            }
            .build();

            let renderer = crate::Renderer::new();
            let tex = renderer
                .create_texture(crate::texture::TextureData::Ktx2Bytes(bytes))
                .await
                .expect("load mipped KTX2");
            assert_eq!(tex.object.format, wgpu::TextureFormat::Rgba8UnormSrgb);
            assert_eq!(tex.object.inner.mip_level_count(), 3);
        });
    }

    // Story: cube maps and arrays are out of scope; the loader rejects them with
    // a clear message instead of producing a half-broken texture.
    #[test]
    fn rejects_cube_map() {
        pollster::block_on(async move {
            let pixels: Vec<u8> = vec![0; 4 * 4 * 4 * 6];
            let mut bytes = Ktx2Test {
                format: VkFormat::R8G8B8A8_UNORM,
                width: 4,
                height: 4,
                levels: vec![pixels],
            }
            .build();
            // Patch face_count = 6 directly (Ktx2Test always emits 1).
            bytes[36..40].copy_from_slice(&6u32.to_le_bytes());

            let renderer = crate::Renderer::new();
            let err = renderer
                .create_texture(crate::texture::TextureData::Ktx2Bytes(bytes))
                .await
                .expect_err("cube maps should be rejected");
            let msg = err.to_string();
            assert!(
                msg.contains("cube"),
                "error should mention cube maps, got: {msg}"
            );
        });
    }
}
