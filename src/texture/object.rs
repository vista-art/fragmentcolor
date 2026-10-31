//! `TextureObject`: the wgpu texture, view and sampler behind every handle,
//! and how one is built from a source, raw bytes, an image or a mip chain.

use std::sync::Arc;

use image::{DynamicImage, GenericImageView};
use parking_lot::RwLock;

use super::loading::decode::{Pick, check_source, decode, infer_format, pick_override, read_file};
use super::loading::ktx2_loader::{self, read_ktx2_file};
use super::loading::{self, SourceData, TextureSource};
use super::write::{write_image_levels, write_levels};
use super::{
    CompareFunction, Mipmap, SamplerOptions, TextureError, TextureOptions, check_decode,
    check_extent, check_format, check_upload, check_usage, create_default_sampler, create_sampler,
    measure_texel, mip_level_count_for, supports_cpu_mipmaps,
};
use crate::{RenderContext, Size};

#[derive(Debug)]
pub(crate) struct TextureObject {
    pub(crate) inner: wgpu::Texture,
    pub(crate) size: wgpu::Extent3d,
    pub(crate) sampler: RwLock<wgpu::Sampler>,
    pub(crate) options: RwLock<SamplerOptions>,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) usage: wgpu::TextureUsages,
}

impl TextureObject {
    /// Create a texture with an explicit usage mask, such as a storage texture.
    /// It fails with a typed error when the device can't make `format` with `usage`.
    pub fn new(
        context: &RenderContext,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        options: SamplerOptions,
    ) -> Result<Self, TextureError> {
        check_format(context, format, usage, pick_dimension(size))?;
        let descriptor = Self::texture_descriptor("Generic Texture", size, format, usage, 1, 1); // @TODO mip_count, sample_count
        let inner = context.device.create_texture(&descriptor);
        let size = inner.size();
        let sampler = create_sampler(&context.device, options);
        Ok(Self {
            inner,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(options),
            format,
            usage,
        })
    }

    /// Builds a texture on `context` from `source`, which the call shares and never copies.
    ///
    /// A URL is fetched on the calling thread, and its bytes go to the worker without
    /// being kept. The decode and the GPU upload run on the background worker on native
    /// and inline on the web. `Renderer::create_texture` and each Material map upload call it,
    /// and `call` names the one that did in an error that refuses its format.
    pub(crate) async fn from_source(
        context: Arc<RenderContext>,
        source: TextureSource,
        call: &'static str,
    ) -> Result<Self, TextureError> {
        let options = source.options.clone();
        let pick = Pick::read(&source, call);
        let label = source.label.clone();
        check_source(source.data.is_sized(), options.size)?;
        let usage = source_usage(options.usage)?;
        let fetched = match &*source.data {
            SourceData::Url(url) | SourceData::Ktx2Url(url) => {
                crate::network::fetch_bytes(url).await.map_err(|e| {
                    TextureError::CreateTextureError(format!("URL fetch failed: {e:?}"))
                })?
            }
            _ => Vec::new(),
        };

        loading::worker::run(move || -> Result<Self, TextureError> {
            let context = context.as_ref();
            let object = match &*source.data {
                SourceData::Bytes(bytes) => Self::from_bytes(context, bytes, &options, pick, usage),
                SourceData::Url(_) => Self::from_bytes(context, &fetched, &options, pick, usage),
                SourceData::Ktx2Bytes(bytes) => {
                    ktx2_loader::from_ktx2_bytes(context, bytes, pick, usage)
                }
                SourceData::Ktx2Url(_) => {
                    ktx2_loader::from_ktx2_bytes(context, &fetched, pick, usage)
                }
                SourceData::Path(path) if ktx2_loader::is_ktx2_file(path) => {
                    read_ktx2_file(context, path, pick, usage)
                }
                SourceData::Path(path) => {
                    let limit = context.device.limits().max_texture_dimension_2d;
                    let image = decode(&read_file(path)?, limit)?;
                    let format = pick_override(&options);
                    Self::from_image(context, &image, format, options.mipmaps, usage)
                }
                SourceData::Ktx2Path(path) => read_ktx2_file(context, path, pick, usage),
                SourceData::Prepared(chain) => {
                    Self::from_chain(context, chain.clone(), pick, usage)
                }
            }?;
            object.set_sampler_options(&context.device, options.sampler);
            Ok(object)
        })
        .await
        .map_err(|error| match &label {
            Some(label) => error.name(label),
            None => error,
        })
    }

