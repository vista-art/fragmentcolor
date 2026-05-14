//! Build a Scene from glTF bytes via `Scene::load` and render it.
//!
//! Demonstrates the loader's end-to-end path: construct a minimal in-memory
//! `.glb` (a single triangle, positions only — `Scene::load` fills in
//! safe defaults for missing normals + UVs), hand it to `Scene::load`, and
//! render the resulting scene with the renderer's default Camera + Light
//! injected at draw time. In real use you'd hand `Scene::load` a path
//! string for a `.gltf` / `.glb` file on disk, or `.glb` bytes fetched
//! from the network or unwrapped from another container.

use fragmentcolor::{Renderer, Scene, SceneSource, Target};

fn main() {
    let bytes = build_minimal_triangle_glb();
    let scene = Scene::load(SceneSource::gltf(bytes)).expect("Scene::load");

    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([256u32, 256u32])
            .await
            .expect("texture target");
        renderer.render(&scene, &target).expect("render");
        let image = target.get_image().await;
        println!(
            "rendered {} bytes from glTF scene — first pixel: ({}, {}, {}, {})",
            image.len(),
            image[0],
            image[1],
            image[2],
            image[3],
        );
    });
}

/// Hand-build a valid `.glb` payload in memory: one triangle, positions
/// only. Three vec3 floats in the BIN chunk; the JSON chunk wires them
/// up through a single buffer / bufferView / accessor / mesh primitive /
/// node / scene per the glTF 2.0 spec.
fn build_minimal_triangle_glb() -> Vec<u8> {
    #[rustfmt::skip]
    let positions: [f32; 9] = [
         0.0,  0.5, 0.0,
        -0.5, -0.5, 0.0,
         0.5, -0.5, 0.0,
    ];
    let bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
    let bin_len = bin.len() as u32;

    let json = r#"{"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":4}]}],"buffers":[{"byteLength":36}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0}],"accessors":[{"bufferView":0,"byteOffset":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]}],"asset":{"version":"2.0"}}"#;
    let mut json_bytes = json.as_bytes().to_vec();
    while json_bytes.len() % 4 != 0 {
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
    glb.extend_from_slice(&bin);
    glb
}
