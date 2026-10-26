#![cfg(not(target_arch = "wasm32"))]

//! The checks `render` runs on the CPU before it records a pass. Each one
//! returns a typed error, where the frame failed with a GPU error before.

use fragmentcolor::{
    AlphaMode, Material, Mesh, Model, Pass, PassError, Renderer, RendererError, Shader, Target,
    TextureData, TextureFormat, TextureInput, TextureOptions, TextureTarget, TextureUsage, Vertex,
};

/// A fullscreen triangle that samples `source`.
const SAMPLE: &str = r#"
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var soft: sampler;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return textureSample(source, soft, position.xy / 16.0);
}
"#;

/// A fullscreen triangle that reads the depth texture `depth`.
const READ_DEPTH: &str = r#"
@group(0) @binding(0) var depth: texture_depth_2d;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let value = textureLoad(depth, vec2<i32>(position.xy), 0);
    return vec4<f32>(value, 0.0, 0.0, 1.0);
}
"#;

/// A compute shader that reads `source` and writes `destination`.
const COPY: &str = r#"
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var destination: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let texel = textureLoad(source, vec2<i32>(id.xy), 0);
    textureStore(destination, vec2<i32>(id.xy), texel);
}
"#;

const COMPUTE: &str = "@compute @workgroup_size(1)\nfn cs_main() {}\n";

async fn target(renderer: &Renderer, side: u32) -> TextureTarget {
    renderer
        .create_texture_target([side, side])
        .await
        .expect("create_texture_target failed")
}

/// A `side` x `side` texture a pass can draw into, of `format`.
async fn drawable(renderer: &Renderer, side: u32, format: TextureFormat) -> fragmentcolor::Texture {
    let usage = TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING;
    let options = TextureOptions {
        size: Some([side, side].into()),
        format,
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

/// A Model that covers the view, drawn with `material`.
fn cover(material: Material) -> Model {
    let mesh = Mesh::new();
    let corners = [[-1.0, -1.0, 0.0], [3.0, -1.0, 0.0], [-1.0, 3.0, 0.0]];
    mesh.add_vertices(
        corners.map(|corner| Vertex::pbr(corner).set(Vertex::NORMAL, [0.0, 0.0, 1.0])),
    );
    Model::new(mesh, material)
}

fn sampler(texture: &fragmentcolor::Texture) -> Shader {
    let shader = Shader::new(SAMPLE).expect("sample shader");
    shader.set("source", texture).expect("set source");
    shader
}

fn is_conflict(result: &Result<(), RendererError>) -> bool {
    matches!(
        result,
        Err(RendererError::PassError(PassError::TextureConflict { .. }))
    )
}

// Story: a depth target of another size than the pass's color attachment is
// refused, for a pass that draws into the frame and one with a target of its own.
#[test]
fn a_depth_target_of_another_size_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let frame = target(&renderer, 64).await;
        let depth = renderer
            .create_depth_texture([32, 32])
            .await
            .expect("depth texture");
        let pass = Pass::from_shader("depth", &Shader::default());
        pass.set_depth_target(&depth).expect("set_depth_target");

        let result = renderer.render(&pass, &frame);
        assert!(
            matches!(
                result,
                Err(RendererError::DepthSizeMismatch {
                    depth: [32, 32],
                    pass: [64, 64],
                })
            ),
            "{result:?}"
        );

        let color = drawable(&renderer, 16, TextureFormat::Rgba).await;
        pass.set_target(&color).expect("set_target");
        let result = renderer.render(&pass, &frame);
        assert!(
            matches!(
                result,
                Err(RendererError::DepthSizeMismatch {
                    depth: [32, 32],
                    pass: [16, 16],
                })
            ),
            "{result:?}"
        );

        let fitting = drawable(&renderer, 32, TextureFormat::Rgba).await;
        pass.set_target(&fitting).expect("set_target");
        renderer
            .render(&pass, &frame)
            .expect("a depth of the same size");
    });
}