    /// Builds a texture from `bytes`: a KTX2 container by its magic, which takes `pick`,
    /// raw pixels when `options` gives a size, or else an encoded image the worker decodes.
    fn from_bytes(
        context: &RenderContext,
        bytes: &[u8],
        options: &TextureOptions,
        pick: Pick,
        usage: wgpu::TextureUsages,
    ) -> Result<Self, TextureError> {
        if ktx2_loader::is_ktx2(bytes) {
            return ktx2_loader::from_ktx2_bytes(context, bytes, pick, usage);
        }
        let format = pick_override(options);
        match options.size {
            Some(size) => {
                let format = format.unwrap_or_else(|| options.format.to_wgpu());
                Self::from_raw_bytes(
                    context,
                    size.to_extent(),
                    format,
                    bytes,
                    options.mipmaps,
                    usage,
                )
            }
            None => {
                let limit = context.device.limits().max_texture_dimension_2d;
                let image = decode(bytes, limit)?;
                Self::from_image(context, &image, format, options.mipmaps, usage)
            }
        }
    }

    /// Creates a texture from raw pixel bytes with explicit size/format.
    /// The data layout is tightly packed with bytes_per_row = bpp * width.
    ///
    /// When `generate_mipmaps = true`, a full mip chain is built CPU-side via the
    /// dispatcher in [`write_levels`], for every format `supports_cpu_mipmaps`
    /// accepts, the float formats included. Other formats, and a 3D size, get one level.
    fn from_raw_bytes(
        context: &RenderContext,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        data: &[u8],
        generate_mipmaps: bool,
        usage: wgpu::TextureUsages,
    ) -> Result<Self, TextureError> {
        check_format(context, format, usage, pick_dimension(size))?;
        check_upload(format)?;
        check_extent(&context.device.limits(), size, pick_dimension(size))?;
        let bpp = measure_texel(format)?;

        let expected = (size.width as usize)
            .saturating_mul(size.height as usize)
            .saturating_mul(size.depth_or_array_layers as usize)
            .saturating_mul(bpp as usize);
        if data.len() != expected {
            let wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: depth,
            } = size;
            return Err(TextureError::InvalidInput(format!(
                "raw bytes are {} long, and {width}x{height}x{depth} {format:?} takes {expected}",
                data.len()
            )));
        }

        // CPU mipmaps resample one slice, so a 3D texture keeps one level and every slice.
        let flat = size.depth_or_array_layers == 1;
        let want_mips = generate_mipmaps && flat && supports_cpu_mipmaps(format);
        let mip_level_count = if want_mips {
            mip_level_count_for(size.width, size.height)
        } else {
            1
        };

        let descriptor =
            Self::texture_descriptor("Raw Texture", size, format, usage, 1, mip_level_count);
        let texture = context.device.create_texture(&descriptor);

        write_levels(context, data, &texture, size, format, mip_level_count)?;

