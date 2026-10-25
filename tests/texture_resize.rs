#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::texture::TextureError;
use fragmentcolor::{
    Material, Mesh, Model, Pass, Renderer, Target, Texture, TextureData, TextureFormat,
    TextureInput, TextureOptions, TextureUsage, Vertex,
};

async fn drawable(renderer: &Renderer, size: [u32; 2]) -> Texture {
    let usage = TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC | TextureUsage::COPY_DST;
    let options = TextureOptions {
        size: Some(size.into()),
        format: TextureFormat::Rgba,
        ..Default::default()
    }
    .with_usage(usage);
    let input = TextureInput {
        data: TextureData::Empty,
        options,
    };
    renderer
        .create_storage_texture(input)
        .await
        .expect("create_storage_texture failed")
}

fn size_of(texture: &Texture) -> [u32; 2] {
    texture.size().into()
}

// Story: a resize reaches every clone of the handle, and the texture takes
// writes and reads at its new size.
#[test]
fn clones_follow_a_resize() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let texture = drawable(&renderer, [4, 4]).await;
        let copy = texture.clone();
        texture.resize([8, 2]).expect("resize failed");
        assert_eq!(size_of(&copy), [8, 2], "the clone reads the new size");
        assert_eq!(copy.id(), texture.id(), "the id stays");

        let pixels = vec![200; 8 * 2 * 4];
        copy.write(&pixels).expect("write failed");
        let image = texture.get_image().await.expect("read back");
        assert_eq!(image, pixels, "both handles reach the same texture");
    });
}

// Story: the current size keeps the texture and its pixels.
#[test]
fn the_current_size_keeps_the_pixels() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let texture = drawable(&renderer, [2, 2]).await;
        let pixels: Vec<u8> = (0..16).collect();
        texture.write(&pixels).expect("write failed");
        texture.resize(texture.size()).expect("resize failed");
        let image = texture.get_image().await.expect("read back");
        assert_eq!(image, pixels);
    });
}

// Story: a texture no pass draws into keeps its size, and so does any texture
// asked for a side of zero.
#[test]
fn resize_refuses_and_keeps_the_size() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let pixels: &[u8] = &[255, 255, 255, 255];
        let source = renderer
            .create_texture((pixels, [1, 1]))
            .await
            .expect("create_texture failed");
        let refused = source.resize([2, 2]);
        assert!(
            matches!(refused, Err(TextureError::FixedSize)),
            "{refused:?}"
        );
        assert_eq!(size_of(&source), [1, 1]);

        let storage = renderer
            .create_storage_texture(([4, 4], TextureFormat::Rgba))
            .await
            .expect("create_storage_texture failed");
        let refused = storage.resize([8, 8]);
        assert!(
            matches!(refused, Err(TextureError::FixedSize)),
            "{refused:?}"
        );

        let texture = drawable(&renderer, [4, 4]).await;
        let refused = texture.resize([0, 8]);
        assert!(
            matches!(refused, Err(TextureError::InvalidInput(_))),
            "{refused:?}"
        );
        assert_eq!(size_of(&texture), [4, 4]);
    });
}

// Story: a pass keeps its depth texture across a window resize, drawing with
// it at the new size.
#[test]
fn a_depth_target_follows_its_texture() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let mut target = renderer
            .create_texture_target([16, 16])
            .await
            .expect("create_texture_target failed");
        let depth = renderer
            .create_depth_texture([16, 16])
            .await
            .expect("create_depth_texture failed");
        let mesh = Mesh::new();
        let corners = [[-1.0, -1.0, 0.0], [3.0, -1.0, 0.0], [-1.0, 3.0, 0.0]];
        mesh.add_vertices(
            corners.map(|corner| Vertex::pbr(corner).set(Vertex::NORMAL, [0.0, 0.0, 1.0])),
        );
        let glow = Material::pbr()
            .base_color([0.0, 0.0, 0.0, 1.0])
            .emissive([1.0, 0.0, 0.0]);
        let pass = Pass::new("scene");
        pass.add(&Model::new(mesh, glow)).expect("add failed");
        pass.set_depth_target(&depth)
            .expect("set_depth_target failed");
        renderer.render(&pass, &target).expect("render failed");

        target.resize([32, 8]).expect("resize failed");
        depth.resize([32, 8]).expect("resize failed");
        renderer.render(&pass, &target).expect("render failed");
        let image = target.get_image().await.expect("read back");
        assert_eq!(image.len(), 32 * 8 * 4);
        assert!(
            image[0] > 200,
            "the pass drew at the new size, got {:?}",
            &image[..4]
        );
    });
}
