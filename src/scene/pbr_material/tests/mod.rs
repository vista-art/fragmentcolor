use std::f32::consts::PI;

use naga::back::wgsl::{WriterFlags, write_string};
use naga::front::wgsl::parse_str;
use naga::valid::{Capabilities, ValidationFlags, Validator};

use super::*;
use crate::testing::race_hooks;
use crate::texture::loading::ktx2_loader::test_helpers::Ktx2Test;
use crate::texture::{SourceData, TextureData, TextureError};
use crate::{Light, Mesh, Mipmap, Model, Size, Vertex};
use crate::{Pass, Renderer, SamplerOptions, Target, TextureFormat, TextureId, TextureTarget};
use crate::{ShaderError, TextureInput, UniformData};

#[test]
fn pbr_seeds_default_uniforms() {
    let mat = PbrMaterial::new();
    let base: [f32; 4] = mat.shader().get("material.base_color").expect("base_color");
    assert_eq!(base, [1.0, 1.0, 1.0, 1.0]);
    let metallic: f32 = mat.shader().get("material.metallic").expect("metallic");
    assert_eq!(metallic, 0.0);
    let roughness: f32 = mat.shader().get("material.roughness").expect("roughness");
    assert_eq!(roughness, 1.0);
}

#[test]
fn builder_setters_update_uniforms() {
    let mat = PbrMaterial::new()
        .base_color([0.4, 0.7, 0.2, 0.8])
        .expect("base color")
        .metallic(0.6)
        .expect("metallic")
        .roughness(0.25)
        .expect("roughness")
        .emissive([0.0, 0.1, 0.0])
        .expect("emissive");

    let base: [f32; 4] = mat.shader().get("material.base_color").unwrap();
    assert_eq!(base, [0.4, 0.7, 0.2, 0.8]);
    let metallic: f32 = mat.shader().get("material.metallic").unwrap();
    assert!((metallic - 0.6).abs() < 1.0e-6);
    let roughness: f32 = mat.shader().get("material.roughness").unwrap();
    assert!((roughness - 0.25).abs() < 1.0e-6);
    let emissive: [f32; 3] = mat.shader().get("material.emissive").unwrap();
    assert_eq!(emissive, [0.0, 0.1, 0.0]);
}

#[test]
fn clone_shares_shader_state() {
    let original = PbrMaterial::new()
        .base_color([1.0, 0.0, 0.0, 1.0])
        .expect("base color");
    let handle_b = original.clone();
    let _ = handle_b
        .shader()
        .set("material.base_color", [0.0_f32, 1.0, 0.0, 1.0]);

    let original_color: [f32; 4] = original.shader().get("material.base_color").unwrap();
    let b_color: [f32; 4] = handle_b.shader().get("material.base_color").unwrap();
    assert_eq!(original_color, [0.0, 1.0, 0.0, 1.0]);
    assert_eq!(b_color, [0.0, 1.0, 0.0, 1.0]);
}

#[test]
fn alpha_mode_setter_threads_to_shader_back_reference() {
    let mat = PbrMaterial::new().alpha_mode(AlphaMode::Mask);
    // ShaderObject picks up the new mode for the pipeline cache key;
    // the alpha_mode_flag uniform picks it up too so fs_main's Mask
    // discard branch fires.
    assert_eq!(*mat.shader.object.alpha_mode.read(), AlphaMode::Mask);
    let flag: u32 = mat.shader().get("material.alpha_mode_flag").unwrap();
    assert_eq!(flag, AlphaMode::Mask.flag());
}

#[test]
fn double_sided_setter_threads_to_shader_back_reference() {
    let mat = PbrMaterial::new().double_sided(true);
    assert!(*mat.shader.object.double_sided.read());
}

