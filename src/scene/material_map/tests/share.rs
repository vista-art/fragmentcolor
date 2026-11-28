//! `MaterialMap::texture` with another map: both maps sample one texture.

use std::sync::Arc;

use super::{GREEN, RED};
use crate::core::shader::uniforms::texture_maps::{MapJob, MapTexture};
use crate::texture::TextureError;
use crate::{MaterialError, MaterialMap, PbrMaterial, RenderContext, Renderer, Shader, Size};
use crate::{TextureData, TextureFormat, TextureId, TextureInput, TextureOptions};
use crate::{TextureTarget, ToonMaterial, UniformData, UnlitMaterial};

/// A 2×2 image of `color`, as raw pixels with their size.
fn paint(color: [u8; 4]) -> (Vec<u8>, [u32; 2]) {
    (color.repeat(4), [2, 2])
}

/// The texture id the slot of `map` names.
fn read_id(map: &MaterialMap) -> TextureId {
    match map.shader.object.get_uniform_data(map.slot.pick_key()) {
        Ok(UniformData::Texture(meta)) => meta.id,
        other => panic!("{map:?} names no texture: {other:?}"),
    }
}

/// The map texture the slot of `map` samples.
fn find_texture(map: &MaterialMap) -> Arc<MapTexture> {
    let found = map.shader.object.find_map(map.slot.pick_key());
    found.unwrap_or_else(|| panic!("{map:?} is no map"))
}

/// The uploads a load into `context` would run for `shaders`.
fn list_jobs(context: &RenderContext, shaders: &[&Shader]) -> Vec<MapJob> {
    let mut jobs = Vec::new();
    for shader in shaders {
        shader.object.find_jobs(context, &mut jobs);
    }
    jobs
}

/// A Renderer and an 8×8 target it made, whose context holds the Renderer's copies.
async fn open_target() -> (Renderer, TextureTarget) {
    let renderer = Renderer::new();
    let target = renderer.create_texture_target([8, 8]).await;
    (renderer, target.expect("a target"))
}

// Story: a toon map that takes a PBR map's texture names the same id and holds the
// same texture, so the source stays in memory once.
#[test]
fn two_maps_share_one_id() {
    let pbr = PbrMaterial::new();
    pbr.base_color_map()
        .texture(paint(RED))
        .expect("base color");
    let toon = ToonMaterial::new();

    let map = toon.base_color_map().texture(&pbr.base_color_map());

    let map = map.expect("the toon map takes the PBR map");
    assert_eq!(read_id(&map), read_id(&pbr.base_color_map()));
    let shared = find_texture(&pbr.base_color_map());
    assert!(Arc::ptr_eq(&shared, &find_texture(&map)), "two textures");
}

// Story: three maps that share one texture cost one upload. A Renderer that loaded the
// texture for the first map holds it for the others, which need no load of their own.
#[test]
fn a_renderer_uploads_a_shared_texture_once() {
    pollster::block_on(async {
        let (renderer, target) = open_target().await;
        let context = target.context.clone();
        let pbr = PbrMaterial::new();
        pbr.base_color_map()
            .texture(paint(RED))
            .expect("base color");
        let toon = ToonMaterial::new();
        let unlit = UnlitMaterial::new();

        let shared = toon.base_color_map().texture(&pbr.base_color_map());
        let shared = shared.expect("the toon map takes the PBR map");
        let chained = unlit.base_color_map().texture(&shared);
        chained.expect("the unlit map takes the toon map");

        let all = [pbr.shader(), toon.shader(), unlit.shader()];
        assert_eq!(list_jobs(&context, &all).len(), 1, "an upload per map");
        renderer.load(pbr.shader()).await.expect("the load");
        let others = list_jobs(&context, &[toon.shader(), unlit.shader()]);
        assert!(others.is_empty(), "the other maps upload again");
        let id = read_id(&unlit.base_color_map());
        assert!(context.has_texture(&id), "no copy under the shared id");
    });
}

