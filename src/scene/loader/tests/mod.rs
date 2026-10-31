//! `Scene::load` tests against well-formed and broken glTF files.

use std::sync::Arc;

use super::materials::map_sampler_options;
use super::meshes::{compute_vertex_normals, reserve};
use super::*;
use crate::texture::SourceData;
use crate::{Renderable, VertexData};

/// The minimal triangle: positions only, read through accessor 0 from
/// bufferView 0. Views 1 (12 bytes) and 2 (24 bytes) sit unused, so a
/// test can point a new accessor at them with a plain text replace.
const TRIANGLE_JSON: &str = r#"{"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":4}]}],"buffers":[{"byteLength":72}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0},{"buffer":0,"byteLength":12,"byteOffset":36},{"buffer":0,"byteLength":24,"byteOffset":48}],"accessors":[{"bufferView":0,"byteOffset":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]}],"asset":{"version":"2.0"}}"#;

/// Build a minimal valid .glb in memory: one triangle, positions only.
fn build_minimal_triangle_glb() -> Vec<u8> {
    build_triangle_glb(TRIANGLE_JSON)
}

/// Pack `json` with the triangle's BIN chunk: three vec3 positions, the
/// u32 indices `[0, 1, 2]` and 24 bytes of `[0.5, 0.5]` float pairs.
fn build_triangle_glb(json: &str) -> Vec<u8> {
    #[rustfmt::skip]
    let positions: [f32; 9] = [
         0.0,  0.5, 0.0,
        -0.5, -0.5, 0.0,
         0.5, -0.5, 0.0,
    ];
    let mut bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
    bin.extend([0u32, 1, 2].iter().flat_map(|i| i.to_le_bytes()));
    bin.extend([0.5f32; 6].iter().flat_map(|f| f.to_le_bytes()));
    build_glb(json, &bin)
}

/// Apply text patches to the minimal triangle's JSON and pack it.
fn mutate_triangle(patches: &[(&str, &str)]) -> Vec<u8> {
    let json = patches
        .iter()
        .fold(TRIANGLE_JSON.to_string(), |json, (from, to)| {
            assert!(json.contains(from), "patch target {from:?} not in the JSON");
            json.replace(from, to)
        });
    build_triangle_glb(&json)
}

/// Wrap a JSON chunk and a BIN chunk in a .glb container.
fn build_glb(json: &str, bin: &[u8]) -> Vec<u8> {
    let bin_len = bin.len() as u32;
    let mut json_bytes = json.as_bytes().to_vec();
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(b' ');
    }
    let json_len = json_bytes.len() as u32;
    let total = 12 + 8 + json_len + 8 + bin_len;

    let mut glb = Vec::with_capacity(total as usize);
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&total.to_le_bytes());
    glb.extend_from_slice(&json_len.to_le_bytes());
    glb.extend_from_slice(b"JSON");
    glb.extend_from_slice(&json_bytes);
    glb.extend_from_slice(&bin_len.to_le_bytes());
    glb.extend_from_slice(b"BIN\0");
    glb.extend_from_slice(bin);
    glb
}

/// Same triangle as `build_minimal_triangle_glb` but with one extra
/// node that holds a perspective camera, and one directional light
/// declared via `KHR_lights_punctual`. Gives the filter tests a real
/// camera + light to drop. The scene has three nodes: mesh (0),
/// camera (1) and light (2).
const CAMERA_LIGHT_JSON: &str = r#"{"asset":{"version":"2.0"},"extensionsUsed":["KHR_lights_punctual"],"extensions":{"KHR_lights_punctual":{"lights":[{"type":"directional","color":[1.0,0.95,0.9],"intensity":2.0}]}},"scene":0,"scenes":[{"nodes":[0,1,2]}],"nodes":[{"mesh":0},{"camera":0},{"extensions":{"KHR_lights_punctual":{"light":0}}}],"cameras":[{"type":"perspective","perspective":{"aspectRatio":1.5,"yfov":1.0,"znear":0.1,"zfar":100.0}}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":4}]}],"buffers":[{"byteLength":36}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0}],"accessors":[{"bufferView":0,"byteOffset":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]}]}"#;

fn build_triangle_with_camera_and_light_glb() -> Vec<u8> {
    build_camera_light_glb(CAMERA_LIGHT_JSON)
}

/// Pack a variant of `CAMERA_LIGHT_JSON` with the triangle's positions.
fn build_camera_light_glb(json: &str) -> Vec<u8> {
    #[rustfmt::skip]
    let positions: [f32; 9] = [
         0.0,  0.5, 0.0,
        -0.5, -0.5, 0.0,
         0.5, -0.5, 0.0,
    ];
    let bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
    build_glb(json, &bin)
}

const CAMERAS: &str = r#""cameras":[{"type":"perspective","perspective":{"aspectRatio":1.5,"yfov":1.0,"znear":0.1,"zfar":100.0}}]"#;

const DIRECTIONAL_LIGHT: &str = r#"{"type":"directional","color":[1.0,0.95,0.9],"intensity":2.0}"#;