// Story: a number setter refuses NaN and an infinity with an error that names the call
// and the item, and the material keeps the value it had.
#[test]
fn number_setters_refuse_numbers_that_are_not_finite() {
    let material = PbrMaterial::new()
        .metallic(0.25)
        .expect("metallic")
        .emissive([0.5, 0.5, 0.5])
        .expect("emissive");

    let scalar = material.metallic(f32::NAN).expect_err("NaN");
    let vector = material.emissive([0.0, f32::INFINITY, 0.0]);
    let vector = vector.expect_err("infinity");

    assert_eq!(
        scalar.to_string(),
        "PbrMaterial::metallic: value must be finite, got NaN"
    );
    assert_eq!(
        vector.to_string(),
        "PbrMaterial::emissive: item 1 must be finite, got inf"
    );
    let metallic: f32 = material.shader().get("material.metallic").expect("read");
    let emissive: [f32; 3] = material.shader().get("material.emissive").expect("read");
    assert_eq!((metallic, emissive), (0.25, [0.5, 0.5, 0.5]));
    let cases = [
        PbrMaterial::new().base_color([0.0, 0.0, f32::NAN, 1.0]),
        PbrMaterial::new().roughness(f32::NEG_INFINITY),
        PbrMaterial::new().normal_scale(f32::NAN),
        PbrMaterial::new().occlusion_strength(f32::INFINITY),
        PbrMaterial::new().alpha_cutoff(f32::NAN),
    ];
    for case in cases {
        let error = case.expect_err("not finite");
        assert!(
            matches!(error, MaterialError::NotFinite { .. }),
            "{error:?}"
        );
    }
}

// Story: a material whose shader holds roughness in an f16 refuses a roughness past
// its range with the shader's error, and a factor the shader doesn't declare is skipped.
#[test]
fn setters_refuse_what_the_shader_fields_cannot_hold() {
    let shader = Shader::new(
        "enable f16;
struct Factors { roughness: f16, metallic: f32 }
@group(0) @binding(0) var<uniform> material: Factors;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return vec4<f32>(f32(i), 0.0, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> {
  return vec4<f32>(f32(material.roughness), material.metallic, 0.0, 1.0);
}",
    )
    .expect("shader");
    let material = PbrMaterial::wrap(shader).roughness(0.5).expect("roughness");

    let refused = material.roughness(70000.0).expect_err("past the f16 range");
    let skipped = material.emissive([1.0, 0.0, 0.0]);

    assert!(matches!(refused, MaterialError::Shader(_)), "{refused:?}");
    assert!(refused.to_string().contains("expected f16, got 70000"));
    let roughness: f32 = material.shader().get("material.roughness").expect("read");
    assert_eq!(roughness, 0.5);
    assert!(skipped.is_ok(), "{skipped:?}");
}

fn pbr_triangle_mesh() -> Mesh {
    let mesh = Mesh::new();
    for (p, uv) in [
        ([0.0, 0.5, 0.0], [0.5, 1.0]),
        ([-0.5, -0.5, 0.0], [0.0, 0.0]),
        ([0.5, -0.5, 0.0], [1.0, 0.0]),
    ] {
        mesh.add_vertex(Vertex::pbr(p).set(Vertex::UV0, uv));
    }
    mesh
}

#[test]
fn pbr_with_no_user_textures_renders_with_defaults() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");
        let mat = PbrMaterial::new()
            .base_color([0.6, 0.2, 0.8, 1.0])
            .expect("base color");
        let model = Model::new(pbr_triangle_mesh(), mat);
        let pass = Pass::new("defaults-only");
        pass.add(&model).expect("add_model");
        renderer
            .render(&pass, &target)
            .expect("render with all default textures");
        let img = target.get_image().await.expect("read back");
        assert_eq!(img.len(), 8 * 8 * 4);
    });
}

