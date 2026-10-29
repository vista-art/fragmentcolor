// Tests that render refuses, before any GPU work, what the device can't run.
use super::layout::build_module;
use super::layout::{plan_layout, texture_binding};
use super::scope_tests::{SpyTarget, Uncaptured, setup};
use super::shader_checks::{TextureFacts, Verdict, check_texture};
use super::*;
use crate::texture::{Registration, Texture, TextureObject};
use crate::{Material, Shader};

const VERTEX: &str = r#"
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}
"#;

/// A render shader: the fullscreen vertex stage, then `rest`.
fn fullscreen(rest: &str) -> Result<Shader, ShaderError> {
    Shader::new(format!("{VERTEX}\n{rest}").as_str())
}

/// Renders `shader` into a spy target and returns the error, or `None` when it renders.
/// Render must acquire no frame, build no pipeline and leave no GPU error uncaptured.
fn refuse(
    renderer: &Renderer,
    context: &RenderContext,
    uncaptured: &Uncaptured,
    shader: &Shader,
) -> Option<RendererError> {
    let target = SpyTarget::new(context, wgpu::TextureUsages::RENDER_ATTACHMENT);
    let error = renderer.render(shader, &target).err()?;
    assert_eq!(
        target.counts.acquired(),
        0,
        "a frame was acquired for {error}"
    );
    assert!(
        context.render_pipelines.is_empty(),
        "a pipeline was built for {error}"
    );
    assert!(
        context.compute_pipelines.is_empty(),
        "a pipeline was built for {error}"
    );
    assert!(uncaptured.lock().is_empty(), "got {:?}", uncaptured.lock());
    Some(error)
}

/// Renders `shader` on a fresh renderer and returns the error, as `refuse` checks it.
fn refuse_alone(shader: &Shader) -> Option<RendererError> {
    pollster::block_on(async move {
        let (renderer, context, uncaptured) = setup().await.ok()?;
        refuse(&renderer, &context, &uncaptured, shader)
    })
}

