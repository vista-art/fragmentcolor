//! The textures a renderer creates and holds by `TextureId`,
//! from upload and readback to how one leaves the registry.

use crate::core::renderer::{InitializationError, RenderContext, Renderable, Renderer};
use crate::core::renderer::{RendererError, Start};
use crate::texture::{Registration, Texture, TextureError, TextureId, TextureInput, TextureObject};
use crate::texture::{TextureSource, check_extent, check_plane, check_upload, check_usage};
use crate::texture::{copy_pixels, measure_texel, pick_dimension};
use crate::{PassObject, Size, TextureData};
use std::mem::discriminant;
use std::sync::Arc;

impl Renderer {
    /// A texture from `input`, checked before the context opens, with its CPU work on a worker.
    pub(super) async fn upload_texture(
        &self,
        input: TextureInput,
    ) -> Result<Texture, RendererError> {
        input.check_options("create_texture")?;

        // CloneOf returns the texture it holds, and Empty has nothing to upload.
        if let TextureData::CloneOf(texture) = input.data {
            return Ok(texture);
        }
        if matches!(input.data, TextureData::Empty) {
            return Err(RendererError::CreateTextureError(
                "create_texture requires source data; for an empty allocation use create_storage_texture(input)".into(),
            ));
        }

        let source = TextureSource::try_from(input)?;
        let context = self.context(None).await?;
        let object = TextureObject::from_source(context.clone(), source, "create_texture").await?;
        let object = Arc::new(object);
        let registration = Registration::create(&context, object);
        Ok(Texture::new(context, registration))
    }

    /// A storage texture of `input`'s size, format and usage, seeded with its bytes if it has any.
    pub(super) async fn allocate_storage(
        &self,
        input: TextureInput,
    ) -> Result<Texture, RendererError> {
        let TextureInput { data, options } = input;
        let context = self.context(None).await?;
        let size = options.size.ok_or_else(|| {
            RendererError::CreateTextureError(
                "create_storage_texture requires options.size — pass `(size, format)` or set the field explicitly".into(),
            )
        })?;
        let extent: wgpu::Extent3d = size.to_extent();
        let format: wgpu::TextureFormat = options.format.to_wgpu();
        // The size comes first, so a bad one reports the same error whatever the seed.
        let limits = context.device.limits();
        check_extent(&limits, extent, pick_dimension(extent))?;
        let usage = options.usage.map(check_usage).transpose()?;
        let usage = usage.unwrap_or(
            wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
        );
        if usage.is_empty() {
            return Err(TextureError::InvalidInput(
                "create_storage_texture needs at least one usage flag".into(),
            )
            .into());
        }

        let seed: Option<Vec<u8>> = match data {
            TextureData::Empty => None,
            TextureData::Bytes(bytes) => Some(bytes),
            other => {
                return Err(RendererError::CreateTextureError(format!(
                    "create_storage_texture only accepts TextureData::Empty or Bytes for now (got {:?}); decode external sources first then pass the raw bytes",
                    discriminant(&other)
                )));
            }
        };

        if let Some(ref bytes) = seed {
            check_upload(format)?;
            if !usage.contains(wgpu::TextureUsages::COPY_DST) {
                return Err(RendererError::CreateTextureError(
                    "create_storage_texture with seed data requires COPY_DST in the usage mask"
                        .into(),
                ));
            }
            let texel = measure_texel(format)?;
            let expected = (extent.width as usize)
                .saturating_mul(extent.height as usize)
                .saturating_mul(extent.depth_or_array_layers.max(1) as usize)
                .saturating_mul(texel as usize);
            if bytes.len() != expected {
                return Err(TextureError::InvalidInput(format!(
                    "seed data is {} bytes, and the texture takes {expected}",
                    bytes.len()
                ))
                .into());
            }
        }

        let object = TextureObject::new(context.as_ref(), extent, format, usage, options.sampler)?;

        if let Some(bytes) = seed {
            let texel = measure_texel(format)?;
            context.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &object.inner,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                &bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(texel * extent.width),
                    rows_per_image: Some(extent.height),
                },
                extent,
            );
        }
        let object = Arc::new(object);
        let registration = Registration::create(&context, object);
        Ok(Texture::new(context, registration))
    }

    /// A `Depth32Float` texture of `size` at the context's sample count.
    pub(super) async fn allocate_depth(&self, size: Size) -> Result<Texture, InitializationError> {
        let context = self.context(None).await?;
        let limit = context.device.limits().max_texture_dimension_2d;
        check_plane(size, limit)?;
        let sample_count = context.sample_count();
        let object =
            TextureObject::create_depth_texture(context.as_ref(), size.to_extent(), sample_count);
        let object = Arc::new(object);
        let registration = Registration::create(&context, object);
        Ok(Texture::new(context, registration))
    }

    /// Reads a registered texture once the frames rendered before `start` have reported their GPU errors.
    pub(crate) async fn read_texture_from(
        &self,
        texture_id: TextureId,
        start: Start,
    ) -> Result<Vec<u8>, RendererError> {
        let context = self
            .context
            .read()
            .as_ref()
            .cloned()
            .ok_or(RendererError::NoContext)?;
        #[cfg(wasm)]
        start.settle(&context).await?;
        #[cfg(not(wasm))]
        let _ = start;
        let texture = context
            .get_texture(&texture_id)
            .ok_or(RendererError::TextureNotFoundError(texture_id))?;
        Ok(copy_pixels(&context, &texture).await?)
    }

    /// Uploads what `renderable` waits for, once the frames rendered before `start`
    /// have reported their GPU errors.
    pub(crate) async fn load_from(
        &self,
        renderable: &impl Renderable,
        start: Start,
    ) -> Result<(), RendererError> {
        #[cfg(wasm)]
        {
            let context = self.context.read().clone();
            if let Some(context) = context {
                start.settle(&context).await?;
            }
        }
        #[cfg(not(wasm))]
        let _ = start;
        self.upload_maps(renderable).await
    }

    /// Uploads into this Renderer each Material map of `renderable` it lacks or holds
    /// from an older set. Each upload that works is kept, whatever happens to the others.
    /// A failure keeps no copy, and the first one returns, so every `load` and native `render`
    /// tries the map again until its source is fixed or replaced. A set during an upload wins.
    async fn upload_maps(&self, renderable: &impl Renderable) -> Result<(), RendererError> {
        let passes = renderable.passes();
        let mapped =
            |pass: &Arc<PassObject>| pass.shaders.read().iter().any(|shader| shader.has_maps());
        if !passes.iter().any(mapped) {
            return Ok(());
        }
        let context = self.context(None).await?;
        let mut jobs = Vec::new();
        for pass in passes.iter() {
            for shader in pass.shaders.read().iter() {
                shader.find_jobs(&context, &mut jobs);
            }
        }
        let mut first = None;
        for job in jobs {
            let upload = TextureObject::from_source(context.clone(), job.source.clone(), "load");
            let upload = upload.await;
            #[cfg(test)]
            crate::testing::race_hooks::run("map uploaded");
            match upload {
                Ok(object) => {
                    job.shader.keep_upload(&job, &context, object);
                }
                Err(error) if job.shader.is_current(&job) => {
                    first.get_or_insert(RendererError::from(error));
                }
                Err(_) => {}
            }
        }
        first.map_or(Ok(()), Err)
    }
}

