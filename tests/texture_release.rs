#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::renderer::error::RendererError;
use fragmentcolor::texture::TextureId;
use fragmentcolor::{Pass, Renderer, Shader, Texture, TextureFormat};

const SAMPLED: &str = r#"
@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var image_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32(index & 1u), f32(index >> 1u));
    return vec4<f32>(corner * 4.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return textureSample(image, image_sampler, position.xy / 4.0);
}
"#;

async fn create(renderer: &Renderer) -> Texture {
    renderer
        .create_storage_texture(([4, 4], TextureFormat::Rgba))
        .await
        .expect("create_storage_texture failed")
}

async fn registered(renderer: &Renderer, id: TextureId) -> bool {
    !matches!(
        renderer.read_texture(id).await,
        Err(RendererError::TextureNotFoundError(_))
    )
}

// Story: the renderer keeps a texture while a handle, a shader uniform or a
// pass target holds it, and lets it go when the last of them drops.
#[test]
fn textures_leave_the_registry_with_their_last_holder() {
    pollster::block_on(async move {
        let renderer = Renderer::new();

        // A dropped handle releases its texture, and a clone keeps it.
        let texture = create(&renderer).await;
        let id = *texture.id();
        let copy = texture.clone();
        drop(texture);
        assert!(registered(&renderer, id).await, "a clone keeps the texture");
        drop(copy);
        assert!(
            !registered(&renderer, id).await,
            "the last handle releases it"
        );

        // A shader uniform keeps its texture until another one takes the slot.
        let shader = Shader::new(SAMPLED).expect("Shader::new failed");
        let first = create(&renderer).await;
        let first_id = *first.id();
        shader.set("image", &first).expect("set failed");
        drop(first);
        assert!(
            registered(&renderer, first_id).await,
            "a bound texture stays"
        );
        let second = create(&renderer).await;
        let second_id = *second.id();
        shader.set("image", &second).expect("set failed");
        drop(second);
        assert!(
            !registered(&renderer, first_id).await,
            "an unbound texture goes"
        );

        // The shader still draws the texture it holds.
        let target = renderer
            .create_texture_target([4, 4])
            .await
            .expect("create_texture_target failed");
        renderer.render(&shader, &target).expect("render failed");
        drop(shader);
        assert!(
            !registered(&renderer, second_id).await,
            "the shader let it go"
        );

        // A texture target holds its texture, and so does a pass drawing into it.
        let id = *target.texture().id();
        assert!(
            registered(&renderer, id).await,
            "the target holds its texture"
        );
        let pass = Pass::new("into the target");
        pass.set_target(&target).expect("set_target failed");
        drop(target);
        assert!(registered(&renderer, id).await, "the pass holds its target");
        drop(pass);
        assert!(!registered(&renderer, id).await, "the pass let it go");
    });
}