/// A 4x4 texture of `format` with `usage`, made past the creation helpers.
fn create_texture(
    context: &Arc<RenderContext>,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> Result<Texture, crate::texture::TextureError> {
    let size = wgpu::Extent3d {
        width: 4,
        height: 4,
        depth_or_array_layers: 1,
    };
    let object = TextureObject::new(context, size, format, usage, Default::default())?;
    let registration = Registration::create(context, Arc::new(object));
    Ok(Texture::new(context.clone(), registration))
}

/// Asserts that `error` is the variant `pattern` names and that its text holds `text`.
macro_rules! assert_refused {
    ($error:expr, $pattern:pat, $text:expr) => {{
        let error = $error;
        assert!(matches!(error, $pattern), "got {error:?}");
        assert!(error.to_string().contains($text), "got {error}");
    }};
}

// Story: a group past the device's bind groups fails before the frame, on every platform alike.
#[test]
fn group_past_the_device_fails() {
    let groups = pollster::block_on(async {
        let (_renderer, context, _) = setup().await.expect("render context");
        context.device.limits().max_bind_groups
    });
    let shader = fullscreen(&format!(
        "@group({groups}) @binding(0) var<uniform> tint: vec4<f32>;
@fragment fn fs_main() -> @location(0) vec4<f32> {{ return tint; }}"
    ))
    .expect("shader");
    let error = refuse_alone(&shader).expect("refused");
    assert_refused!(
        error,
        RendererError::LimitError { .. },
        &format!(
            "'tint' at @group({groups}) @binding(0) needs max_bind_groups of at least {}, and the device has {groups}",
            groups + 1
        )
    );
}

// Story: the group that holds immediate roots on a device without native immediates
// takes the lowest free index, so a shader that leaves a gap still fits.
#[test]
fn lowered_immediates_take_the_lowest_free_group() {
    pollster::block_on(async move {
        let (renderer, context, uncaptured) = setup().await.expect("render context");
        let last = context.device.limits().max_bind_groups - 1;
        let target = renderer
            .create_texture_target([4, 4])
            .await
            .expect("texture target");
        // Two immediate roots never take native immediates. An entry point reads one at most.
        let shader = fullscreen(&format!(
            "var<immediate> red: f32;
var<immediate> spare: f32;
@group(0) @binding(0) var<uniform> blue: f32;
@group({last}) @binding(0) var<uniform> alpha: f32;
@fragment fn fs_main() -> @location(0) vec4<f32> {{ return vec4<f32>(red, 0.0, blue, alpha); }}"
        ))
        .expect("shader");
        // A bare immediate root takes its bytes whole.
        for key in ["red", "spare"] {
            shader
                .set(key, 1.0f32.to_le_bytes().as_slice())
                .expect("set");
        }
        shader.set("blue", 1.0).expect("blue");
        shader.set("alpha", 1.0).expect("alpha");

        renderer.render(&shader, &target).expect("render");
        let image = target.get_image().await.expect("read back");

        assert_eq!(image[..4], [255, 0, 255, 255]);
        assert!(uncaptured.lock().is_empty(), "got {:?}", uncaptured.lock());
    });
}

// Story: more bindings of one kind in a stage than the device takes fail before the frame,
// and the same bindings split across two stages fit, since each counts only where it is used.
#[test]
fn stage_counts_follow_the_stages_that_use_each_binding() {
    pollster::block_on(async move {
        let (renderer, context, uncaptured) = setup().await.expect("render context");
        let max = context.device.limits().max_samplers_per_shader_stage;
        let declare = |count: u32| -> String {
            (0..count)
                .map(|index| format!("@group(0) @binding({index}) var s{index}: sampler;\n"))
                .collect()
        };
        let sample = |range: std::ops::Range<u32>| -> String {
            range
                .map(|index| format!("textureSampleLevel(image, s{index}, vec2<f32>(0.5), 0.0)"))
                .collect::<Vec<_>>()
                .join(" + ")
        };
        let image = "@group(1) @binding(0) var image: texture_2d<f32>;";
        let crowded = Shader::new(
            format!(
                "{}{image}
{VERTEX}
@fragment fn fs_main() -> @location(0) vec4<f32> {{ return {}; }}",
                declare(max + 1),
                sample(0..max + 1)
            )
            .as_str(),
        )
        .expect("shader");
        let texture = create_texture(
            &context,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING,
        )
        .expect("texture");
        crowded.set("image", &texture).expect("image");
        let error = refuse(&renderer, &context, &uncaptured, &crowded).expect("refused");
        assert_refused!(
            error,
            RendererError::LimitError { .. },
            &format!(
                "the fragment stage needs max_samplers_per_shader_stage of at least {}, and the device has {max}",
                max + 1
            )
        );

        let half = (max + 1).div_ceil(2);
        let split = Shader::new(
            format!(
                "{}{image}
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {{
  return {} * f32(i);
}}
@fragment fn fs_main() -> @location(0) vec4<f32> {{ return {}; }}",
                declare(max + 1),
                sample(0..half),
                sample(half..max + 1)
            )
            .as_str(),
        )
        .expect("shader");
        split.set("image", &texture).expect("image");
        let target = renderer
            .create_texture_target([4, 4])
            .await
            .expect("texture target");
        renderer
            .render(&split, &target)
            .expect("split samplers fit");
    });
}

// Story: a binding past the device's bindings, a uniform past its size limit and a storage
// buffer past its size limit each fail with an error that names the root and the limit.
#[test]
fn slots_past_the_limits_fail() {
    pollster::block_on(async move {
        let (_renderer, context, _) = setup().await.expect("render context");
        // The WebGPU defaults, which the web's devices give.
        let limits = wgpu::Limits::default();
        let check = |source: String| {
            let shader = fullscreen(&source).expect("shader");
            let layout = plan_layout(&shader.object, &context, None);
            layout
                .slots
                .iter()
                .find_map(|slot| {
                    super::shader_checks::check_slot(&format!("'{}'", slot.name), slot, &limits)
                        .err()
                        .map(RendererError::from)
                })
                .expect("a slot past the limits")
                .to_string()
        };
        let binding = check(
            "@group(0) @binding(1000) var<uniform> tint: vec4<f32>;
@fragment fn fs_main() -> @location(0) vec4<f32> { return tint; }"
                .into(),
        );
        assert!(
            binding.contains(
                "'tint' needs max_bindings_per_bind_group of at least 1001, and the device has 1000"
            ),
            "{binding}"
        );
        let uniform = check(
            "@group(0) @binding(0) var<uniform> big: array<vec4<f32>, 8192>;
@fragment fn fs_main() -> @location(0) vec4<f32> { return big[0]; }"
                .into(),
        );
        assert!(
            uniform.contains(
                "'big' needs max_uniform_buffer_binding_size of at least 131072, and the device has 65536"
            ),
            "{uniform}"
        );
        let storage = check(
            "@group(0) @binding(0) var<storage, read> big: array<vec4<f32>, 16777216>;
@fragment fn fs_main() -> @location(0) vec4<f32> { return big[0]; }"
                .into(),
        );
        assert!(
            storage.contains(
                "'big' needs max_storage_buffer_binding_size of at least 268435456, and the device has 134217728"
            ),
            "{storage}"
        );
    });
}

// Story: a storage texture binds in the format the shader declares,
// including the formats outside the old table, such as r32float.
#[test]
fn storage_textures_bind_their_declared_format() {
    pollster::block_on(async move {
        let (renderer, context, uncaptured) = setup().await.expect("render context");
        let shader = Shader::new(
            r#"
@group(0) @binding(0) var image: texture_storage_2d<r32float, write>;
@compute @workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
  textureStore(image, id.xy, vec4<f32>(0.5));
}
"#,
        )
        .expect("shader");
        let usage = wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC;
        let texture =
            create_texture(&context, wgpu::TextureFormat::R32Float, usage).expect("texture");
        shader.set("image", &texture).expect("image");
        let pass = crate::Pass::from_shader("store", &shader);
        pass.set_compute_dispatch(4, 4, 1);
        let target = renderer
            .create_texture_target([4, 4])
            .await
            .expect("texture target");

        renderer.render(&pass, &target).expect("render");
        let bytes = renderer
            .read_texture(*texture.id())
            .await
            .expect("read back");

        assert_eq!(bytes[..4], 0.5f32.to_le_bytes());
        assert!(uncaptured.lock().is_empty(), "got {:?}", uncaptured.lock());
    });
}

