//! Reads each glTF material into a `Material`, with the sampler
//! and the UV set and transform each of its texture slots reads.

use super::SceneLoadError;
use crate::Material;
use crate::scene::{TextureSlot, UvSet, UvTransform};
use crate::texture::TextureSource;

/// Translate a glTF texture's sampler into FragmentColor's
/// [`SamplerOptions`]. glTF's `MIRRORED_REPEAT` collapses to `REPEAT`
/// (FragmentColor's sampler doesn't expose mirror today); mipmap-filter
/// variants of `min_filter` collapse to their base filter — the upload
/// path runs its own mipmap chain decision based on `options.mipmaps`.
pub(super) fn map_sampler_options(sampler: &gltf::texture::Sampler<'_>) -> crate::SamplerOptions {
    use gltf::texture::{MagFilter, MinFilter, WrappingMode};
    let smooth = match (sampler.mag_filter(), sampler.min_filter()) {
        // `Nearest` mag-filter is the strongest signal for pixel-art /
        // texel-art assets; respect it. Linear (or unspecified) keeps
        // the FragmentColor default smooth=true.
        (Some(MagFilter::Nearest), _) => false,
        (_, Some(MinFilter::Nearest))
        | (_, Some(MinFilter::NearestMipmapNearest))
        | (_, Some(MinFilter::NearestMipmapLinear)) => false,
        _ => true,
    };
    let repeat_x = matches!(
        sampler.wrap_s(),
        WrappingMode::Repeat | WrappingMode::MirroredRepeat
    );
    let repeat_y = matches!(
        sampler.wrap_t(),
        WrappingMode::Repeat | WrappingMode::MirroredRepeat
    );
    crate::SamplerOptions {
        repeat_x,
        repeat_y,
        smooth,
        compare: None,
    }
}

/// Build the PBR Material of a glTF material. Each texture slot becomes a map
/// of its image's source in `images`, which every slot that reads the image shares.
pub(super) fn build_material(
    gltf_material: &gltf::Material<'_>,
    images: &[TextureSource],
) -> Result<Material, SceneLoadError> {
    let pbr = gltf_material.pbr_metallic_roughness();
    let mut material = Material::pbr()
        .base_color(pbr.base_color_factor())
        .metallic(pbr.metallic_factor())
        .roughness(pbr.roughness_factor())
        .emissive(gltf_material.emissive_factor())
        .alpha_cutoff(gltf_material.alpha_cutoff().unwrap_or(0.5))
        .alpha_mode(match gltf_material.alpha_mode() {
            gltf::material::AlphaMode::Opaque => crate::AlphaMode::Opaque,
            gltf::material::AlphaMode::Mask => crate::AlphaMode::Mask,
            gltf::material::AlphaMode::Blend => crate::AlphaMode::Blend,
        })
        .double_sided(gltf_material.double_sided());

    if let Some(scale) = gltf_material.normal_texture().map(|info| info.scale()) {
        material = material.normal_scale(scale);
    }
    if let Some(strength) = gltf_material
        .occlusion_texture()
        .map(|info| info.strength())
    {
        material = material.occlusion_strength(strength);
    }

    let read = |texture: gltf::Texture<'_>| convert_image(&texture, images);
    if let Some(info) = pbr.base_color_texture() {
        material.set_map(TextureSlot::BaseColor, read(info.texture())?);
        apply_uv(&material, TextureSlot::BaseColor, read_info_uv(&info))?;
    }
    if let Some(info) = pbr.metallic_roughness_texture() {
        material.set_map(TextureSlot::MetallicRoughness, read(info.texture())?);
        apply_uv(
            &material,
            TextureSlot::MetallicRoughness,
            read_info_uv(&info),
        )?;
    }
    if let Some(info) = gltf_material.normal_texture() {
        material.set_map(TextureSlot::Normal, read(info.texture())?);
        let extension = info.extension_value(TEXTURE_TRANSFORM);
        let uv = read_extension_uv(info.tex_coord(), extension)?;
        apply_uv(&material, TextureSlot::Normal, uv)?;
    }
    if let Some(info) = gltf_material.occlusion_texture() {
        material.set_map(TextureSlot::Occlusion, read(info.texture())?);
        let extension = info.extension_value(TEXTURE_TRANSFORM);
        let uv = read_extension_uv(info.tex_coord(), extension)?;
        apply_uv(&material, TextureSlot::Occlusion, uv)?;
    }
    if let Some(info) = gltf_material.emissive_texture() {
        material.set_map(TextureSlot::Emissive, read(info.texture())?);
        apply_uv(&material, TextureSlot::Emissive, read_info_uv(&info))?;
    }

    Ok(material)
}