// Story: a Model pass with a depth target that misses the frame's size is
// refused, where the frame failed with a GPU error.
#[test]
fn a_model_pass_checks_its_depth_size() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let mut frame = target(&renderer, 32).await;
        let depth = renderer
            .create_depth_texture([32, 32])
            .await
            .expect("depth texture");
        let pass = Pass::new("models");
        pass.add(&cover(Material::pbr())).expect("add");
        pass.set_depth_target(&depth).expect("set_depth_target");
        renderer.render(&pass, &frame).expect("the same size");

        frame.resize([48, 48]).expect("resize");
        let result = renderer.render(&pass, &frame);
        assert!(
            matches!(
                result,
                Err(RendererError::DepthSizeMismatch {
                    depth: [32, 32],
                    pass: [48, 48],
                })
            ),
            "{result:?}"
        );
    });
}

// Story: a depth target that unregister_texture removed, or that another
// Renderer created, returns TextureNotFoundError, where the pass drew without depth.
#[test]
fn a_missing_depth_target_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let frame = target(&renderer, 16).await;
        let pass = Pass::new("models");
        pass.add(&cover(Material::pbr())).expect("add");

        let depth = renderer
            .create_depth_texture([16, 16])
            .await
            .expect("depth texture");
        pass.set_depth_target(&depth).expect("set_depth_target");
        renderer.render(&pass, &frame).expect("a registered depth");
        renderer
            .unregister_texture(*depth.id())
            .expect("unregister_texture");
        let result = renderer.render(&pass, &frame);
        assert!(
            matches!(result, Err(RendererError::TextureNotFoundError(id)) if id == *depth.id()),
            "{result:?}"
        );

        let other = Renderer::new();
        let foreign = other
            .create_depth_texture([16, 16])
            .await
            .expect("depth texture");
        pass.set_depth_target(&foreign).expect("set_depth_target");
        let result = renderer.render(&pass, &frame);
        assert!(
            matches!(result, Err(RendererError::TextureNotFoundError(id)) if id == *foreign.id()),
            "{result:?}"
        );
    });
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

// Story: a dispatch past the device's workgroup limit is refused before the frame,
// where the whole render failed with a GPU error.
#[test]
fn a_dispatch_past_the_limit_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let frame = target(&renderer, 8).await;
        let compute = Shader::new(COMPUTE).expect("compute shader");
        let pass = Pass::from_shader("big", &compute);

        pass.set_compute_dispatch(u32::MAX, 1, 1);
        let result = renderer.render(&pass, &frame);
        assert!(
            matches!(
                &result,
                Err(RendererError::PassError(PassError::DispatchLimit { pass, dispatch, .. }))
                    if pass == "big" && *dispatch == [u32::MAX, 1, 1]
            ),
            "{result:?}"
        );

        pass.set_compute_dispatch(4, 4, 1);
        renderer
            .render(&pass, &frame)
            .expect("a dispatch that fits");
    });
}

// Story: a shader whose output a color target's format can't take is refused
// before the frame, where the pipeline failed to build.
#[test]
fn an_output_the_target_cannot_take_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let frame = target(&renderer, 8).await;
        let integer = drawable(&renderer, 8, TextureFormat::Rgba32Uint).await;
        let pass = Pass::from_shader("integer", &Shader::default());
        pass.set_target(&integer).expect("set_target");

        let result = renderer.render(&pass, &frame);
        assert!(
            matches!(
                &result,
                Err(RendererError::PassError(PassError::InvalidColorTarget(message)))
                    if message.contains("vec4<u32>") && message.contains("vec4<f32>")
            ),
            "{result:?}"
        );

        let float = drawable(&renderer, 8, TextureFormat::Rgba32Float).await;
        pass.set_target(&float).expect("set_target");
        renderer.render(&pass, &frame).expect("a float target");
    });
}