// Story: a fragment entry point writes a storage texture, and the pass draws.
#[test]
fn fragment_writes_a_storage_texture() {
    pollster::block_on(async move {
        let (renderer, context, uncaptured) = setup().await.expect("render context");
        let shader = fullscreen(
            "@group(0) @binding(0) var marks: texture_storage_2d<rgba8unorm, write>;
@fragment fn fs_main(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
  textureStore(marks, vec2<u32>(at.xy), vec4<f32>(1.0, 0.0, 0.0, 1.0));
  return vec4<f32>(1.0);
}",
        )
        .expect("shader");
        let usage = wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC;
        let marks =
            create_texture(&context, wgpu::TextureFormat::Rgba8Unorm, usage).expect("texture");
        shader.set("marks", &marks).expect("marks");
        let target = renderer
            .create_texture_target([4, 4])
            .await
            .expect("texture target");

        renderer.render(&shader, &target).expect("render");
        let bytes = renderer.read_texture(*marks.id()).await.expect("read back");

        assert_eq!(bytes[..4], [255, 0, 0, 255]);
        assert!(uncaptured.lock().is_empty(), "got {:?}", uncaptured.lock());
    });
}

// Story: a compute shader that leaves group 0 empty still dispatches,
// with an empty group in the gap as a render pipeline has.
#[test]
fn compute_fills_group_gaps() {
    pollster::block_on(async move {
        let (renderer, _context, uncaptured) = setup().await.expect("render context");
        let shader = Shader::new(
            r#"
@group(1) @binding(0) var<storage, read_write> gap_values: array<f32, 4>;
@compute @workgroup_size(4)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) { gap_values[id.x] = 1.0; }
"#,
        )
        .expect("shader");
        let target = renderer
            .create_texture_target([4, 4])
            .await
            .expect("texture target");

        renderer.render(&shader, &target).expect("render");
        let bytes = renderer
            .read_storage(&shader, "gap_values")
            .await
            .expect("read back");

        assert_eq!(bytes, [1.0f32; 4].map(f32::to_le_bytes).concat());
        assert!(uncaptured.lock().is_empty(), "got {:?}", uncaptured.lock());
    });
}

