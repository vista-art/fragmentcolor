#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Renderer, Scene, Target};

// A quad over the whole view with an occlusion map on TEXCOORD_1. The map
// is white on its left half and black on its right. TEXCOORD_0 stays on the
// white half, and TEXCOORD_1 spans the whole map.
const OCCLUSION_UV1: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/assets/occlusion_uv1.gltf"
);

const SIZE: u32 = 16;

/// The RGB of the pixel at `row` and `column`.
fn read_pixel(image: &[u8], row: u32, column: u32) -> [u8; 3] {
    let start = ((row * SIZE + column) * 4) as usize;
    [image[start], image[start + 1], image[start + 2]]
}

// Story: an occlusion map on TEXCOORD_1 darkens the quad where UV1 points.
#[test]
fn occlusion_on_texcoord_1_reads_uv1() {
    pollster::block_on(async {
        // Arrange: draw the quad where it sits in clip space, lit by a white
        // ambient term alone, so each pixel shows the occlusion it samples.
        let scene = Scene::load(OCCLUSION_UV1).expect("load occlusion_uv1.gltf");
        scene.no_defaults().ambient([1.0, 1.0, 1.0]);
        for model in scene.models() {
            let shader = model.material().shader();
            shader.set("lights.count", 0u32).expect("light count");
        }
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([SIZE, SIZE])
            .await
            .expect("texture target");

        // Act
        renderer.render(&scene, &target).expect("render the scene");
        let image = target.get_image().await.expect("read back");

        // Assert: UV1 reaches the black half on the right, which UV0 never does.
        assert_eq!(read_pixel(&image, 8, 1), [255, 255, 255], "left edge");
        assert_eq!(read_pixel(&image, 8, 14), [0, 0, 0], "right edge");
    });
}