#[test]
fn pbr_samples_base_color_texture() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16u32, 16u32])
            .await
            .expect("texture target");
        #[rustfmt::skip]
            let pixels: [u8; 16] = [
                255,   0,   0, 255,    0, 255,   0, 255,
                  0,   0, 255, 255,  255, 255, 255, 255,
            ];
        let tex = renderer
            .create_texture((&pixels[..], Size::from((2u32, 2u32))))
            .await
            .expect("base_color texture");
        tex.set_sampler_options(SamplerOptions {
            smooth: false,
            ..Default::default()
        });
        let mat = PbrMaterial::new()
            .base_color([1.0, 1.0, 1.0, 1.0])
            .expect("base color");
        mat.base_color_map().texture(&tex).expect("base color map");

        mat.shader()
            .set("camera.position", [0.0_f32, 0.0, 1.0])
            .ok();
        let light = Light::directional([0.0, 0.0, -1.0], [1.0, 1.0, 1.0]).expect("light");

        let mesh = Mesh::new();
        for (pos, uv) in [
            ([-1.0_f32, 1.0, 0.0], [0.0_f32, 1.0]),
            ([-1.0, -1.0, 0.0], [0.0, 0.0]),
            ([1.0, -1.0, 0.0], [1.0, 0.0]),
        ] {
            mesh.add_vertex(Vertex::pbr(pos).set(Vertex::UV0, uv));
        }
        let model = Model::new(mesh, mat);
        let pass = Pass::new("textured-triangle");
        pass.add(&model).expect("add_model");
        pass.add(&light).expect("light");
        renderer
            .render(&pass, &target)
            .expect("render textured triangle");
        let img = target.get_image().await.expect("read back");
        assert_eq!(img.len(), 16 * 16 * 4);

        let mut has_red = false;
        let mut has_blue = false;
        for px in img.as_chunks::<4>().0 {
            if px[0] > 80 && px[1] < 60 && px[2] < 60 {
                has_red = true;
            }
            if px[2] > 80 && px[0] < 60 && px[1] < 60 {
                has_blue = true;
            }
        }
        assert!(
            has_red && has_blue,
            "expected sampled base_color texture to contribute red and blue pixels; got neither (has_red={has_red}, has_blue={has_blue})"
        );
    });
}

/// The id `material`'s map for `key` holds, the one each Renderer's copy goes under.
fn read_map(material: &PbrMaterial, key: &str) -> TextureId {
    let maps = material.shader.object.maps.read();
    let map = maps.iter().find(|map| map.key == key);
    map.map(|map| map.id).expect("a map")
}

/// Whether the Renderer that made `target` holds a copy of `material`'s map for `key`.
fn holds_copy(target: &TextureTarget, material: &PbrMaterial, key: &str) -> bool {
    let id = read_map(material, key);
    target.texture().context.has_texture(&id)
}

/// A material whose fullscreen shader draws texel (0, 0) of `base_color_map`.
fn build_mapped() -> PbrMaterial {
    let shader = Shader::new(
            r#"
@group(0) @binding(0) var base_color_map: texture_2d<f32>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  var p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return textureLoad(base_color_map, vec2<i32>(0), 0); }
"#,
        )
        .expect("shader");
    PbrMaterial::wrap(shader)
}

/// A 2x2 image of `color`, as raw pixels.
fn paint(color: [u8; 4]) -> (Vec<u8>, [u32; 2]) {
    (color.repeat(4), [2, 2])
}

/// The first pixel of `target`.
async fn read_pixel(target: &TextureTarget) -> [u8; 4] {
    let image = target.get_image().await.expect("read back");
    [image[0], image[1], image[2], image[3]]
}

// Story: raw bytes make a map, which the slot names before any Renderer holds
// a copy, and the first render uploads a copy into its Renderer under that id.
#[test]
fn base_color_texture_with_raw_bytes_uploads_at_render() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");

        #[rustfmt::skip]
            let pixels: Vec<u8> = vec![
                255, 0, 0, 255,    0, 255, 0, 255,
                  0, 0, 255, 255,  255, 255, 255, 255,
            ];
        let mat = PbrMaterial::new();
        mat.base_color_map()
            .texture((pixels, [2u32, 2u32]))
            .expect("base color map");

        let id = read_map(&mat, "base_color_map");
        assert!(
            !holds_copy(&target, &mat, "base_color_map"),
            "a copy before render"
        );

        let model = Model::new(pbr_triangle_mesh(), mat.clone());
        let pass = Pass::new("lazy-upload");
        pass.add(&model).expect("add_model");
        renderer
            .render(&pass, &target)
            .expect("render drains pending");

        assert!(
            holds_copy(&target, &mat, "base_color_map"),
            "no copy after render"
        );
        let slot = mat.shader.object.get_uniform_data("base_color_map");
        assert!(
            matches!(&slot, Ok(UniformData::Texture(meta)) if meta.id == id),
            "{slot:?}"
        );
        let img = target.get_image().await.expect("read back");
        assert_eq!(img.len(), 8 * 8 * 4);
    });
}