// Story: AlphaMode::Blend into a target the device can't blend is refused before
// the frame, where the pipeline failed to build. The device requests no float32 blending.
#[test]
fn blending_into_a_target_that_cannot_blend_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let frame = target(&renderer, 8).await;
        let float = drawable(&renderer, 8, TextureFormat::Rgba32Float).await;
        let pass = Pass::new("glass");
        pass.add(&cover(Material::pbr().alpha_mode(AlphaMode::Blend)))
            .expect("add");
        pass.set_target(&float).expect("set_target");

        let result = renderer.render(&pass, &frame);
        assert!(
            matches!(
                &result,
                Err(RendererError::PassError(PassError::InvalidColorTarget(message)))
                    if message.contains("can't blend")
            ),
            "{result:?}"
        );

        let half = drawable(&renderer, 8, TextureFormat::Rgba16Float).await;
        pass.set_target(&half).expect("set_target");
        renderer
            .render(&pass, &frame)
            .expect("a half float target blends");
    });
}

// Story: a shader that samples the texture its pass draws into is refused, for the
// frame of a TextureTarget and for a pass target alike, while a later pass reads it.
#[test]
fn sampling_what_the_pass_draws_into_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let frame = target(&renderer, 16).await;

        let feedback = sampler(&frame.texture());
        let result = renderer.render(&feedback, &frame);
        assert!(is_conflict(&result), "{result:?}");

        let canvas = drawable(&renderer, 16, TextureFormat::Rgba).await;
        let draw = Pass::from_shader("draw", &sampler(&canvas));
        draw.set_target(&canvas).expect("set_target");
        let result = renderer.render(&draw, &frame);
        assert!(is_conflict(&result), "{result:?}");
        let text = result
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default();
        assert!(
            text.contains("'source' reads a texture that the pass draws into"),
            "{text}"
        );

        let fill = Pass::from_shader("fill", &Shader::default());
        fill.set_target(&canvas).expect("set_target");
        let show = Pass::from_shader("show", &sampler(&canvas));
        renderer
            .render(&vec![fill, show], &frame)
            .expect("a later pass reads it");
    });
}

// Story: a shader that reads the depth target its pass draws depth into is refused.
#[test]
fn reading_the_depth_target_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let frame = target(&renderer, 16).await;
        let depth = renderer
            .create_depth_texture([16, 16])
            .await
            .expect("depth texture");
        let shader = Shader::new(READ_DEPTH).expect("depth shader");
        shader.set("depth", &depth).expect("set depth");
        let pass = Pass::from_shader("depth", &shader);
        pass.set_depth_target(&depth).expect("set_depth_target");

        let result = renderer.render(&pass, &frame);
        assert!(is_conflict(&result), "{result:?}");
    });
}

// Story: a compute shader that reads and writes one texture in one dispatch is
// refused, while reading one texture and writing another works.
#[test]
fn a_dispatch_that_reads_what_it_writes_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let frame = target(&renderer, 8).await;
        let make = || async {
            let options = TextureOptions {
                size: Some([8, 8].into()),
                format: TextureFormat::Rgba,
                ..Default::default()
            };
            let input = TextureInput {
                data: TextureData::Empty,
                options,
            };
            renderer
                .create_storage_texture(input)
                .await
                .expect("create_storage_texture failed")
        };
        let first = make().await;
        let second = make().await;
        let shader = Shader::new(COPY).expect("copy shader");
        let pass = Pass::from_shader("copy", &shader);
        pass.set_compute_dispatch(8, 8, 1);

        shader.set("source", &first).expect("set source");
        shader.set("destination", &first).expect("set destination");
        let result = renderer.render(&pass, &frame);
        assert!(is_conflict(&result), "{result:?}");

        shader.set("destination", &second).expect("set destination");
        renderer.render(&pass, &frame).expect("two textures");
    });
}