#[test]
fn load_names_the_light_at_fault() {
    // Light 0 is fine; the node points at light 1, whose range is negative.
    let lights = format!(r#"{DIRECTIONAL_LIGHT},{{"type":"point","range":-5.0}}"#);
    let json = CAMERA_LIGHT_JSON
        .replace(DIRECTIONAL_LIGHT, &lights)
        .replace(
            r#"{"KHR_lights_punctual":{"light":0}}"#,
            r#"{"KHR_lights_punctual":{"light":1}}"#,
        );
    assert_invalid(
        build_camera_light_glb(&json),
        "glTF light 1: Light::set_range",
    );
}

#[test]
fn load_refuses_a_negative_light_range() {
    let json = CAMERA_LIGHT_JSON.replace(DIRECTIONAL_LIGHT, r#"{"type":"point","range":-5.0}"#);
    assert_invalid(
        build_camera_light_glb(&json),
        "glTF light 0: Light::set_range expects a non-negative number, got -5",
    );
}

/// Load a variant of the camera file and expect `SceneLoadError::Camera`
/// with a message that holds `needle`.
fn assert_camera_error(json: &str, needle: &str) {
    match Scene::load(build_camera_light_glb(json)) {
        Err(error @ SceneLoadError::Camera { .. }) => {
            let message = error.to_string();
            assert!(message.contains(needle), "{message:?} lacks {needle:?}")
        }
        Err(other) => panic!("expected SceneLoadError::Camera, got {other:?}"),
        Ok(_) => panic!("expected SceneLoadError::Camera, got a Scene"),
    }
}

#[test]
fn load_refuses_a_camera_with_no_field_of_view() {
    assert_camera_error(
        &CAMERA_LIGHT_JSON.replace(r#""yfov":1.0"#, r#""yfov":0.0"#),
        "glTF camera 0: Camera::perspective",
    );
}

#[test]
fn load_refuses_a_camera_node_with_zero_scale() {
    let json =
        CAMERA_LIGHT_JSON.replace(r#"{"camera":0}"#, r#"{"camera":0,"scale":[0.0,0.0,0.0]}"#);
    assert_camera_error(&json, "glTF camera 0: Camera::look_at");
}

#[test]
fn load_names_the_camera_at_fault() {
    // Camera 0 is fine; the node points at camera 1, whose yfov is 0 and which
    // has no zfar, so the error names its lens and no far plane the file never wrote.
    let cameras = r#""cameras":[{"type":"perspective","perspective":{"aspectRatio":1.5,"yfov":1.0,"znear":0.1,"zfar":100.0}},{"type":"perspective","perspective":{"yfov":0.0,"znear":0.1}}]"#;
    let json = CAMERA_LIGHT_JSON
        .replace(CAMERAS, cameras)
        .replace(r#"{"camera":0}"#, r#"{"camera":1}"#);
    assert_camera_error(
        &json,
        "glTF camera 1: a perspective camera with no zfar needs finite values with 0 < yfov < π, a nonzero aspect ratio and a positive znear, got yfov 0, aspect ratio 1, znear 0.1",
    );
}

#[test]
fn load_takes_a_camera_with_no_far_plane() {
    // glTF reads an absent zfar as an infinite projection: nothing ahead of
    // the near plane clips, however far, and any near plane builds one. Far
    // enough out f32 rounds the depth to 1, which the LessEqual test draws.
    let depth_at = |znear: &str, distance: f32| {
        let json = CAMERA_LIGHT_JSON.replace(
            r#""znear":0.1,"zfar":100.0"#,
            &format!(r#""znear":{znear}"#),
        );
        let scene = Scene::load(build_camera_light_glb(&json)).expect("load");
        let cameras = scene.cameras();
        let view_proj = glam::Mat4::from_cols_array_2d(&cameras[0].view_proj());
        assert!(view_proj.is_finite(), "{view_proj:?}");
        let clip = view_proj * glam::Vec4::new(0.0, 0.0, -distance, 1.0);
        clip.z / clip.w
    };
    for (znear, distance) in [("0.001", 1.0e6), ("0.1", 1.0e9), ("1.0e16", 1.0e30)] {
        let depth = depth_at(znear, distance);
        assert!(
            (0.0..=1.0).contains(&depth),
            "znear {znear}: a point {distance} ahead lands at depth {depth}"
        );
    }
}

#[test]
fn load_refuses_a_camera_whose_far_plane_is_nearer() {
    let json =
        CAMERA_LIGHT_JSON.replace(r#""znear":0.1,"zfar":100.0"#, r#""znear":10.0,"zfar":1.0"#);
    assert_invalid(
        build_camera_light_glb(&json),
        "camera 0 has zfar 1 at or before znear 10",
    );
}

#[test]
fn load_default_options_keeps_gltf_camera_and_light() {
    let bytes = build_triangle_with_camera_and_light_glb();
    let scene = Scene::load(bytes).expect("load");
    assert_eq!(scene.models().len(), 1, "geometry stays");
    assert_eq!(
        scene.cameras().len(),
        1,
        "default options load the embedded camera"
    );
    assert_eq!(
        scene.lights().len(),
        1,
        "default options load the embedded light"
    );
}

#[test]
fn cameras_filter_skips_gltf_camera_nodes() {
    let bytes = build_triangle_with_camera_and_light_glb();
    let scene = Scene::load(GltfSource::from(bytes).cameras(false)).expect("load");
    assert_eq!(scene.models().len(), 1, "geometry stays");
    assert!(
        scene.cameras().is_empty(),
        "cameras filter drops the camera node"
    );
    assert_eq!(scene.lights().len(), 1, "lights still load by default");
}

#[test]
fn lights_filter_skips_khr_lights_punctual() {
    let bytes = build_triangle_with_camera_and_light_glb();
    let scene = Scene::load(GltfSource::from(bytes).lights(false)).expect("load");
    assert_eq!(scene.models().len(), 1);
    assert_eq!(scene.cameras().len(), 1, "cameras still load by default");
    assert!(
        scene.lights().is_empty(),
        "lights filter drops the light node"
    );
}

#[test]
fn both_filters_off_yields_geometry_only_scene() {
    let bytes = build_triangle_with_camera_and_light_glb();
    let scene = Scene::load(GltfSource::from(bytes).cameras(false).lights(false)).expect("load");
    assert_eq!(scene.models().len(), 1, "geometry survives every filter");
    assert!(scene.cameras().is_empty());
    assert!(scene.lights().is_empty());
}

#[test]
fn load_minimal_triangle_glb_returns_scene_with_one_model() {
    let bytes = build_minimal_triangle_glb();
    let scene = Scene::load(bytes).expect("load triangle.glb");
    let passes = scene.passes();
    assert_eq!(passes.len(), 1, "expected one default pass");
    // The Scene's default pass should hold one Model entry (one
    // primitive in the test asset).
    assert_eq!(
        passes[0].model_entries.read().len(),
        1,
        "expected one Model on the default pass"
    );
}

#[test]
fn load_falls_back_through_into_for_path_inputs() {
    // Exercises the From<&str> -> SceneSource conversion (bare path, no
    // SceneSource named in the call); the actual file IO error is what
    // we want to see.
    let result = Scene::load("/definitely/not/a/real/path.glb");
    assert!(result.is_err(), "expected a load error for a bogus path");
}

/// Load `bytes` and expect `SceneLoadError::Invalid` naming `needle`.
fn assert_invalid(bytes: Vec<u8>, needle: &str) {
    match Scene::load(bytes) {
        Err(SceneLoadError::Invalid(message)) => {
            assert!(message.contains(needle), "{message:?} lacks {needle:?}")
        }
        Err(other) => panic!("expected SceneLoadError::Invalid, got {other:?}"),
        Ok(_) => panic!("expected SceneLoadError::Invalid, got a Scene"),
    }
}

const POSITION_VIEW: &str = r#"{"buffer":0,"byteLength":36,"byteOffset":0}"#;
const LAST_ACCESSOR: &str = r#""max":[0.5,0.5,0.0]}]"#;
const ATTRIBUTES: &str = r#"{"POSITION":0}"#;
const MODE: &str = r#""mode":4}"#;

#[test]
fn load_refuses_a_stride_shorter_than_the_element() {
    let bytes = mutate_triangle(&[(
        POSITION_VIEW,
        r#"{"buffer":0,"byteLength":36,"byteOffset":0,"byteStride":4}"#,
    )]);
    assert_invalid(bytes, "byteStride 4");
}

#[test]
fn load_refuses_float_indices() {
    let bytes = mutate_triangle(&[
        (MODE, r#""indices":1,"mode":4}"#),
        (
            LAST_ACCESSOR,
            r#""max":[0.5,0.5,0.0]},{"bufferView":1,"componentType":5126,"count":3,"type":"SCALAR"}]"#,
        ),
    ]);
    assert_invalid(bytes, "(indices) holds F32 Scalar");
}

#[test]
fn load_refuses_unsigned_int_tex_coords() {
    let bytes = mutate_triangle(&[
        (ATTRIBUTES, r#"{"POSITION":0,"TEXCOORD_0":1}"#),
        (
            LAST_ACCESSOR,
            r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5125,"count":3,"type":"VEC2"}]"#,
        ),
    ]);
    assert_invalid(bytes, "(TEXCOORD_0) holds U32 Vec2");
}

#[test]
fn load_refuses_vec2_colors() {
    let bytes = mutate_triangle(&[
        (ATTRIBUTES, r#"{"POSITION":0,"COLOR_0":1}"#),
        (
            LAST_ACCESSOR,
            r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5126,"count":3,"type":"VEC2"}]"#,
        ),
    ]);
    assert_invalid(bytes, "(COLOR_0) holds F32 Vec2");
}

#[test]
fn load_refuses_an_empty_position_accessor() {
    let bytes = mutate_triangle(&[(r#""count":3,"type":"VEC3""#, r#""count":0,"type":"VEC3""#)]);
    assert_invalid(bytes, "(POSITION) has count 0");
}

#[test]
fn load_refuses_an_empty_index_accessor() {
    let bytes = mutate_triangle(&[
        (MODE, r#""indices":1,"mode":4}"#),
        (
            LAST_ACCESSOR,
            r#""max":[0.5,0.5,0.0]},{"bufferView":1,"componentType":5125,"count":0,"type":"SCALAR"}]"#,
        ),
    ]);
    assert_invalid(bytes, "(indices) has count 0");
}

#[test]
fn load_refuses_a_count_that_overflows_the_byte_range() {
    let huge = mutate_triangle(&[(
        r#""count":3,"type":"VEC3""#,
        r#""count":4611686018427387904,"type":"VEC3""#,
    )]);
    assert_invalid(huge, "holds 4611686018427387904 elements");
    let largest = mutate_triangle(&[(
        r#""count":3,"type":"VEC3""#,
        r#""count":4294967295,"type":"VEC3""#,
    )]);
    assert_invalid(largest, "reads past the end of bufferView 0");
}

#[test]
fn load_refuses_an_accessor_past_its_view() {
    // A declared NORMAL the reader can't reach is an error, not a cue to
    // compute normals: its last element would end at byte 28 of 24.
    let bytes = mutate_triangle(&[
        (ATTRIBUTES, r#"{"POSITION":0,"NORMAL":1}"#),
        (
            LAST_ACCESSOR,
            r#""max":[0.5,0.5,0.0]},{"bufferView":2,"byteOffset":4,"componentType":5126,"count":2,"type":"VEC3"}]"#,
        ),
    ]);
    assert_invalid(bytes, "(NORMAL) reads past the end of bufferView 2");
}

#[test]
fn load_refuses_a_view_past_its_buffer() {
    let bytes = mutate_triangle(&[
        (
            r#"{"buffer":0,"byteLength":24,"byteOffset":48}"#,
            r#"{"buffer":0,"byteLength":24,"byteOffset":64}"#,
        ),
        (ATTRIBUTES, r#"{"POSITION":0,"TEXCOORD_0":1}"#),
        (
            LAST_ACCESSOR,
            r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5126,"count":3,"type":"VEC2"}]"#,
        ),
    ]);
    assert_invalid(bytes, "runs past the end of buffer 0");
}

#[test]
fn load_refuses_a_node_that_is_its_own_child() {
    let bytes = mutate_triangle(&[(
        r#""nodes":[{"mesh":0}]"#,
        r#""nodes":[{"mesh":0,"children":[0]}]"#,
    )]);
    assert_invalid(bytes, "glTF node 0 is reached twice");
}

#[test]
fn load_refuses_a_child_with_two_parents() {
    let bytes = mutate_triangle(&[(
        r#""scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}]"#,
        r#""scenes":[{"nodes":[0,1]}],"nodes":[{"children":[2]},{"children":[2]},{"mesh":0}]"#,
    )]);
    assert_invalid(bytes, "glTF node 2 is reached twice");
}

#[test]
fn load_names_the_mesh_and_primitive_at_fault() {
    // Mesh 0 is unused; the node draws mesh 1, whose third primitive is points.
    let triangle = r#"{"attributes":{"POSITION":0},"mode":4}"#;
    let meshes = format!(
        r#""meshes":[{{"primitives":[{triangle}]}},{{"primitives":[{triangle},{triangle},{{"attributes":{{"POSITION":0}},"mode":0}}]}}]"#
    );
    let bytes = mutate_triangle(&[
        (
            r#""meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":4}]}]"#,
            &meshes,
        ),
        (r#""nodes":[{"mesh":0}]"#, r#""nodes":[{"mesh":1}]"#),
    ]);
    assert_invalid(bytes, "glTF mesh 1 primitive 2 draws in mode Points");
}

#[test]
fn load_refuses_a_cycle_between_two_nodes() {
    let bytes = mutate_triangle(&[(
        r#""nodes":[{"mesh":0}]"#,
        r#""nodes":[{"mesh":0,"children":[1]},{"children":[0]}]"#,
    )]);
    assert_invalid(bytes, "glTF node 0 is reached twice");
}

const POSITION_ACCESSOR: &str = r#"{"bufferView":0,"byteOffset":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]}"#;

/// The triangle with POSITION as a sparse accessor of `count` elements
/// and no bufferView: zeros, except element 0 read from view 0.
fn sparse_positions(count: &str) -> Vec<u8> {
    let sparse = r#"{"componentType":5126,"count":COUNT,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0],"sparse":{"count":1,"indices":{"bufferView":1,"componentType":5125},"values":{"bufferView":0}}}"#;
    mutate_triangle(&[(POSITION_ACCESSOR, &sparse.replace("COUNT", count))])
}

#[test]
fn load_reads_a_sparse_accessor_without_a_view() {
    let scene = Scene::load(sparse_positions("3")).expect("load sparse POSITION");
    assert_eq!(scene.models().len(), 1);
}

#[test]
fn load_refuses_a_sparse_accessor_too_large_to_read() {
    // Without a view, nothing in the file bounds the count,
    // and 2^62 positions would overflow the allocation.
    assert_invalid(
        sparse_positions("4611686018427387904"),
        "(POSITION) holds 4611686018427387904 elements",
    );
    assert_invalid(
        sparse_positions("4294967296"),
        "(POSITION) holds 4294967296 elements",
    );
}

/// Pack a .glb whose BIN chunk holds the triangle's positions, one u32
/// sparse index of 0, one vec3 sparse value, then `extra` bytes.
fn build_sparse_glb(json: &str, extra: &[u8]) -> Vec<u8> {
    #[rustfmt::skip]
    let positions: [f32; 9] = [
         0.0,  0.5, 0.0,
        -0.5, -0.5, 0.0,
         0.5, -0.5, 0.0,
    ];
    let mut bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
    bin.extend(0u32.to_le_bytes());
    bin.extend([0.0f32, 0.1, 0.0].iter().flat_map(|f| f.to_le_bytes()));
    bin.extend_from_slice(extra);
    build_glb(json, &bin)
}

#[test]
fn load_ignores_a_bufferless_animation_track() {
    // A 100-key translation track whose output is sparse without a view,
    // the shape glTF-Transform's sparse() writes. The loader never reads
    // animations, so the track's 1200 bytes of zeros don't count.
    let times: Vec<u8> = (0..100)
        .flat_map(|key| (key as f32 / 3.0).to_le_bytes())
        .collect();
    let json = r#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":4}]}],"buffers":[{"byteLength":452}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0},{"buffer":0,"byteLength":4,"byteOffset":36},{"buffer":0,"byteLength":12,"byteOffset":40},{"buffer":0,"byteLength":400,"byteOffset":52}],"accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]},{"bufferView":3,"componentType":5126,"count":100,"type":"SCALAR","min":[0.0],"max":[33.0]},{"componentType":5126,"count":100,"type":"VEC3","sparse":{"count":1,"indices":{"bufferView":1,"componentType":5125},"values":{"bufferView":2}}}],"animations":[{"channels":[{"sampler":0,"target":{"node":0,"path":"translation"}}],"samplers":[{"input":1,"output":2}]}]}"#;
    let scene = Scene::load(build_sparse_glb(json, &times)).expect("load");
    assert_eq!(scene.models().len(), 1);
}

const UNUSED_ACCESSOR_JSON: &str = r#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":4}]}],"buffers":[{"byteLength":52}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0},{"buffer":0,"byteLength":4,"byteOffset":36},{"buffer":0,"byteLength":12,"byteOffset":40}],"accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]},UNUSED]}"#;

#[test]
fn load_ignores_an_unused_bufferless_accessor() {
    // A sparse accessor 1000 elements long that nothing references.
    let unused = r#"{"componentType":5126,"count":1000,"type":"VEC3","sparse":{"count":1,"indices":{"bufferView":1,"componentType":5125},"values":{"bufferView":2}}}"#;
    let json = UNUSED_ACCESSOR_JSON.replace("UNUSED", unused);
    let scene = Scene::load(build_sparse_glb(&json, &[])).expect("load");
    assert_eq!(scene.models().len(), 1);
}

#[test]
fn load_reports_an_all_zero_accessor_as_a_parse_error() {
    // glTF reads an accessor with neither a view nor sparse values as all
    // zeros, but the glTF parser refuses it before the loader runs.
    let json = UNUSED_ACCESSOR_JSON.replace(
        "UNUSED",
        r#"{"componentType":5126,"count":1000,"type":"VEC3"}"#,
    );
    assert!(matches!(
        Scene::load(build_sparse_glb(&json, &[])),
        Err(SceneLoadError::Gltf(_))
    ));
}

#[test]
fn load_reads_a_bufferless_color_beside_dense_positions() {
    // 99 positions take 1188 bytes. Their sparse COLOR_0 spans 1584 bytes
    // of zeros, more than the file holds, but it can't outgrow POSITION.
    let json = r#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0,"COLOR_0":1},"mode":4}]}],"buffers":[{"byteLength":1208}],"bufferViews":[{"buffer":0,"byteLength":1188,"byteOffset":0},{"buffer":0,"byteLength":4,"byteOffset":1188},{"buffer":0,"byteLength":16,"byteOffset":1192}],"accessors":[{"bufferView":0,"componentType":5126,"count":99,"type":"VEC3","min":[0.0,0.0,0.0],"max":[33.0,1.0,0.0]},{"componentType":5126,"count":99,"type":"VEC4","sparse":{"count":1,"indices":{"bufferView":1,"componentType":5125},"values":{"bufferView":2}}}]}"#;
    let mut bin: Vec<u8> = (0..99)
        .flat_map(|vertex| [vertex as f32 / 3.0, (vertex % 3 == 1) as u8 as f32, 0.0])
        .flat_map(|value| value.to_le_bytes())
        .collect();
    bin.extend(0u32.to_le_bytes());
    bin.extend([1.0f32; 4].iter().flat_map(|f| f.to_le_bytes()));
    let scene = Scene::load(build_glb(json, &bin)).expect("load");
    assert_eq!(scene.models().len(), 1);
}

#[test]
fn load_refuses_a_sparse_accessor_larger_than_the_file() {
    // The file holds 72 bytes of buffers. Its one sparse value can't stand
    // for 100,000 positions, and a count just under u32::MAX would ask for
    // 51 GB that an overcommitting system grants and then can't fill.
    assert_invalid(
        sparse_positions("100000"),
        "glTF accessor 0 (POSITION) spans 1200000 bytes without a bufferView, more than the 72 bytes",
    );
    assert_invalid(
        sparse_positions("4294967295"),
        "glTF accessor 0 (POSITION) spans 51539607540 bytes without a bufferView",
    );
}

#[test]
fn reserve_refuses_a_count_the_allocator_cant_give() {
    // Every Vec the loader sizes from a count goes through `reserve`, so a
    // refused allocation is an error there instead of an abort.
    let refused = reserve::<[f32; 3]>(usize::MAX, "glTF accessor 0 (POSITION)");
    let message = refused
        .expect_err("usize::MAX positions overflow")
        .to_string();
    assert!(
        message.contains("glTF accessor 0 (POSITION) needs"),
        "{message}"
    );
}

#[test]
fn load_refuses_sparse_indices_too_large_to_read() {
    let indices = r#""max":[0.5,0.5,0.0]},{"componentType":5125,"count":4611686018427387904,"type":"SCALAR","sparse":{"count":1,"indices":{"bufferView":1,"componentType":5125},"values":{"bufferView":1}}}]"#;
    let bytes = mutate_triangle(&[(MODE, r#""indices":1,"mode":4}"#), (LAST_ACCESSOR, indices)]);
    assert_invalid(bytes, "(indices) holds 4611686018427387904 elements");
}

#[test]
fn load_reads_valid_indices_and_uvs() {
    // The checks must not refuse the accessors they exist to protect.
    let bytes = mutate_triangle(&[
        (ATTRIBUTES, r#"{"POSITION":0,"TEXCOORD_0":1}"#),
        (MODE, r#""indices":2,"mode":4}"#),
        (
            LAST_ACCESSOR,
            r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5126,"count":3,"type":"VEC2"},{"bufferView":1,"componentType":5125,"count":3,"type":"SCALAR"}]"#,
        ),
    ]);
    let scene = Scene::load(bytes).expect("load");
    assert_eq!(scene.models().len(), 1);
}

/// A unit quad, four positions read through accessor 0, drawn as a strip.
const QUAD_JSON: &str = r#"{"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":5}]}],"buffers":[{"byteLength":48}],"bufferViews":[{"buffer":0,"byteLength":48,"byteOffset":0}],"accessors":[{"bufferView":0,"byteOffset":0,"componentType":5126,"count":4,"type":"VEC3","min":[0.0,0.0,0.0],"max":[1.0,1.0,0.0]}],"asset":{"version":"2.0"}}"#;

/// Load the quad with `mode` and return the indices its mesh draws.
fn load_quad_indices(mode: &str) -> Vec<u32> {
    #[rustfmt::skip]
    let positions: [f32; 12] = [
        0.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        1.0, 1.0, 0.0,
    ];
    let bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
    let json = QUAD_JSON.replace(r#""mode":5"#, mode);
    let scene = Scene::load(build_glb(&json, &bin)).expect("load quad");
    let models = scene.models();
    let mesh = &models[0].mesh().object;
    assert!(*mesh.indices_overridden.read(), "{mode} should set indices");
    mesh.indices.read().clone()
}

#[test]
fn load_expands_a_triangle_strip() {
    // Every second strip triangle swaps its first two corners, so both
    // triangles keep the strip's winding.
    assert_eq!(load_quad_indices(r#""mode":5"#), [0, 1, 2, 2, 1, 3]);
}

#[test]
fn load_expands_a_triangle_fan() {
    assert_eq!(load_quad_indices(r#""mode":6"#), [0, 1, 2, 0, 2, 3]);
}

#[test]
fn load_refuses_points_and_lines() {
    for (mode, name) in [
        ("0", "Points"),
        ("1", "Lines"),
        ("2", "LineLoop"),
        ("3", "LineStrip"),
    ] {
        let bytes = mutate_triangle(&[(MODE, &format!(r#""mode":{mode}}}"#))]);
        assert_invalid(
            bytes,
            &format!("glTF mesh 0 primitive 0 draws in mode {name}"),
        );
    }
}

#[test]
fn load_refuses_an_attribute_shorter_than_positions() {
    let bytes = mutate_triangle(&[
        (ATTRIBUTES, r#"{"POSITION":0,"NORMAL":1}"#),
        (
            LAST_ACCESSOR,
            r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5126,"count":2,"type":"VEC3"}]"#,
        ),
    ]);
    assert_invalid(bytes, "(NORMAL) holds 2 elements for 3 positions");
}

// Story: glTF draws TRIANGLES in threes, so a list that ends partway through a
// triangle, indexed or not, is refused with the primitive named.
#[test]
fn load_refuses_a_partial_triangle_list() {
    #[rustfmt::skip]
    let positions: [f32; 12] = [
        0.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        1.0, 1.0, 0.0,
    ];
    let bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
    let json = QUAD_JSON.replace(r#""mode":5"#, r#""mode":4"#);
    assert_invalid(
        build_glb(&json, &bin),
        "glTF mesh 0 primitive 0 has 4 vertices and no indices, and a triangle list takes a multiple of 3",
    );

    let bytes = mutate_triangle(&[
        (MODE, r#""indices":1,"mode":4}"#),
        (
            LAST_ACCESSOR,
            r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5125,"count":4,"type":"SCALAR"}]"#,
        ),
    ]);
    assert_invalid(
        bytes,
        "glTF mesh 0 primitive 0 has 4 indices, and a triangle list takes a multiple of 3",
    );
}

#[test]
fn load_refuses_an_index_past_the_last_vertex() {
    // View 2 holds 0.5 floats, which read as u32 0x3F00_0000.
    let bytes = mutate_triangle(&[
        (MODE, r#""indices":1,"mode":4}"#),
        (
            LAST_ACCESSOR,
            r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5125,"count":3,"type":"SCALAR"}]"#,
        ),
    ]);
    assert_invalid(
        bytes,
        "glTF mesh 0 primitive 0 index 1056964608 points past the last of 3 vertices",
    );
}

/// The minimal triangle with a base color texture read from an embedded PNG.
const TEXTURED_JSON: &str = r#"{"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"material":0,"mode":4}]}],"materials":[{"pbrMetallicRoughness":{"baseColorTexture":{"index":0}}}],"textures":[{"source":0}],"images":[{"bufferView":1,"mimeType":"image/png"}],"buffers":[{"byteLength":BUFFER_LENGTH}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0},{"buffer":0,"byteLength":PNG_LENGTH,"byteOffset":36}],"accessors":[{"bufferView":0,"byteOffset":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]}],"asset":{"version":"2.0"}}"#;

/// `image` encoded as a PNG file's bytes.
fn encode_png(image: image::DynamicImage) -> Vec<u8> {
    let mut png = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut png, image::ImageFormat::Png)
        .expect("encode PNG");
    png.into_inner()
}

/// Load the textured triangle with the encoded `png` as its base color image.
fn load_png(png: &[u8]) -> Result<Scene, SceneLoadError> {
    #[rustfmt::skip]
    let positions: [f32; 9] = [
         0.0,  0.5, 0.0,
        -0.5, -0.5, 0.0,
         0.5, -0.5, 0.0,
    ];
    let mut bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
    bin.extend_from_slice(png);
    let json = TEXTURED_JSON
        .replace("BUFFER_LENGTH", &bin.len().to_string())
        .replace("PNG_LENGTH", &png.len().to_string());
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    Scene::load(build_glb(&json, &bin))
}

/// The source the base color map of `scene`'s first model keeps.
fn read_source(scene: &Scene) -> Arc<SourceData> {
    let models = scene.models();
    let maps = models[0].material().shader().object.maps.read();
    assert_eq!(maps.len(), 1, "the base color texture should be a map");
    maps[0].source.data.clone()
}

/// The base color material of the textured triangle, which the UV tests replace.
const TEXTURED_MATERIAL: &str =
    r#""materials":[{"pbrMetallicRoughness":{"baseColorTexture":{"index":0}}}]"#;

/// Load the textured triangle with `materials` in place of its own,
/// every texture reading one 2×2 PNG.
fn load_textured(materials: &str) -> Result<Scene, SceneLoadError> {
    let white = image::ImageBuffer::from_pixel(2, 2, image::Rgba([255u8; 4]));
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(white)
        .write_to(&mut png, image::ImageFormat::Png)
        .expect("encode PNG");
    let png = png.into_inner();
    #[rustfmt::skip]
    let positions: [f32; 9] = [
         0.0,  0.5, 0.0,
        -0.5, -0.5, 0.0,
         0.5, -0.5, 0.0,
    ];
    let mut bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
    bin.extend_from_slice(&png);
    assert!(TEXTURED_JSON.contains(TEXTURED_MATERIAL));
    let json = TEXTURED_JSON
        .replace(TEXTURED_MATERIAL, materials)
        .replace("BUFFER_LENGTH", &bin.len().to_string())
        .replace("PNG_LENGTH", &png.len().to_string());
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    Scene::load(build_glb(&json, &bin))
}

/// The UV set and transform of the material slot at `index`.
fn read_slot(material: &Material, index: usize) -> (u32, [f32; 2], [f32; 2], f32) {
    let read = |field: &str| format!("material.uv[{index}].{field}");
    let shader = material.shader();
    (
        shader.get(&read("uv_set")).expect("uv_set"),
        shader.get(&read("offset")).expect("offset"),
        shader.get(&read("scale")).expect("scale"),
        shader.get(&read("rotation")).expect("rotation"),
    )
}

/// JSON written with single quotes, so each line stays one Rust string.
fn quote(lines: &[&str]) -> String {
    lines.concat().replace('\'', "\"")
}

// Story: an ORM image that the metallic-roughness and occlusion slots both read
// is read once, and the two maps share its one source.
#[test]
fn two_slots_that_read_one_image_share_one_source() {
    let materials = quote(&[
        "'materials':[{'pbrMetallicRoughness':{'metallicRoughnessTexture':{'index':0}},",
        "'occlusionTexture':{'index':0}}]",
    ]);
    let scene = load_textured(&materials).expect("load");
    let models = scene.models();
    let maps = models[0].material().shader().object.maps.read();
    let read = |key: &str| {
        let map = maps.iter().find(|map| map.key == key);
        map.map(|map| map.source.data.clone()).expect("a map")
    };

    let roughness = read("metallic_roughness_map");
    let occlusion = read("occlusion_map");

    assert!(
        Arc::ptr_eq(&roughness, &occlusion),
        "each slot read a copy of its own"
    );
}

#[test]
fn load_reads_each_texture_coord_and_transform() {
    // Base color and occlusion carry different transforms. Normal moves to
    // set 1 and emissive back to set 0 through the extension's texCoord.
    let materials = quote(&[
        "'materials':[{'pbrMetallicRoughness':{'baseColorTexture':{'index':0,",
        "'extensions':{'KHR_texture_transform':{'offset':[0.5,0.0],'scale':[2.0,2.0],'rotation':0.25}}}},",
        "'occlusionTexture':{'index':0,'texCoord':1,",
        "'extensions':{'KHR_texture_transform':{'offset':[0.0,0.25],'scale':[4.0,4.0],'rotation':1.0}}},",
        "'normalTexture':{'index':0,'extensions':{'KHR_texture_transform':{'texCoord':1}}},",
        "'emissiveTexture':{'index':0,'texCoord':1,'extensions':{'KHR_texture_transform':{'texCoord':0}}}}]",
    ]);
    let scene = load_textured(&materials).expect("load");
    let models = scene.models();
    let material = models[0].material();
    assert_eq!(read_slot(material, 0), (0, [0.5, 0.0], [2.0, 2.0], 0.25));
    assert_eq!(read_slot(material, 1), (0, [0.0, 0.0], [1.0, 1.0], 0.0));
    assert_eq!(read_slot(material, 2), (1, [0.0, 0.0], [1.0, 1.0], 0.0));
    assert_eq!(read_slot(material, 3), (1, [0.0, 0.25], [4.0, 4.0], 1.0));
    assert_eq!(read_slot(material, 4), (0, [0.0, 0.0], [1.0, 1.0], 0.0));
}

#[test]
fn load_reads_a_null_texture_transform_as_none_on_every_slot() {
    // A null KHR_texture_transform is no transform on any slot.
    let materials = quote(&[
        "'materials':[{'pbrMetallicRoughness':{",
        "'baseColorTexture':{'index':0,'extensions':{'KHR_texture_transform':null}},",
        "'metallicRoughnessTexture':{'index':0,'extensions':{'KHR_texture_transform':null}}},",
        "'normalTexture':{'index':0,'texCoord':1,'extensions':{'KHR_texture_transform':null}},",
        "'occlusionTexture':{'index':0,'extensions':{'KHR_texture_transform':null}},",
        "'emissiveTexture':{'index':0,'extensions':{'KHR_texture_transform':null}}}]",
    ]);
    let scene = load_textured(&materials).expect("a null transform loads");
    let models = scene.models();
    let material = models[0].material();
    for index in [0, 1, 3, 4] {
        assert_eq!(read_slot(material, index), (0, [0.0, 0.0], [1.0, 1.0], 0.0));
    }
    assert_eq!(read_slot(material, 2), (1, [0.0, 0.0], [1.0, 1.0], 0.0));
}

#[test]
fn load_refuses_a_third_texture_coord() {
    let occlusion = quote(&["'materials':[{'occlusionTexture':{'index':0,'texCoord':2}}]"]);
    match load_textured(&occlusion) {
        Err(SceneLoadError::Invalid(message)) => {
            assert!(
                message.contains("occlusionTexture reads TEXCOORD_2"),
                "{message}"
            );
        }
        other => panic!("expected SceneLoadError::Invalid, got {other:?}"),
    }
    let base = quote(&[
        "'materials':[{'pbrMetallicRoughness':{'baseColorTexture':{'index':0,",
        "'extensions':{'KHR_texture_transform':{'texCoord':3}}}}}]",
    ]);
    match load_textured(&base) {
        Err(SceneLoadError::Invalid(message)) => {
            assert!(
                message.contains("baseColorTexture reads TEXCOORD_3"),
                "{message}"
            );
        }
        other => panic!("expected SceneLoadError::Invalid, got {other:?}"),
    }
}

/// Load the triangle with `attributes` and the 0.5 pairs of view 2 and
/// the position floats of view 0 as two VEC2 accessors, 1 and 2.
/// Returns the UV0 and UV1 of each vertex.
fn load_vertex_uvs(attributes: &str) -> Vec<(VertexData, VertexData)> {
    let bytes = mutate_triangle(&[
        (ATTRIBUTES, attributes),
        (
            LAST_ACCESSOR,
            r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5126,"count":3,"type":"VEC2"},{"bufferView":0,"componentType":5126,"count":3,"type":"VEC2"}]"#,
        ),
    ]);
    let scene = Scene::load(bytes).expect("load");
    let models = scene.models();
    let verts = models[0].mesh().object.verts.read();
    verts
        .iter()
        .map(|vertex| {
            let read = |key: &str| vertex.properties.get(key).copied().expect(key);
            (read(crate::Vertex::UV0), read(crate::Vertex::UV1))
        })
        .collect()
}

#[test]
fn load_reads_texcoord_1_into_uv1() {
    let uvs = load_vertex_uvs(r#"{"POSITION":0,"TEXCOORD_0":1,"TEXCOORD_1":2}"#);
    let uv1: Vec<VertexData> = uvs.iter().map(|(_, uv1)| *uv1).collect();
    assert_eq!(
        uv1,
        [
            VertexData::F32x2([0.0, 0.5]),
            VertexData::F32x2([0.0, -0.5]),
            VertexData::F32x2([-0.5, 0.0]),
        ]
    );
    assert!(
        uvs.iter()
            .all(|(uv0, _)| *uv0 == VertexData::F32x2([0.5, 0.5]))
    );
}

#[test]
fn load_gives_uv0_to_uv1_without_texcoord_1() {
    let uvs = load_vertex_uvs(r#"{"POSITION":0,"TEXCOORD_0":2}"#);
    for (uv0, uv1) in uvs {
        assert_eq!(uv1, uv0);
        assert_ne!(
            uv0,
            VertexData::F32x2([0.0, 0.0]),
            "UV0 comes from the file"
        );
    }
}

// Story: a map keeps its glTF image as the file holds it, so the scene keeps the
// encoded size in RAM, and each Renderer decodes the image when it loads the map.
#[test]
fn a_map_keeps_the_encoded_image() {
    let gray = image::ImageBuffer::from_pixel(8, 8, image::Luma([u16::MAX]));
    let png = encode_png(image::DynamicImage::ImageLuma16(gray));
    let scene = load_png(&png).expect("load");

    let source = read_source(&scene);
    match &*source {
        SourceData::Bytes(bytes) => {
            assert_eq!(bytes.len(), png.len(), "the scene keeps more than the file");
            assert_eq!(bytes, &png, "the scene keeps other bytes than the file's");
        }
        other => panic!("expected the encoded image, got {other:?}"),
    }
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        renderer
            .load(&scene)
            .await
            .expect("the 16-bit image decodes at load");
    });
}

// Story: an image whose header reads but whose body is corrupt loads with the scene,
// and fails at the first load into a Renderer with an error that names the image.
#[test]
fn a_corrupt_image_body_fails_at_the_first_load() {
    let noise = image::ImageBuffer::from_fn(32, 32, |x, y| {
        image::Rgba([(x * 7) as u8, (y * 13) as u8, (x * y) as u8, 255])
    });
    let png = encode_png(image::DynamicImage::ImageRgba8(noise));
    let cut = &png[..png.len() / 2];
    let scene = load_png(cut).expect("the header reads, so the scene loads");

    let result = pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        renderer.load(&scene).await
    });

    let error = result.expect_err("the corrupt body uploaded");
    assert!(error.to_string().contains("glTF image 0"), "{error}");
}

// Story: an image whose header can't be read fails `Scene::load` and names the image,
// as a file of another format or of no pixels does, before any Renderer sees it.
#[test]
fn an_unreadable_image_header_fails_scene_load() {
    let error = load_png(b"not an image at all").expect_err("the scene loaded");
    assert!(
        matches!(&error, SceneLoadError::Invalid(message) if message.starts_with("glTF image 0")),
        "{error:?}"
    );
}

// Story: an image in a file beside a `.gltf` keeps only its path, which each Renderer
// reads when it loads the map, and the name in the URI is percent-decoded.
#[test]
fn an_external_image_keeps_its_path() {
    let folder = std::env::temp_dir().join(format!("fc-gltf-{}", std::process::id()));
    std::fs::create_dir_all(&folder).expect("a folder");
    let white = image::ImageBuffer::from_pixel(2, 2, image::Rgba([255u8; 4]));
    let png = encode_png(image::DynamicImage::ImageRgba8(white));
    std::fs::write(folder.join("base color.png"), &png).expect("the image");
    #[rustfmt::skip]
    let positions: [f32; 9] = [
         0.0,  0.5, 0.0,
        -0.5, -0.5, 0.0,
         0.5, -0.5, 0.0,
    ];
    let bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
    std::fs::write(folder.join("triangle.bin"), &bin).expect("the buffer");
    let json = quote(&[
        "{'scene':0,'scenes':[{'nodes':[0]}],'nodes':[{'mesh':0}],",
        "'meshes':[{'primitives':[{'attributes':{'POSITION':0},'material':0,'mode':4}]}],",
        "'materials':[{'pbrMetallicRoughness':{'baseColorTexture':{'index':0}}}],",
        "'textures':[{'source':0}],'images':[{'uri':'base%20color.png'}],",
        "'buffers':[{'byteLength':36,'uri':'triangle.bin'}],",
        "'bufferViews':[{'buffer':0,'byteLength':36,'byteOffset':0}],",
        "'accessors':[{'bufferView':0,'byteOffset':0,'componentType':5126,'count':3,",
        "'type':'VEC3','min':[-0.5,-0.5,0.0],'max':[0.5,0.5,0.0]}],'asset':{'version':'2.0'}}",
    ]);
    let path = folder.join("triangle.gltf");
    std::fs::write(&path, json).expect("the glTF");

    let scene = Scene::load(path.as_path()).expect("load");

    let source = read_source(&scene);
    assert!(
        matches!(&*source, SourceData::Path(found) if *found == folder.join("base color.png")),
        "{source:?}"
    );
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        renderer
            .load(&scene)
            .await
            .expect("the image reads at load");
    });
    std::fs::remove_dir_all(&folder).expect("cleans up");
}

#[test]
fn compute_vertex_normals_indexed_yz_face() {
    // Triangle in the YZ plane. Face normal = +X. Without
    // face-normal computation the fallback +Z would shade this
    // triangle as if it pointed forward — visibly wrong.
    let positions = [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let indices = [0u32, 1, 2];
    let normals = compute_vertex_normals("test", &positions, Some(&indices)).expect("normals");
    for n in normals {
        assert!((n[0] - 1.0).abs() < 1.0e-6, "got {n:?}");
        assert!(n[1].abs() < 1.0e-6);
        assert!(n[2].abs() < 1.0e-6);
    }
}

#[test]
fn compute_vertex_normals_non_indexed_walks_triplets() {
    // Same YZ-plane triangle but unindexed: positions in sequential
    // triplets. The loader treats every three consecutive positions
    // as one face.
    let positions = [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let normals = compute_vertex_normals("test", &positions, None).expect("normals");
    for n in normals {
        assert!((n[0] - 1.0).abs() < 1.0e-6, "got {n:?}");
    }
}

#[test]
fn compute_vertex_normals_averages_shared_vertex() {
    // Two coplanar triangles sharing vertex 0 — both contribute the
    // same +Z face normal, so the shared vertex normalizes to +Z too
    // (un-degenerate after normalization).
    let positions = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, -1.0, 0.0],
    ];
    let indices = [0u32, 1, 2, 0, 3, 4];
    let normals = compute_vertex_normals("test", &positions, Some(&indices)).expect("normals");
    assert!((normals[0][2] - 1.0).abs() < 1.0e-6, "shared {normals:?}");
}

#[test]
fn map_sampler_options_handles_repeat_and_nearest() {
    // Build a minimal glTF programmatically that includes a sampler
    // with REPEAT wrap + NEAREST filtering, then exercise the
    // translation to FragmentColor's SamplerOptions.
    let json = r#"{
            "asset": { "version": "2.0" },
            "scene": 0,
            "scenes": [{ "nodes": [] }],
            "samplers": [{
                "magFilter": 9728,
                "minFilter": 9728,
                "wrapS": 10497,
                "wrapT": 10497
            }]
        }"#;
    let doc = gltf::Gltf::from_slice(json.as_bytes())
        .expect("parse glTF JSON")
        .document;
    let sampler = doc.samplers().next().expect("sampler");
    let opts = map_sampler_options(&sampler);
    assert!(opts.repeat_x);
    assert!(opts.repeat_y);
    assert!(
        !opts.smooth,
        "magFilter=9728 (NEAREST) should set smooth=false"
    );
}