// Story: a compute shader reads its immediate data, natively for one root that fits
// and through the uniform group when it declares two roots.
#[test]
fn compute_reads_immediates() {
    pollster::block_on(async move {
        let (renderer, _context, uncaptured) = setup().await.expect("render context");
        let target = renderer
            .create_texture_target([4, 4])
            .await
            .expect("texture target");
        let one = Shader::new(
            r#"
var<immediate> params: vec4<f32>;
@group(0) @binding(0) var<storage, read_write> one_root: vec4<f32>;
@compute @workgroup_size(1) fn cs_main() { one_root = params; }
"#,
        )
        .expect("shader");
        let two = Shader::new(
            r#"
var<immediate> spare: vec2<f32>;
var<immediate> params: vec4<f32>;
@group(0) @binding(0) var<storage, read_write> two_roots: vec4<f32>;
@compute @workgroup_size(1) fn cs_main() { two_roots = params; }
"#,
        )
        .expect("shader");
        // A bare immediate root takes its bytes whole.
        let expected = [1.0f32, 2.0, 3.0, 4.0].map(f32::to_le_bytes).concat();
        one.set("params", expected.as_slice()).expect("params");
        two.set("spare", [0u8; 8].as_slice()).expect("spare");
        two.set("params", expected.as_slice()).expect("params");

        for (shader, root) in [(&one, "one_root"), (&two, "two_roots")] {
            renderer.render(shader, &target).expect("render");
            let bytes = renderer
                .read_storage(shader, root)
                .await
                .expect("read back");
            assert_eq!(bytes, expected, "{root}");
        }
        assert!(uncaptured.lock().is_empty(), "got {:?}", uncaptured.lock());
    });
}

// Story: a workgroup past the device's limits fails before any GPU work,
// also when an override sizes it.
#[test]
fn workgroup_past_the_device_fails() {
    let wide = Shader::new("@compute @workgroup_size(4096) fn cs_main() {}").expect("shader");
    assert_refused!(
        refuse_alone(&wide).expect("refused"),
        RendererError::LimitError { .. },
        "`cs_main` needs max_compute_workgroup_size_x of at least 4096, and the device has"
    );
    let crowded = Shader::new(
        "override side: u32 = 64;
@compute @workgroup_size(side, side) fn cs_main() {}",
    )
    .expect("shader");
    assert_refused!(
        refuse_alone(&crowded).expect("refused"),
        RendererError::LimitError { .. },
        "`cs_main` needs max_compute_invocations_per_workgroup of at least 4096, and the device has"
    );
}

// Story: a render shader without a vertex or a fragment entry point fails before any GPU work.
#[test]
fn missing_entry_points_fail() {
    let fragment =
        Shader::new("@fragment fn main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }")
            .expect("a lone fragment stage parses");
    assert_refused!(
        refuse_alone(&fragment).expect("refused"),
        RendererError::EntryError { .. },
        "the shader has no @vertex entry point"
    );
    let vertex = Shader::new(VERTEX).expect("a lone vertex stage parses");
    assert_refused!(
        refuse_alone(&vertex).expect("refused"),
        RendererError::EntryError { .. },
        "the shader has no @fragment entry point"
    );
}

// Story: every texture a slot can't take fails before the frame with an error naming the
// slot and the mismatch, the same on the web, where no error scope resolves in time.
#[test]
fn textures_their_slots_cannot_take_fail() {
    pollster::block_on(async move {
        let (renderer, context, uncaptured) = setup().await.expect("render context");
        let sampled = "@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var linear: sampler;
@fragment fn fs_main() -> @location(0) vec4<f32> { return textureSample(image, linear, vec2<f32>(0.5)); }";
        let storage = "@group(0) @binding(0) var image: texture_storage_2d<rgba8unorm, write>;
@fragment fn fs_main() -> @location(0) vec4<f32> {
  textureStore(image, vec2<u32>(0u), vec4<f32>(1.0));
  return vec4<f32>(1.0);
}";
        let depth = renderer
            .create_depth_texture([4, 4])
            .await
            .expect("depth texture");
        let volume = renderer
            .create_texture((vec![0u8; 4 * 4 * 4 * 4], [4u32, 4, 4]))
            .await
            .expect("3D texture");
        let plain = renderer
            .create_texture((vec![0u8; 4 * 4 * 4], [4u32, 4]))
            .await
            .expect("2D texture");
        let cases: [(&str, &Texture, &str); 3] = [
            (
                sampled,
                &depth,
                "'image' takes a texture that samples as Float { filterable: true }, not Depth32Float, which samples as Depth",
            ),
            (
                sampled,
                &volume,
                "'image' takes a D2 texture, not a D3 texture",
            ),
            (
                storage,
                &plain,
                "'image' takes a texture with STORAGE_BINDING usage, not usage",
            ),
        ];
        for (source, texture, problem) in cases {
            let shader = fullscreen(source).expect("shader");
            shader.set("image", texture).expect("set");
            let error = refuse(&renderer, &context, &uncaptured, &shader).expect("refused");
            assert_refused!(error, RendererError::BindingError { .. }, problem);
        }

        let unset = fullscreen(sampled).expect("shader");
        assert_refused!(
            refuse(&renderer, &context, &uncaptured, &unset).expect("refused"),
            RendererError::BindingError { .. },
            "'image' takes a texture, not none, since it was never set"
        );
        let removed = fullscreen(sampled).expect("shader");
        removed.set("image", &plain).expect("set");
        renderer
            .unregister_texture(*plain.id())
            .expect("unregister");
        assert_refused!(
            refuse(&renderer, &context, &uncaptured, &removed).expect("refused"),
            RendererError::TextureNotFoundError(_),
            "not found"
        );
    });
}