/// The extension that gives a glTF texture reference its own UV transform.
const TEXTURE_TRANSFORM: &str = "KHR_texture_transform";

/// The UVs a glTF texture reference reads: the `TEXCOORD_<n>` set it
/// names, and the transform from its `KHR_texture_transform`.
struct TextureUv {
    uv_set: u32,
    transform: UvTransform,
}

/// Read the UVs of a texture reference the gltf crate types, whose
/// `KHR_texture_transform` it parses. The extension's `texCoord` wins
/// over the reference's own.
fn read_info_uv(info: &gltf::texture::Info<'_>) -> TextureUv {
    match info.texture_transform() {
        Some(transform) => TextureUv {
            uv_set: transform.tex_coord().unwrap_or(info.tex_coord()),
            transform: UvTransform {
                offset: transform.offset(),
                scale: transform.scale(),
                rotation: transform.rotation(),
            },
        },
        None => TextureUv {
            uv_set: info.tex_coord(),
            transform: UvTransform::default(),
        },
    }
}

/// Read the UVs of a normal or occlusion texture reference. The gltf crate
/// keeps their `KHR_texture_transform` as raw JSON, parsed here as it parses
/// the typed slots, so a `null` extension counts as no extension.
fn read_extension_uv(
    uv_set: u32,
    extension: Option<&gltf::json::Value>,
) -> Result<TextureUv, SceneLoadError> {
    use gltf::json::extensions::texture::TextureTransform;
    let parsed: Option<TextureTransform> = match extension {
        Some(value) => gltf::json::deserialize::from_value(value.clone())
            .map_err(|error| SceneLoadError::Invalid(format!("{TEXTURE_TRANSFORM}: {error}")))?,
        None => None,
    };
    let Some(transform) = parsed else {
        return Ok(TextureUv {
            uv_set,
            transform: UvTransform::default(),
        });
    };
    Ok(TextureUv {
        uv_set: transform.tex_coord.unwrap_or(uv_set),
        transform: UvTransform {
            offset: transform.offset.0,
            scale: transform.scale.0,
            rotation: transform.rotation.0,
        },
    })
}

/// Point one material slot at the UV set and transform its glTF texture reads.
/// The PBR shader samples two UV sets, so `TEXCOORD_2` and above are an error.
fn apply_uv(material: &Material, slot: TextureSlot, uv: TextureUv) -> Result<(), SceneLoadError> {
    let Ok(set) = UvSet::try_from(uv.uv_set) else {
        return Err(SceneLoadError::Invalid(format!(
            "glTF {} reads TEXCOORD_{}, and the PBR material samples TEXCOORD_0 and TEXCOORD_1 only",
            name_slot(slot),
            uv.uv_set
        )));
    };
    material.set_slot_uv(slot, set, uv.transform);
    Ok(())
}

/// The glTF property that holds a slot's texture reference.
fn name_slot(slot: TextureSlot) -> &'static str {
    match slot {
        TextureSlot::BaseColor => "baseColorTexture",
        TextureSlot::MetallicRoughness => "metallicRoughnessTexture",
        TextureSlot::Normal => "normalTexture",
        TextureSlot::Occlusion => "occlusionTexture",
        TextureSlot::Emissive => "emissiveTexture",
    }
}

/// Convert a glTF texture reference into the source a Material map keeps, with the
/// texture's own sampler. Every reference to one image shares that image's source.
fn convert_image(
    texture: &gltf::Texture<'_>,
    images: &[TextureSource],
) -> Result<TextureSource, SceneLoadError> {
    let image = images
        .get(texture.source().index())
        .ok_or_else(|| SceneLoadError::Invalid("glTF texture references missing image".into()))?;
    let mut source = image.clone();
    source.options.sampler = map_sampler_options(&texture.sampler());
    Ok(source)
}