// Story: a map that fails to upload keeps no copy and fails every render the same way,
// while the other maps upload on the first try.
#[test]
fn failed_maps_keep_no_copy_and_the_rest_upload() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");
        let normal: Vec<u8> = [128u8, 128, 255, 255].repeat(4);
        let mat = PbrMaterial::new();
        mat.base_color_map()
            .texture("tests/assets/missing.png")
            .expect("base color map");
        mat.normal_map()
            .texture((normal, [2u32, 2u32]))
            .expect("normal map");
        let model = Model::new(pbr_triangle_mesh(), mat.clone());
        let pass = Pass::new("partial-upload");
        pass.add(&model).expect("add_model");

        let first = renderer.render(&pass, &target);
        let second = renderer.render(&pass, &target);

        let first = first.expect_err("the missing map fails the first render");
        let second = second.expect_err("and every render after it");
        assert_eq!(first.to_string(), second.to_string());
        assert!(
            holds_copy(&target, &mat, "normal_map"),
            "the good map uploads on the first render"
        );
        assert!(
            !holds_copy(&target, &mat, "base_color_map"),
            "the failing map keeps no copy"
        );
    });
}

// Story: a slot the shader doesn't declare is skipped with a warning, for a source and
// a `Texture` alike, and makes no map, so the render draws the maps the shader declares.
#[test]
fn an_undeclared_map_warns_at_the_setter() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");
        let texture = renderer
            .create_texture(paint([0, 255, 0, 255]))
            .await
            .expect("a texture");
        let pixel: Vec<u8> = [255u8, 0, 0, 255].repeat(4);
        let mat = build_mapped();

        let mapped = mat.normal_map().texture((pixel.clone(), [2u32, 2u32]));
        let bound = mat.occlusion_map().texture(&texture);
        mat.base_color_map()
            .texture((pixel, [2u32, 2u32]))
            .expect("base color map");

        assert!(mapped.is_ok(), "{mapped:?}");
        assert!(bound.is_ok(), "{bound:?}");
        assert!(
            !mat.shader.object.has_map("normal_map"),
            "an undeclared map"
        );
        renderer
            .render(mat.shader(), &target)
            .expect("the render draws");
        assert!(holds_copy(&target, &mat, "base_color_map"));
    });
}

// Story: the latest setter call for a map wins, so a fixed input replaces a failing one
// under the same id, and the key keeps one map.
#[test]
fn latest_map_setter_wins() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");
        let pixels: Vec<u8> = [255u8, 0, 0, 255].repeat(4);
        let mat = PbrMaterial::new();
        mat.base_color_map()
            .texture("tests/assets/missing.png")
            .expect("base color map");
        let id = read_map(&mat, "base_color_map");
        mat.base_color_map()
            .texture((pixels, [2u32, 2u32]))
            .expect("base color map");
        {
            let maps = mat.shader.object.maps.read();
            assert_eq!(maps.len(), 1, "one map for the key");
            assert_eq!(maps[0].id, id, "the map kept its id");
            let latest = matches!(&*maps[0].source.data, SourceData::Bytes(_));
            assert!(latest, "{:?}", maps[0].source);
        }
        let model = Model::new(pbr_triangle_mesh(), mat.clone());
        let pass = Pass::new("replaced-upload");
        pass.add(&model).expect("add_model");

        renderer
            .render(&pass, &target)
            .expect("the replaced map uploads");

        assert!(holds_copy(&target, &mat, "base_color_map"));
    });
}

// Story: a texture set while a map's upload fails wins. The load drops the failure
// of the map that ended and returns Ok, and the render draws the texture.
#[test]
fn a_texture_set_during_a_failed_upload_wins() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("a target");
        let texture = renderer
            .create_texture(paint([0, 255, 0, 255]))
            .await
            .expect("a texture");
        let mat = build_mapped();
        mat.base_color_map()
            .texture("tests/assets/missing.png")
            .expect("base color map");
        let late = mat.clone();
        race_hooks::once("map uploaded", move || {
            late.base_color_map()
                .texture(&texture)
                .expect("base color map");
        });

        let loaded = renderer.load(mat.shader()).await;

        assert!(
            loaded.is_ok(),
            "the ended map's failure came back: {loaded:?}"
        );
        assert!(
            !mat.shader.object.has_map("base_color_map"),
            "the map stayed"
        );
        renderer.render(mat.shader(), &target).expect("draws");
        assert_eq!(read_pixel(&target).await, [0, 255, 0, 255]);
    });
}