// Story: a Material map given a texture its slot can't take fails at render, the same
// on every platform, instead of drawing a broken frame on the web.
#[test]
fn material_maps_check_their_textures() {
    pollster::block_on(async move {
        let (renderer, context, uncaptured) = setup().await.expect("render context");
        let depth = renderer
            .create_depth_texture([4, 4])
            .await
            .expect("depth texture");
        let material = Material::pbr().base_color_texture(&depth);

        let error = refuse(&renderer, &context, &uncaptured, material.shader()).expect("refused");

        assert_refused!(
            error,
            RendererError::BindingError { .. },
            "'base_color_map' takes a texture that samples as Float { filterable: true }, not Depth32Float"
        );
    });
}

// Story: the bind-time texture check follows wgpu's rules: an unfilterable float slot takes
// filterable floats and depth, and a multisampled slot takes only a multisampled texture.
#[test]
fn texture_checks_follow_wgpu_rules() {
    pollster::block_on(async move {
        let (_renderer, context, _) = setup().await.expect("render context");
        let usage = wgpu::TextureUsages::TEXTURE_BINDING;
        let color =
            create_texture(&context, wgpu::TextureFormat::Rgba8Unorm, usage).expect("texture");
        let meta = |sampled: bool| crate::texture::TextureMeta {
            sampled,
            ..crate::texture::TextureMeta::with_id_only(*color.id())
        };
        let features = context.device.features();
        let facts = TextureFacts::read(&color.object());
        let loaded = texture_binding(&meta(false));
        assert!(check_texture("t", &loaded, &facts, features).is_ok());
        let filtered = texture_binding(&meta(true));
        assert!(check_texture("t", &filtered, &facts, features).is_ok());
        let multi = wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: true,
        };
        let error = check_texture("t", &multi, &facts, features).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("'t' takes a multisampled texture, not 1 samples"),
            "{error}"
        );
    });
}

/// A context on the device of `context`, with its own verdicts, for a test to set
/// the backend, downlevel flags or padding of a device it can't open.
fn copy_context(context: &RenderContext) -> RenderContext {
    let mut copy = RenderContext::new(context.device.clone(), context.queue.clone());
    copy.adapter = context.adapter.clone();
    copy.downlevel = context.downlevel;
    copy.backend = context.backend;
    copy
}

// Story: the GL backends write a buffer struct member by member into a std140 block,
// so they refuse an explicit @size or @align layout instead of shifting its members.
// The refusal names the shader's own struct, also where WebGL2 pads a copy of it.
#[test]
fn gl_refuses_explicit_layouts() {
    pollster::block_on(async move {
        let (_renderer, context, _) = setup().await.expect("render context");
        let shader = fullscreen(
            "struct Params { scale: f32, @align(16) offset: vec2<f32> };
@group(0) @binding(0) var<uniform> params: Params;
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(params.offset, params.scale, 1.0); }",
        )
        .expect("shader");
        let short = fullscreen(
            "struct P { @size(8) a: f32, b: f32 };
@group(0) @binding(0) var<uniform> p: P;
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(p.a, p.b, 0.0, 1.0); }",
        )
        .expect("shader");
        let mut gl = copy_context(&context);
        gl.backend = wgpu::Backend::Gl;
        let mut padded = copy_context(&context);
        padded.backend = wgpu::Backend::Gl;
        padded
            .aligned_uniform_bindings
            .store(true, Ordering::Relaxed);

        let error = gl.check_shader(&shader.object).unwrap_err();
        let short_error = padded.check_shader(&short.object).unwrap_err();

        assert_refused!(
            error,
            RendererError::FeatureError { .. },
            "struct `Params` (spell its padding out as members instead) needs @size and @align offsets in a std140 block, which the device lacks"
        );
        assert_refused!(
            short_error,
            RendererError::FeatureError { .. },
            "struct `P` (spell its padding out"
        );
        assert!(copy_context(&context).check_shader(&shader.object).is_ok());
    });
}