impl RenderContext {
    /// Adds `texture` under a new id. Ids are unique across renderers, so a
    /// shader binding finds the one texture its id names, except a Material map's,
    /// which names one copy in each Renderer that loaded it. A `Registration`
    /// calls this and removes the entry when the last handle drops.
    pub(crate) fn register_texture(&self, texture: Arc<TextureObject>) -> TextureId {
        let id = TextureId::create();
        self.textures.insert(id, texture);
        id
    }

    /// Puts `texture` under `id`, as this context's copy of a Material map,
    /// and returns the copy it replaces, which the caller drops after its locks.
    pub(crate) fn insert_texture(
        &self,
        id: TextureId,
        texture: Arc<TextureObject>,
    ) -> Option<Arc<TextureObject>> {
        self.textures.insert(id, texture)
    }

    /// Whether the registry holds a texture under `id`.
    pub(crate) fn has_texture(&self, id: &TextureId) -> bool {
        self.textures.contains_key(id)
    }

    pub(crate) fn release_texture(&self, id: &TextureId) {
        self.textures.remove(id);
    }

    pub(crate) fn get_texture(&self, id: &TextureId) -> Option<Arc<TextureObject>> {
        self.textures.get(id).map(|texture| texture.clone())
    }

    /// Points a resized texture's id at its new object, so every pass target
    /// and shader binding that resolves the id at render time draws the new one.
    /// An id that `unregister_texture` removed stays removed.
    pub(crate) fn update_texture(&self, id: TextureId, texture: Arc<TextureObject>) {
        if let Some(mut entry) = self.textures.get_mut(&id) {
            *entry = texture;
        }
    }
}

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