// Story: a set that lands while a load uploads the older source wins. The older upload
// stands in as that Renderer's copy, and each Renderer then draws the latest.
#[test]
fn a_set_during_an_upload_ends_on_the_last_value() {
    pollster::block_on(async move {
        let first = Renderer::new();
        let first_target = first.create_texture_target([8, 8]).await.expect("a target");
        let second = Renderer::new();
        let second_target = second
            .create_texture_target([8, 8])
            .await
            .expect("a target");
        let mat = build_mapped();
        mat.base_color_map()
            .texture(paint([255, 0, 0, 255]))
            .expect("base color map");
        let late = mat.clone();
        race_hooks::once("map uploaded", move || {
            late.base_color_map()
                .texture(paint([0, 0, 255, 255]))
                .expect("base color map");
        });

        first.load(mat.shader()).await.expect("the load");
        assert!(
            holds_copy(&first_target, &mat, "base_color_map"),
            "the older upload left the first Renderer without a copy"
        );

        first.render(mat.shader(), &first_target).expect("draws");
        second.render(mat.shader(), &second_target).expect("draws");
        assert_eq!(read_pixel(&first_target).await, [0, 0, 255, 255]);
        assert_eq!(read_pixel(&second_target).await, [0, 0, 255, 255]);
    });
}

// Story: a `Texture` keeps the options it was created with, so the setter refuses
// options beside one with an error that names the call, and the slot keeps its texture.
#[test]
fn a_texture_with_options_leaves_the_slot() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let first = renderer
            .create_texture(paint([255, 0, 0, 255]))
            .await
            .expect("a texture");
        let second = renderer
            .create_texture(paint([0, 0, 255, 255]))
            .await
            .expect("a texture");
        let mat = PbrMaterial::new();
        mat.base_color_map()
            .texture(&first)
            .expect("base color map");

        let refused = mat
            .base_color_map()
            .texture((&second, TextureFormat::Rgba8Unorm));

        let error = refused.expect_err("options beside a Texture");
        assert!(
            matches!(error, MaterialError::Texture(TextureError::InvalidInput(_))),
            "{error:?}"
        );
        assert_eq!(
            error.to_string(),
            "Invalid texture input: PbrMaterial::base_color_map().texture: a Texture keeps the options it was created with, so it takes none"
        );
        let slot = mat.shader.object.get_uniform_data("base_color_map");
        assert!(
            matches!(&slot, Ok(UniformData::Texture(meta)) if meta.id == *first.id()),
            "{slot:?}"
        );
    });
}

// Story: an empty byte list, path or URL holds no image, so the setter refuses it
// at the call, where it failed the next load, and the slot keeps its map.
#[test]
fn an_empty_source_leaves_the_slot() {
    let mat = PbrMaterial::new();
    mat.base_color_map()
        .texture(paint([255, 0, 0, 255]))
        .expect("base color map");
    let id = read_map(&mat, "base_color_map");
    let empties = [
        (TextureData::Bytes(Vec::new()), "byte list"),
        (TextureData::Ktx2Bytes(Vec::new()), "byte list"),
        (TextureData::Path("".into()), "path"),
        (TextureData::Url(String::new()), "URL"),
    ];

    for (input, form) in empties {
        let error = mat
            .base_color_map()
            .texture(input)
            .expect_err("an empty source");

        let expected = format!(
            "Invalid texture input: PbrMaterial::base_color_map().texture: a texture source needs data, and it got an empty {form}"
        );
        assert_eq!(error.to_string(), expected);
        assert_eq!(read_map(&mat, "base_color_map"), id, "{form}");
    }
}

// Story: an input that holds no data makes no map, so the setter returns an error
// that names the call, and the slot keeps the map it held.
#[test]
fn an_input_with_no_data_leaves_the_slot() {
    let mat = PbrMaterial::new();
    mat.base_color_map()
        .texture(paint([255, 0, 0, 255]))
        .expect("base color map");
    let id = read_map(&mat, "base_color_map");

    let refused = mat.base_color_map().texture(TextureData::Empty);

    let error = refused.expect_err("an input with no data");
    assert!(
        matches!(error, MaterialError::Texture(TextureError::InvalidInput(_))),
        "{error:?}"
    );
    assert_eq!(
        error.to_string(),
        "Invalid texture input: PbrMaterial::base_color_map().texture: a texture source needs data: bytes, a path, a URL or a Mipmap"
    );
    assert_eq!(read_map(&mat, "base_color_map"), id);
    let maps = mat.shader.object.maps.read();
    let kept = matches!(&*maps[0].source.data, SourceData::Bytes(_));
    assert!(kept, "{:?}", maps[0].source);
}