// Story: a device that takes lights through a uniform block refuses a shader that
// measures a light root with `arrayLength`, and keeps that verdict like any other.
#[test]
fn uniform_lights_refuse_measured_roots() {
    pollster::block_on(async move {
        let (_renderer, context, _) = setup().await.expect("render context");
        let shader = fullscreen(
            "struct Light { kind: u32, intensity: f32, range: f32, inner_cone_cos: f32, position: vec3<f32>, outer_cone_cos: f32, direction: vec3<f32>, color: vec3<f32> }
struct LightGrid { view_proj: mat4x4<f32>, depth_plane: vec4<f32>, size: vec3<u32>, slicing: u32, slice_scale: f32, slice_bias: f32, reach: f32 }
struct Lights { count: u32, global_count: u32, promoted_count: u32, ambient: vec3<f32>, grid: LightGrid, lights: array<Light> }
@group(0) @binding(0) var<storage, read> lights: Lights;
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(f32(arrayLength(&lights.lights))); }",
        )
        .expect("shader");
        let uniform = copy_context(&context);
        *uniform.light_path.write() = LightPath::Uniform { count: 4 };

        let first = uniform.check_shader(&shader.object).unwrap_err();
        let second = uniform.check_shader(&shader.object).unwrap_err();

        assert_eq!(first.to_string(), second.to_string());
        assert_refused!(
            first,
            RendererError::FeatureError { .. },
            "a shader that measures `lights` or `light_grid` with `arrayLength` (loop to `lights.count` instead) needs clustered lights in storage buffers"
        );
        let verdict = uniform.checked.get(&shader.object.hash).expect("a verdict");
        assert!(
            matches!(verdict.value(), Verdict::Refused(_)),
            "got {:?}",
            verdict.value()
        );
    });
}

// Story: a binding no entry point uses is visible to the compute stage only, so a render
// pipeline with an unused storage buffer builds on a device without vertex storage.
#[test]
fn unused_storage_needs_no_vertex_storage() {
    pollster::block_on(async move {
        let (_renderer, context, _) = setup().await.expect("render context");
        let mut plain = copy_context(&context);
        plain.downlevel = context.downlevel - wgpu::DownlevelFlags::VERTEX_STORAGE;
        let unused = fullscreen(
            "@group(0) @binding(0) var<storage, read> spare: array<vec4<f32>, 4>;
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
        )
        .expect("shader");
        let read = Shader::new(
            "@group(0) @binding(0) var<storage, read> spare: array<vec4<f32>, 4>;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return spare[0]; }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
        )
        .expect("shader");

        assert!(plain.check_shader(&unused.object).is_ok());
        assert_refused!(
            plain.check_shader(&read.object).unwrap_err(),
            RendererError::FeatureError { .. },
            "'spare' at @group(0) @binding(0) needs VERTEX_STORAGE"
        );
    });
}

// Story: on a backend that pads uniform bindings to 16 bytes, such as WebGL2, an immediate
// root lowered to a uniform is padded too, so a 12-byte root binds as 16 bytes.
#[test]
fn lowered_immediates_fill_sixteen_bytes() {
    pollster::block_on(async move {
        let (_renderer, context, _) = setup().await.expect("render context");
        let padded = copy_context(&context);
        padded
            .aligned_uniform_bindings
            .store(true, Ordering::Relaxed);
        let shader = fullscreen(
            "var<immediate> tint: vec3<f32>;
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(tint, 1.0); }",
        )
        .expect("shader");

        let layout = plan_layout(&shader.object, &padded, None);
        let module = build_module(&shader.object, &layout, true, None).expect("module");

        let mut layouter = naga::proc::Layouter::default();
        layouter.update(module.to_ctx()).expect("layout");
        let tint = module
            .global_variables
            .iter()
            .find(|(_, global)| global.name.as_deref() == Some("tint"))
            .map(|(_, global)| global)
            .expect("tint");
        assert_eq!(tint.space, naga::AddressSpace::Uniform);
        assert_eq!(
            layouter[tint.ty].size % 16,
            0,
            "binds a multiple of 16 bytes"
        );
        assert!(padded.check_shader(&shader.object).is_ok());
    });
}

