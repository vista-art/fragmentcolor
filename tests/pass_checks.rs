#![cfg(not(target_arch = "wasm32"))]

//! The checks `render` runs on the CPU before it records a pass. Each one
//! returns a typed error, where the frame failed with a GPU error before.

use fragmentcolor::{Renderer, RendererError, Shader, TextureTarget};

const COMPUTE: &str = "@compute @workgroup_size(1)\nfn cs_main() {}\n";

async fn target(renderer: &Renderer, side: u32) -> TextureTarget {
    renderer
        .create_texture_target([side, side])
        .await
        .expect("create_texture_target failed")
}

// Story: a Renderer refuses a target another Renderer created, before it takes a
// frame, while a render of compute passes alone never touches the target.
#[test]
fn a_target_from_another_renderer_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let other = Renderer::new();
        let foreign = target(&other, 8).await;
        let own = target(&renderer, 8).await;

        let result = renderer.render(&Shader::default(), &foreign);
        assert!(
            matches!(result, Err(RendererError::ForeignTarget)),
            "{result:?}"
        );

        let compute = Shader::new(COMPUTE).expect("compute shader");
        renderer
            .render(&compute, &foreign)
            .expect("compute draws into no target");
        renderer
            .render(&Shader::default(), &own)
            .expect("its own target");
    });
}