        let sampler = create_default_sampler(&context.device);
        Ok(Self {
            inner: texture,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(SamplerOptions::default()),
            format,
            usage,
        })
    }

    /// Internal method to create a TextureObject from a DynamicImage instance.
    ///
    /// `format_override = None` infers the format from `image.color()`, and any
    /// format outside `supports_cpu_mipmaps` is refused. The image is decoded again
    /// to match the format, so `R16Unorm` goes through `to_luma16()` and keeps 16 bits.
    /// Mips of a unorm format come from `image::imageops::resize` with the Triangle filter.
    /// The float formats take [`downsample_floats`] instead, since that resize clamps them to 0..1.
    fn from_image(
        context: &RenderContext,
        image: &DynamicImage,
        format_override: Option<wgpu::TextureFormat>,
        generate_mipmaps: bool,
        usage: wgpu::TextureUsages,
    ) -> Result<Self, TextureError> {
        let label = "Source Texture from Loaded Image";
        let (width, height) = image.dimensions();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let format = format_override.unwrap_or_else(|| infer_format(image.color()));
        check_format(context, format, usage, pick_dimension(size))?;
        check_upload(format)?;
        check_decode(format)?;
        check_extent(&context.device.limits(), size, pick_dimension(size))?;

        let want_mips = generate_mipmaps && supports_cpu_mipmaps(format);
        let mip_level_count = if want_mips {
            mip_level_count_for(width, height)
        } else {
            1
        };
        let descriptor = Self::texture_descriptor(label, size, format, usage, 1, mip_level_count);
        let texture = context.device.create_texture(&descriptor);

        write_image_levels(context, image, &texture, format, mip_level_count)?;

        let sampler = create_default_sampler(&context.device);

        Ok(Self {
            inner: texture,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(SamplerOptions::default()),
            format,
            usage,
        })
    }

    /// Creates a texture from a mip chain built on the CPU, with GPU writes only,
    /// in the chain's own format or the one `pick` asks for where it reads the same bytes.
    fn from_chain(
        context: &RenderContext,
        chain: Mipmap,
        pick: Pick,
        usage: wgpu::TextureUsages,
    ) -> Result<Self, TextureError> {
        let (w, h) = chain.base_size;
        if w == 0 || h == 0 {
            return Err(TextureError::CreateTextureError(
                "Mipmap size must be non-zero".into(),
            ));
        }
        if chain.levels.is_empty() {
            return Err(TextureError::CreateTextureError(
                "Mipmap must contain at least one level".into(),
            ));
        }
        let levels = chain.levels.as_slice();
        let format = pick.apply(chain.format, "a Mipmap")?;
        check_format(context, format, usage, wgpu::TextureDimension::D2)?;
        let bpp = measure_texel(format)?;

        for (i, level_bytes) in levels.iter().enumerate() {
            let level_w = (w >> i).max(1);
            let level_h = (h >> i).max(1);
            let expected = (bpp as usize) * (level_w as usize) * (level_h as usize);
            if level_bytes.len() != expected {
                return Err(TextureError::CreateTextureError(format!(
                    "Mipmap level {} has {} bytes, expected {} ({}x{} @ {} bpp)",
                    i,
                    level_bytes.len(),
                    expected,
                    level_w,
                    level_h,
                    bpp
                )));
            }
        }

        let size = wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        };
        check_extent(&context.device.limits(), size, pick_dimension(size))?;
        let mip_level_count = levels.len() as u32;
        let descriptor =
            Self::texture_descriptor("Prepared Texture", size, format, usage, 1, mip_level_count);
        let texture = context.device.create_texture(&descriptor);

        for (i, level_bytes) in levels.iter().enumerate() {
            let level_w = (w >> i).max(1);
            let level_h = (h >> i).max(1);
            context.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: i as u32,
                    origin: wgpu::Origin3d::ZERO,
                },
                level_bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bpp * level_w),
                    rows_per_image: Some(level_h),
                },
                wgpu::Extent3d {
                    width: level_w,
                    height: level_h,
                    depth_or_array_layers: 1,
                },
            );
        }

        let sampler = create_default_sampler(&context.device);
        Ok(Self {
            inner: texture,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(SamplerOptions::default()),
            format,
            usage,
        })
    }

    /// Internal method to create a Texture marked as a destination for rendering
    pub fn create_destination_texture(
        context: &RenderContext,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
    ) -> Self {
        let label = "Render Target Texture";
        let descriptor = Self::target_texture_descriptor(label, size, format);
        let texture = context.device.create_texture(&descriptor);
        let sampler = create_default_sampler(&context.device);

        Self {
            inner: texture,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(SamplerOptions::default()),
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
        }
    }

    /// We need the DEPTH_FORMAT for when we create the depth stage of
    /// the render_pipeline and for creating the depth texture itself.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// Creates a depth texture with the given MSAA `sample_count`. Pass
    /// `context.sample_count()` to inherit the renderer's negotiated count;
    /// pass `1` for a non-MSAA depth attachment.
    pub fn create_depth_texture(
        context: &RenderContext,
        size: wgpu::Extent3d,
        sample_count: u32,
    ) -> Self {
        let format = Self::DEPTH_FORMAT;
        let descriptor = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: sample_count.max(1),
            dimension: pick_dimension(size),
            format,
            view_formats: &[],
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };
        let texture = context.device.create_texture(&descriptor);
        let options = SamplerOptions {
            compare: Some(CompareFunction::LessEqual),
            ..Default::default()
        };
        let sampler = create_sampler(&context.device, options);

        Self {
            inner: texture,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(options),
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        }
    }

    /// The sampler a `sampler` binding gets from this texture.
    /// It is the texture's own sampler when the binding's `comparison` matches the options,
    /// and otherwise a sampler built from the same options with the comparison switched on or off.
    pub fn sampler_for(&self, device: &wgpu::Device, comparison: bool) -> wgpu::Sampler {
        let options = self.options.read();
        if options.compare.is_some() == comparison {
            return self.sampler.read().clone();
        }
        let compare = comparison.then_some(CompareFunction::LessEqual);
        create_sampler(
            device,
            SamplerOptions {
                compare,
                ..*options
            },
        )
    }

    pub fn set_sampler_options(&self, device: &wgpu::Device, options: SamplerOptions) {
        let sampler = create_sampler(device, options);
        // Hold the options lock across both writes so `sampler_for` never sees a mismatched pair.
        let mut current = self.options.write();
        *self.sampler.write() = sampler;
        *current = options;
    }

    pub fn size(&self) -> Size {
        Size::from_extent(self.size, self.inner.dimension())
    }

    pub fn aspect(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn create_view(&self) -> wgpu::TextureView {
        self.inner
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    /// This texture at `size`, with the same format, usage, sample count and
    /// sampler, and as many of its mip levels as fit. Its pixels start blank.
    pub(crate) fn create_resized(&self, context: &RenderContext, size: wgpu::Extent3d) -> Self {
        let label = match self.format.is_depth_stencil_format() {
            true => "Depth Texture",
            false => "Render Target Texture",
        };
        let levels = mip_level_count_for(size.width, size.height);
        let descriptor = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: self.inner.mip_level_count().min(levels),
            sample_count: self.inner.sample_count(),
            dimension: self.inner.dimension(),
            format: self.format,
            view_formats: &[],
            usage: self.usage,
        };
        let options = self.options.read();
        Self {
            inner: context.device.create_texture(&descriptor),
            size,
            sampler: RwLock::new(self.sampler.read().clone()),
            options: RwLock::new(*options),
            format: self.format,
            usage: self.usage,
        }
    }

    /// Creates a texture descriptor for a Target Texture
    fn target_texture_descriptor(
        label: &'static str,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
    ) -> wgpu::TextureDescriptor<'static> {
        let sample_count = 1;
        let mip_level_count = 1;
        Self::texture_descriptor(
            label,
            size,
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            sample_count,
            mip_level_count,
        )
    }

    /// Creates a texture descriptor
    fn texture_descriptor(
        label: &'static str,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        sample_count: u32,
        mip_level_count: u32,
    ) -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count,
            sample_count,
            dimension: pick_dimension(size),
            format,
            view_formats: &[],
            usage,
        }
    }
}

/// Usage of a texture made from source data.
/// It always holds `TEXTURE_BINDING | COPY_DST`, since the upload and sampling need them,
/// plus any bits from `TextureOptions::usage`, such as `COPY_SRC` for `read_texture`.
pub(crate) fn source_usage(extra: Option<u32>) -> Result<wgpu::TextureUsages, TextureError> {
    let required = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
    let extra = extra.map(check_usage).transpose()?;
    Ok(required | extra.unwrap_or(wgpu::TextureUsages::empty()))
}

/// The dimension a texture of `size` gets: 2D with one layer, and 3D with more.
pub(crate) fn pick_dimension(size: wgpu::Extent3d) -> wgpu::TextureDimension {
    match size.depth_or_array_layers {
        1 => wgpu::TextureDimension::D2,
        _ => wgpu::TextureDimension::D3,
    }
}