// Story: more workgroup memory than the device holds fails before any GPU work,
// where the browser refused the pipeline after render returned.
#[test]
fn workgroup_memory_past_the_device_fails() {
    let max = pollster::block_on(async {
        let (_renderer, context, _) = setup().await.expect("render context");
        context.limits.max_compute_workgroup_storage_size
    });
    let count = max / 16 + 1;
    let shader = Shader::new(
        format!(
            "var<workgroup> tile: array<vec4<f32>, {count}>;
@compute @workgroup_size(1) fn cs_main() {{ tile[0] = vec4<f32>(1.0); }}"
        )
        .as_str(),
    )
    .expect("shader");
    assert_refused!(
        refuse_alone(&shader).expect("refused"),
        RendererError::LimitError { .. },
        &format!(
            "`cs_main` needs max_compute_workgroup_storage_size of at least {}, and the device has {max}",
            count * 16
        )
    );
}

// Story: a refused shader keeps its verdict, so each later frame returns the same
// error without building and validating its module again.
#[test]
fn refused_shaders_keep_their_verdict() {
    pollster::block_on(async move {
        let (renderer, context, uncaptured) = setup().await.expect("render context");
        let lone =
            Shader::new("@fragment fn main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }")
                .expect("shader");

        let first = refuse(&renderer, &context, &uncaptured, &lone).expect("refused");
        let second = refuse(&renderer, &context, &uncaptured, &lone).expect("refused");

        assert_eq!(first.to_string(), second.to_string());
        let verdict = context.checked.get(&lone.object.hash).expect("a verdict");
        assert!(
            matches!(verdict.value(), Verdict::Refused(_)),
            "got {:?}",
            verdict.value()
        );
    });
}

// Story: an unset PBR map takes its 1x1 default only where the slot takes a 2D float
// texture, and a compute shader, which binds no default, needs its texture set.
#[test]
fn pbr_defaults_fit_their_slots() {
    pollster::block_on(async move {
        let (renderer, context, uncaptured) = setup().await.expect("render context");
        let layered = fullscreen(
            "@group(0) @binding(0) var normal_map: texture_2d_array<f32>;
@fragment fn fs_main() -> @location(0) vec4<f32> { return textureLoad(normal_map, vec2<i32>(0), 0, 0); }",
        )
        .expect("shader");
        assert_refused!(
            refuse(&renderer, &context, &uncaptured, &layered).expect("refused"),
            RendererError::BindingError { .. },
            "'normal_map' takes a D2Array view, which no FragmentColor texture makes yet, not a D2 texture"
        );
        let compute = Shader::new(
            "@group(0) @binding(0) var base_color_map: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> pbr_out: vec4<f32>;
@compute @workgroup_size(1) fn cs_main() { pbr_out = textureLoad(base_color_map, vec2<i32>(0), 0); }",
        )
        .expect("shader");
        assert_refused!(
            refuse(&renderer, &context, &uncaptured, &compute).expect("refused"),
            RendererError::BindingError { .. },
            "'base_color_map' takes a texture, not none, since it was never set"
        );
    });
}

// Story: a cube slot takes no FragmentColor texture yet, and says so.
#[test]
fn cube_slots_take_no_texture_yet() {
    pollster::block_on(async move {
        let (renderer, context, uncaptured) = setup().await.expect("render context");
        let sky = fullscreen(
            "@group(0) @binding(0) var sky: texture_cube<f32>;
@group(0) @binding(1) var linear: sampler;
@fragment fn fs_main() -> @location(0) vec4<f32> { return textureSample(sky, linear, vec3<f32>(1.0, 0.0, 0.0)); }",
        )
        .expect("shader");
        let plain = renderer
            .create_texture((vec![0u8; 4 * 4 * 4], [4u32, 4]))
            .await
            .expect("2D texture");
        sky.set("sky", &plain).expect("set");

        assert_refused!(
            refuse(&renderer, &context, &uncaptured, &sky).expect("refused"),
            RendererError::BindingError { .. },
            "'sky' takes a Cube view, which no FragmentColor texture makes yet"
        );
    });
}