/// A material whose shader declares `base_color_map` as a color.
fn build_numbered() -> PbrMaterial {
    let shader = Shader::new(
        r#"
@group(0) @binding(0) var<uniform> base_color_map: vec4<f32>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return vec4<f32>(f32(i), 0.0, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return base_color_map; }
"#,
    )
    .expect("shader");
    PbrMaterial::wrap(shader)
}

// Story: a custom shader that declares a slot as a number refuses a `Texture` and a source
// there with the shader's error, and the slot keeps its value and makes no map.
#[test]
fn a_slot_declared_as_a_number_refuses_textures() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let texture = renderer
            .create_texture(paint([255, 0, 0, 255]))
            .await
            .expect("a texture");
        let mat = build_numbered();
        let color = [0.25_f32, 0.5, 0.75, 1.0];
        mat.shader().set("base_color_map", color).expect("a color");

        let bound = mat.base_color_map().texture(&texture);
        let mapped = mat.base_color_map().texture(paint([0, 255, 0, 255]));

        for refused in [bound, mapped] {
            let error = refused.expect_err("a texture on a number");
            assert!(
                matches!(error, MaterialError::Shader(ShaderError::TypeMismatch(_))),
                "{error:?}"
            );
        }
        let kept: [f32; 4] = mat.shader().get("base_color_map").expect("read");
        assert_eq!(kept, color);
        assert!(!mat.shader.object.has_map("base_color_map"), "a map");
    });
}

/// A 4x4 KTX2 container of zeros in `format`, at `bytes` per texel.
fn build_container(format: ktx2::Format, bytes: usize) -> TextureInput {
    let levels = vec![vec![0u8; 4 * 4 * bytes]];
    let container = Ktx2Test {
        format,
        width: 4,
        height: 4,
        levels,
    };
    TextureInput::from(TextureData::Ktx2Bytes(container.build()))
}

// Story: a map takes its slot's format as a hint. A Mipmap or KTX2 container whose bytes
// read in it uploads in its sRGB twin, and a float one, whose bytes don't, keeps its own.
#[test]
fn a_container_map_takes_the_slot_format_only_where_it_reads() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("a target");
        let half = (
            vec![0u8; 4 * 4 * 8],
            TextureFormat::Rgba16Float,
            [4u32, 4u32],
        );
        let half = Mipmap::build(half).expect("a float chain");
        let plain = Mipmap::build((vec![255u8; 4 * 4 * 4], [4u32, 4u32]));
        let plain = plain.expect("a chain");
        let cases: [(TextureInput, wgpu::TextureFormat); 4] = [
            (half.into(), wgpu::TextureFormat::Rgba16Float),
            (plain.into(), wgpu::TextureFormat::Rgba8UnormSrgb),
            (
                build_container(ktx2::Format::R16G16B16A16_SFLOAT, 8),
                wgpu::TextureFormat::Rgba16Float,
            ),
            (
                build_container(ktx2::Format::R8G8B8A8_UNORM, 4),
                wgpu::TextureFormat::Rgba8UnormSrgb,
            ),
        ];

        for (input, expected) in cases {
            let mat = build_mapped();
            mat.base_color_map().texture(input).expect("base color map");
            renderer
                .load(mat.shader())
                .await
                .expect("the source uploads");
            let id = read_map(&mat, "base_color_map");
            let copy = target.texture().context.get_texture(&id).expect("a copy");
            assert_eq!(copy.format, expected);
        }
    });
}

// Story: a `Texture` binds as it is, so the slot holds its id and no map.
#[test]
fn texture_setter_with_existing_texture_makes_no_map() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        #[rustfmt::skip]
            let pixels: [u8; 16] = [
                255, 0, 0, 255,    0, 255, 0, 255,
                  0, 0, 255, 255,  255, 255, 255, 255,
            ];
        let tex = renderer
            .create_texture((&pixels[..], Size::from((2u32, 2u32))))
            .await
            .expect("texture");

        let mat = PbrMaterial::new();
        mat.base_color_map().texture(&tex).expect("base color map");
        assert!(
            mat.shader.object.maps.read().is_empty(),
            "a Texture made a map"
        );
        let slot = mat.shader.object.get_uniform_data("base_color_map");
        assert!(
            matches!(&slot, Ok(UniformData::Texture(meta)) if meta.id == *tex.id()),
            "{slot:?}"
        );
    });
}