// Story: a map that takes a map set from a `Texture` binds that texture as it is,
// and keeps it after the texture's handle and the first material drop.
#[test]
fn a_map_takes_the_texture_another_map_binds() {
    pollster::block_on(async {
        let (renderer, target) = open_target().await;
        let texture = renderer.create_texture(paint(RED)).await.expect("texture");
        let id = *texture.id();
        let pbr = PbrMaterial::new();
        pbr.base_color_map().texture(&texture).expect("base color");
        let toon = ToonMaterial::new();

        let map = toon.base_color_map().texture(&pbr.base_color_map());

        assert_eq!(read_id(&map.expect("the toon map")), id);
        let object = &toon.shader().object;
        assert!(!object.has_map("base_color_map"), "the texture made a map");
        drop((texture, pbr));
        assert!(target.context.has_texture(&id), "the texture left");
    });
}

// Story: a map with no texture of its own, which draws its 1×1 default, leaves nothing
// to share. The call is refused, and the taking map keeps the texture it had.
#[test]
fn a_map_with_no_texture_has_none_to_share() {
    let pbr = PbrMaterial::new();
    let toon = ToonMaterial::new();
    toon.base_color_map()
        .texture(paint(GREEN))
        .expect("base color");
    let id = read_id(&toon.base_color_map());

    let refused = toon.base_color_map().texture(&pbr.normal_map());

    let error = refused.expect_err("an empty map");
    assert!(
        matches!(error, MaterialError::Texture(TextureError::InvalidInput(_))),
        "{error:?}"
    );
    assert_eq!(
        error.to_string(),
        "Invalid texture input: ToonMaterial::base_color_map().texture: PbrMaterial::normal_map() has no texture to share"
    );
    assert_eq!(read_id(&toon.base_color_map()), id, "the map changed");
}

// Story: a base color map reads its source as sRGB and a normal map as linear, which
// one upload can't serve. A normal map given a base color map keeps the same source
// and uploads a copy of its own in its own color space. Two linear maps share one.
#[test]
fn a_map_in_another_color_space_uploads_a_copy_of_its_own() {
    let pbr = PbrMaterial::new();
    pbr.base_color_map()
        .texture(paint(RED))
        .expect("base color");

    let normal = pbr.normal_map().texture(&pbr.base_color_map());
    let normal = normal.expect("the normal map takes the base color map");
    let occlusion = pbr.occlusion_map().texture(&normal);
    let occlusion = occlusion.expect("the occlusion map takes the normal map");

    let color = find_texture(&pbr.base_color_map());
    let linear = find_texture(&normal);
    assert_ne!(color.id, linear.id, "one upload in two color spaces");
    let (color, linear) = (color.read_source(), linear.read_source());
    assert!(Arc::ptr_eq(&color.data, &linear.data), "a second source");
    assert_eq!(color.options.format, TextureFormat::Rgba8UnormSrgb);
    assert_eq!(linear.options.format, TextureFormat::Rgba8Unorm);
    assert_eq!(read_id(&occlusion), read_id(&normal), "two linear uploads");
}

// Story: a format the caller picked is no hint, so a map in another color space
// shares the texture in that format, as a `Texture` binds in its own.
#[test]
fn a_format_the_caller_picked_stays_shared() {
    let pbr = PbrMaterial::new();
    let input = TextureInput {
        data: TextureData::Bytes(RED.repeat(4)),
        options: TextureOptions {
            size: Some(Size::from([2u32, 2])),
            format: TextureFormat::Rgba8Unorm,
            ..Default::default()
        },
    };
    pbr.base_color_map().texture(input).expect("base color");
    let toon = ToonMaterial::new();

    let map = toon.base_color_map().texture(&pbr.base_color_map());

    let id = read_id(&pbr.base_color_map());
    assert_eq!(read_id(&map.expect("the toon map")), id);
}

