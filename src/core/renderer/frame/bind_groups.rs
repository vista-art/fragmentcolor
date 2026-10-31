//! The bind group parts every pass builds,
//! and the default maps a PBR material binds for the ones it leaves unset.

use crate::core::renderer::pipelines::PushMode;
use crate::core::renderer::{RenderContext, RendererError, pools};
use crate::texture::TextureMeta;
use crate::{ShaderError, ShaderObject};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct PbrDefaults {
    /// Five views the renderer binds to the PBR-named slots
    /// when a Material::pbr Shader doesn't have a user-supplied texture for
    /// the slot. The byte payload is chosen so `factor * sample = factor`
    /// (binding the default is equivalent to "no map").
    pub(crate) base_color: wgpu::TextureView,
    pub(crate) metallic_roughness: wgpu::TextureView,
    pub(crate) normal: wgpu::TextureView,
    pub(crate) occlusion: wgpu::TextureView,
    pub(crate) emissive: wgpu::TextureView,
}

impl PbrDefaults {
    /// Whether `uniform_name` is a PBR map slot, which takes a default when no texture is set.
    pub(in crate::core::renderer) fn has_slot(uniform_name: &str) -> bool {
        matches!(
            uniform_name,
            "base_color_map"
                | "metallic_roughness_map"
                | "normal_map"
                | "occlusion_map"
                | "emissive_map"
        )
    }

    pub(super) fn slot(&self, uniform_name: &str) -> Option<&wgpu::TextureView> {
        match uniform_name {
            "base_color_map" => Some(&self.base_color),
            "metallic_roughness_map" => Some(&self.metallic_roughness),
            "normal_map" => Some(&self.normal),
            "occlusion_map" => Some(&self.occlusion),
            "emissive_map" => Some(&self.emissive),
            _ => None,
        }
    }
}

impl RenderContext {
    /// Borrow the lazy 1x1 PBR fallback textures, creating them on first use.
    /// Used to fill the default PBR shader's texture-map bindings when a
    /// `Material::pbr` doesn't supply its own texture for the slot. Pure
    /// sync calls against the wgpu device — no FC texture registry, no
    /// async hop.
    pub(super) fn pbr_defaults(&self) -> Arc<PbrDefaults> {
        if let Some(existing) = self.pbr_defaults.read().clone() {
            return existing;
        }
        let make = |label: &'static str, rgba: [u8; 4]| -> wgpu::TextureView {
            let tex = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
            tex.create_view(&wgpu::TextureViewDescriptor::default())
        };
        // Defaults chosen so `factor * sample = factor` in the shader.
        let bundle = Arc::new(PbrDefaults {
            base_color: make("PBR default base_color", [255, 255, 255, 255]),
            // glTF stores metallic in .b and roughness in .g; the shader's
            // .bgr swizzle puts metallic in .r (=1) and roughness in .g (=1).
            metallic_roughness: make("PBR default metallic_roughness", [0, 255, 255, 255]),
            // Neutral tangent-space normal (0, 0, 1) after the * 2.0 - 1.0 decode.
            normal: make("PBR default normal", [128, 128, 255, 255]),
            occlusion: make("PBR default occlusion", [255, 255, 255, 255]),
            emissive: make("PBR default emissive", [255, 255, 255, 255]),
        });
        *self.pbr_defaults.write() = Some(bundle.clone());
        bundle
    }

    /// Uploads the immediate root `name` when `push` lowers it to a uniform, and
    /// returns the group, binding and pool location it binds at. Native immediates upload nothing.
    pub(super) fn upload_push(
        &self,
        shader: &ShaderObject,
        name: &str,
        push: Option<&PushMode>,
    ) -> Result<Option<(u32, u32, pools::BufferLocation)>, RendererError> {
        let Some(PushMode::Fallback { group, bindings }) = push else {
            return Ok(None);
        };
        let Some(binding) = bindings.get(name) else {
            return Ok(None);
        };
        let storage = shader.storage.read();
        let bytes = storage
            .get_bytes(name)
            .ok_or_else(|| ShaderError::UniformNotFound(name.to_string()))?;
        let location = self
            .buffer_pool
            .write()
            .upload(bytes, &self.queue, &self.device);
        Ok(Some((*group, *binding, location)))
    }

    /// An empty bind group for each group of `layouts` that no binding fills,
    /// so each group index the pipeline layout holds gets a group.
    pub(super) fn create_empty_groups(
        &self,
        layouts: &HashMap<u32, wgpu::BindGroupLayout>,
        present: &HashSet<u32>,
    ) -> Result<Vec<(u32, wgpu::BindGroup)>, RendererError> {
        let mut groups = Vec::new();
        for (group, layout) in layouts.iter() {
            if present.contains(group) {
                continue;
            }
            let label = format!("Empty Bind Group for group: {}", group);
            let empty = self.validate_step(&label, || {
                Ok(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout,
                    entries: &[],
                    label: Some(&label),
                }))
            })?;
            groups.push((*group, empty));
        }
        Ok(groups)
    }
}

/// The error for a texture uniform whose texture the registry doesn't hold:
/// one never set, or one `unregister_texture` removed.
pub(in crate::core::renderer) fn report_missing(name: &str, meta: &TextureMeta) -> RendererError {
    match meta.id.id {
        0 => RendererError::BindingError {
            uniform: name.to_string(),
            expected: "a texture".into(),
            found: "none, since it was never set".into(),
        },
        _ => RendererError::TextureNotFoundError(meta.id),
    }
}