#[test]
fn shared_texture_handle_is_reused_across_materials() {
    // One GPU upload, N shader references. The texture ID stored in
    // each Material's `base_color_map` uniform points at the same
    // underlying TextureObject; the renderer's texture registry
    // dedupes by ID at draw time, so N Materials all wallpapered with
    // the same `&shared` texture pay for one GPU resource.
    pollster::block_on(async move {
        let renderer = Renderer::new();
        #[rustfmt::skip]
            let pixels: [u8; 16] = [
                255, 0, 0, 255,    0, 255, 0, 255,
                  0, 0, 255, 255,  255, 255, 255, 255,
            ];
        let shared = renderer
            .create_texture((&pixels[..], Size::from((2u32, 2u32))))
            .await
            .expect("texture");
        let shared_id = *shared.id();

        // 4 Materials, all reading `&shared`. Each Material's
        // `base_color_map` uniform stores the same texture ID.
        let mats: Vec<PbrMaterial> = (0..4)
            .map(|_| {
                let material = PbrMaterial::new();
                let map = material.base_color_map().texture(&shared);
                map.expect("base color map");
                material
            })
            .collect();

        for mat in &mats {
            let data = mat
                .shader()
                .object
                .get_uniform_data("base_color_map")
                .expect("base_color_map");
            match data {
                UniformData::Texture(meta) => {
                    assert_eq!(
                        meta.id, shared_id,
                        "each Material's base_color_map must point at the shared texture's ID"
                    );
                }
                other => panic!("expected UniformData::Texture, got {other:?}"),
            }
        }
    });
}

/// A material whose fullscreen shader draws texel (0, 0) of its base color map,
/// a 2x2 image of `color`.
fn build_painted(color: [u8; 4]) -> PbrMaterial {
    let material = build_mapped();
    let map = material.base_color_map().texture(paint(color));
    map.expect("base color map");
    material
}

/// A Renderer and an 8x8 target it made.
async fn open_target() -> (Renderer, TextureTarget) {
    let renderer = Renderer::new();
    let target = renderer.create_texture_target([8, 8]).await;
    (renderer, target.expect("a target"))
}

// Story: one material draws in two Renderers, each from a copy of its own.
#[test]
fn one_material_draws_in_two_renderers() {
    pollster::block_on(async move {
        let material = build_painted([255, 0, 0, 255]);
        let (first, first_target) = open_target().await;
        let (second, second_target) = open_target().await;

        first
            .render(material.shader(), &first_target)
            .expect("the first draws");
        second
            .render(material.shader(), &second_target)
            .expect("the second draws");

        assert_eq!(read_pixel(&first_target).await, [255, 0, 0, 255]);
        assert_eq!(read_pixel(&second_target).await, [255, 0, 0, 255]);
    });
}

// Story: a load in a second Renderer uploads its own copy of a map set again,
// and the first Renderer still draws, with the new map.
#[test]
fn loading_in_a_second_renderer_keeps_the_first() {
    pollster::block_on(async move {
        let material = build_painted([255, 0, 0, 255]);
        let (first, first_target) = open_target().await;
        let (second, second_target) = open_target().await;
        first
            .render(material.shader(), &first_target)
            .expect("the first draws");

        let map = material.base_color_map().texture(paint([0, 255, 0, 255]));
        map.expect("base color map");
        second.load(material.shader()).await.expect("the load");
        first
            .render(material.shader(), &first_target)
            .expect("the first still draws");
        second
            .render(material.shader(), &second_target)
            .expect("the second draws");

        assert_eq!(read_pixel(&first_target).await, [0, 255, 0, 255]);
        assert_eq!(read_pixel(&second_target).await, [0, 255, 0, 255]);
    });
}