// Story: a map takes its own texture again and keeps it, with the same id and source.
#[test]
fn a_map_takes_its_own_texture() {
    let pbr = PbrMaterial::new();
    let map = pbr
        .base_color_map()
        .texture(paint(RED))
        .expect("base color");
    let texture = find_texture(&map);

    let again = map.texture(&map).expect("its own texture");

    assert_eq!(read_id(&again), texture.id);
    assert!(
        Arc::ptr_eq(&texture, &find_texture(&again)),
        "a new texture"
    );
}

// Story: a map that takes a new source while another map shares its texture gets a new
// texture of its own. The other map keeps the old one, and the new one draws the old copy
// in each Renderer until the next load uploads the new source, as a map set again does.
#[test]
fn a_new_source_leaves_the_shared_texture_to_the_other_map() {
    pollster::block_on(async {
        let (renderer, target) = open_target().await;
        let context = target.context.clone();
        let pbr = PbrMaterial::new();
        pbr.base_color_map()
            .texture(paint(RED))
            .expect("base color");
        let toon = ToonMaterial::new();
        toon.base_color_map()
            .texture(&pbr.base_color_map())
            .expect("the toon map");
        renderer.load(pbr.shader()).await.expect("the load");
        let old = read_id(&toon.base_color_map());

        pbr.base_color_map()
            .texture(paint(GREEN))
            .expect("a new source");

        let new = read_id(&pbr.base_color_map());
        assert_ne!(new, old, "the shared texture took the new source");
        assert_eq!(read_id(&toon.base_color_map()), old, "the toon map moved");
        let copy = context.get_texture(&old).expect("the old copy");
        let stand_in = context.get_texture(&new).expect("a copy to draw");
        assert!(
            Arc::ptr_eq(&copy, &stand_in),
            "the stand-in is another copy"
        );
        assert!(list_jobs(&context, &[toon.shader()]).is_empty());
        let jobs = list_jobs(&context, &[pbr.shader()]);
        assert_eq!(jobs.iter().map(|job| job.id).collect::<Vec<_>>(), [new]);
    });
}

// Story: once the other maps let go of a shared texture, one by a new source and one
// by dropping, the map left with it keeps its id when it takes a new source.
#[test]
fn a_texture_no_other_map_holds_keeps_its_id() {
    let pbr = PbrMaterial::new();
    pbr.base_color_map()
        .texture(paint(RED))
        .expect("base color");
    let id = read_id(&pbr.base_color_map());
    let toon = ToonMaterial::new();
    toon.base_color_map()
        .texture(&pbr.base_color_map())
        .expect("the toon map");
    let unlit = UnlitMaterial::new();
    unlit
        .base_color_map()
        .texture(&pbr.base_color_map())
        .expect("the unlit map");

    toon.base_color_map()
        .texture(paint(GREEN))
        .expect("a new toon source");
    drop(unlit);
    pbr.base_color_map()
        .texture(paint(GREEN))
        .expect("a new source");

    assert_eq!(read_id(&pbr.base_color_map()), id, "the map took a new id");
}

// Story: a shared copy stays in a Renderer while any map that samples it is alive,
// and goes when the last one drops.
#[test]
fn the_last_map_frees_a_shared_copy() {
    pollster::block_on(async {
        let (renderer, target) = open_target().await;
        let pbr = PbrMaterial::new();
        pbr.base_color_map()
            .texture(paint(RED))
            .expect("base color");
        let toon = ToonMaterial::new();
        toon.base_color_map()
            .texture(&pbr.base_color_map())
            .expect("the toon map");
        renderer.load(toon.shader()).await.expect("the load");
        let id = read_id(&toon.base_color_map());

        drop(pbr);
        assert!(
            target.context.has_texture(&id),
            "the copy left with the PBR map"
        );
        drop(toon);
        assert!(
            !target.context.has_texture(&id),
            "the copy outlived its maps"
        );
    });
}