// Story: a map set again reaches every Renderer that draws it.
#[test]
fn a_later_set_wins_in_every_renderer() {
    pollster::block_on(async move {
        let material = build_painted([255, 0, 0, 255]);
        let (first, first_target) = open_target().await;
        let (second, second_target) = open_target().await;
        for (renderer, target) in [(&first, &first_target), (&second, &second_target)] {
            renderer.render(material.shader(), target).expect("draws");
            assert_eq!(read_pixel(target).await, [255, 0, 0, 255]);
        }

        let map = material.base_color_map().texture(paint([0, 255, 0, 255]));
        map.expect("base color map");

        for (renderer, target) in [(&first, &first_target), (&second, &second_target)] {
            renderer.render(material.shader(), target).expect("draws");
            assert_eq!(read_pixel(target).await, [0, 255, 0, 255]);
        }
    });
}

// Story: a new Renderer draws a map after the Renderer that drew it first is gone,
// as an app does when it builds a new Renderer for a new surface.
#[test]
fn a_new_renderer_draws_after_the_first_drops() {
    pollster::block_on(async move {
        let material = build_painted([255, 0, 0, 255]);
        let (first, first_target) = open_target().await;
        first
            .render(material.shader(), &first_target)
            .expect("the first draws");
        drop((first, first_target));

        let (second, second_target) = open_target().await;
        second
            .render(material.shader(), &second_target)
            .expect("the new Renderer draws");

        assert_eq!(read_pixel(&second_target).await, [255, 0, 0, 255]);
    });
}

/// Writes `module` back as WGSL, as wgpu does for a browser, and parses that again.
fn write_for_the_web(module: &naga::Module) -> naga::Module {
    let info = Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(module)
        .expect("the module validates");
    let flags = WriterFlags::empty();
    let written = write_string(module, &info, flags).expect("writes WGSL");
    parse_str(&written).expect("the written WGSL parses")
}

/// The span and member offsets of every struct in `module`, in declaration order.
/// Names are left out, since the writer renames a struct whose name repeats.
fn read_layouts(module: &naga::Module) -> Vec<(u32, Vec<u32>)> {
    module
        .types
        .iter()
        .filter_map(|(_, ty)| match &ty.inner {
            naga::TypeInner::Struct { members, span } => {
                Some((*span, members.iter().map(|member| member.offset).collect()))
            }
            _ => None,
        })
        .collect()
}

// Story: on the web wgpu hands the browser the WGSL naga writes back, which drops
// @size and @align. The PBR shader's uniforms keep their layout through that
// round trip, on the storage path and on the uniform light path alike.
#[test]
fn pbr_layout_survives_the_web_round_trip() {
    // Arrange
    let material = PbrMaterial::new();
    let object = &material.shader().object;
    let uniform = object
        .lower_module(false, Some(4))
        .expect("the uniform light path lowers");
    let modules = [&object.module, uniform.as_ref()];

    // Act
    let written: Vec<naga::Module> = modules.iter().map(|m| write_for_the_web(m)).collect();

    // Assert
    for (module, written) in modules.iter().zip(&written) {
        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
        let checked = validator.validate(written);
        assert!(checked.is_ok(), "the web module fails: {checked:?}");
        assert_eq!(read_layouts(written), read_layouts(module));
    }
}

// Story: a double-sided PBR sheet seen from behind lights as the side the camera sees,
// since its tangent frame flips on the back face, as glTF asks. Unflipped, the light
// would reach the side turned away and the back would draw black.
#[test]
fn a_back_face_lights_as_the_side_the_camera_sees() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");
        let material = PbrMaterial::new().double_sided(true);
        let shader = material.shader();
        shader
            .set("camera.position", [0.0_f32, 0.0, 1.0])
            .expect("eye");
        shader.set("lights.ambient", [0.0_f32; 3]).expect("ambient");
        // The quad's corners wound clockwise, with a normal along -z, away from the eye.
        let back = Mesh::new();
        for position in [
            [-1.0, 1.0, 0.0],
            [1.0, -1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, -1.0, 0.0],
        ] {
            back.add_vertex(Vertex::pbr(position).set(Vertex::NORMAL, [0.0_f32, 0.0, -1.0]));
        }
        let pass = Pass::new("back");
        pass.add(&Model::new(back, &material))
            .expect("add the model");
        let light = Light::directional([0.0, 0.0, -1.0], [PI; 3]).expect("light");
        pass.add(&light).expect("add the light");

        // Act
        renderer.render(&pass, &target).expect("render");

        // Assert
        let pixel = read_pixel(&target).await;
        assert!(
            pixel[0] > 150,
            "the light reaches the side the camera sees: {pixel:?}"
        );
    });
}
