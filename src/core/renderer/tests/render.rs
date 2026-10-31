//! Renders that draw, read back and fail:
//! textures, samplers, vertex inputs, depth, MSAA and pools.

use crate::TextureOptions;
use crate::core::renderer::*;
use std::thread;
use std::time::Duration;

use crate::targets::TargetFrame;
use crate::{Pass, Shader};

// Story: Creating a texture from RGBA8 raw bytes with explicit size succeeds and yields expected size.
#[test]
fn create_texture_with_raw_bytes_success() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        // 2x2 RGBA (4 pixels): solid colors
        #[rustfmt::skip]
        let bytes: [u8; 16] = [
            255, 0, 0, 255,   0, 255, 0, 255,
            0, 0, 255, 255,   255, 255, 255, 255,
        ];
        let tex = renderer
            .create_texture((&bytes[..], crate::Size::from((2u32, 2u32))))
            .await
            .expect("texture raw bytes");
        let sz = tex.size();
        assert_eq!([sz.width, sz.height], [2, 2]);
    });
}

// Story: TextureOptions.usage adds COPY_SRC, so read_texture returns the uploaded pixels.
#[test]
fn create_texture_honors_usage_for_readback() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        #[rustfmt::skip]
        let bytes: Vec<u8> = vec![
            255, 0, 0, 255,   0, 255, 0, 255,
            0, 0, 255, 255,   255, 255, 255, 255,
        ];
        let options = TextureOptions {
            size: Some([2u32, 2].into()),
            format: crate::TextureFormat::Rgba8Unorm,
            mipmaps: false,
            ..Default::default()
        }
        .with_usage(crate::TextureUsage::COPY_SRC);
        let texture = renderer
            .create_texture((bytes.clone(), options))
            .await
            .expect("texture with COPY_SRC");
        let read = renderer.read_texture(texture.id()).await.expect("readback");
        assert_eq!(read, bytes);

        let plain = renderer
            .create_texture((bytes.clone(), [2u32, 2]))
            .await
            .expect("texture without COPY_SRC");
        assert!(renderer.read_texture(*plain.id()).await.is_err());
    });
}

// Story: Creating a texture from invalid raw bytes with explicit format/size returns an error.
#[test]
fn create_texture_with_invalid_raw_bytes_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let bad = [1u8, 2, 3];
        let res = renderer
            .create_texture((
                &bad[..],
                TextureOptions::from(crate::Size::from((2u32, 2u32))),
            ))
            .await;
        assert!(res.is_err(), "expected error for insufficient raw bytes");
    });
}

// Story: Creating a texture from a non-existent file path returns an error wrapped by InitializationError.
#[test]
fn create_texture_from_nonexistent_path_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let p = std::path::PathBuf::from("/path/does/not/exist.png");
        let res = renderer
            .create_texture((&p, TextureOptions::from(crate::Size::from((1u32, 1u32)))))
            .await;
        assert!(res.is_err());
    });
}

// Story: validate() surfaces wgpu validation errors as
// RendererError::DeviceError instead of leaking through the device's
// uncaptured-error handler. Repro: a layout that requires one uniform
// buffer entry, paired with a descriptor that supplies zero entries.
#[test]
fn validate_surfaces_wgpu_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let context = renderer.context(None).await.expect("render context");
        let layout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("test layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let result = context.validate("intentionally invalid bind group", || {
            Ok(context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &layout,
                    entries: &[],
                    label: Some("intentionally invalid bind group"),
                }))
        });

        match result {
            Err(RendererError::DeviceError {
                kind,
                label,
                detail,
            }) => {
                assert_eq!(kind, ErrorKind::Validation);
                assert_eq!(label, "intentionally invalid bind group");
                assert!(
                    !detail.is_empty(),
                    "expected non-empty validation message, got empty"
                );
            }
            Err(other) => panic!(
                "expected DeviceError, got different RendererError: {:?}",
                other
            ),
            Ok(_) => {
                panic!("expected DeviceError for layout/entries mismatch, got Ok bind group")
            }
        }
    });
}

struct TestFrame {
    view: wgpu::TextureView,
    format: wgpu::TextureFormat,
    size: wgpu::Extent3d,
}

impl TargetFrame for TestFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
    fn format(&self) -> wgpu::TextureFormat {
        self.format
    }
    fn size(&self) -> wgpu::Extent3d {
        self.size
    }
    fn present(self: Box<Self>, _queue: &wgpu::Queue) {}
    fn auto_present(&self) -> bool {
        false
    }
}

/// A target that never has a frame, like a minimized window.
struct FramelessTarget;

impl crate::targets::FrameSource for FramelessTarget {
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, crate::SurfaceError> {
        Err(crate::SurfaceError::Occluded)
    }

    fn owner(&self) -> Option<Arc<RenderContext>> {
        None
    }
}

impl crate::Target for FramelessTarget {
    fn size(&self) -> crate::Size {
        crate::Size::from((1u32, 1u32))
    }
    fn resize(
        &mut self,
        _size: impl Into<crate::Size>,
    ) -> Result<(), crate::texture::TextureError> {
        Ok(())
    }
    async fn get_image(&self) -> Result<Vec<u8>, crate::texture::TextureError> {
        Ok(Vec::new())
    }
}

async fn create_device_and_queue() -> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
    let instance = device::create_instance().await;
    let adapter = device::request_adapter(&instance, None)
        .await
        .expect("adapter");
    let (device, queue) = device::request_device(&adapter).await.expect("device");
    (adapter, device, queue)
}

// E2E: stress set() under contention while rendering to a texture target.
//
// Previously flaky (~1-in-3 locally; same race in CI) — two
// independent failure modes shared distinct root causes:
//
//   * `unexpected last time: 0.25` — `set`'s dual write path
//     (try_write fast path + queue fallback) wasn't ordered against
//     `flush_pending`. A queued value from an earlier failed
//     `try_write` could survive in `pending` while later `set` calls
//     direct-wrote `storage`; the next flush then re-applied the
//     stale queued value, clobbering the newer direct write.
//     Fixed by clearing the pending entry for the same key when the
//     direct-write path succeeds (see `ShaderObject::set`).
//
//   * `render: ShaderError(Busy("storage read"))` — the renderer's
//     bind path used `try_read` to acquire the storage lock for
//     uniform/storage-buffer reads, surfacing as `Busy` whenever a
//     concurrent `set` happened to be mid-`try_write`. Reads don't
//     conflict with each other, and `set` releases the write lock
//     in microseconds, so the bounded wait of a blocking `read()`
//     was always the right call. The renderer (and
//     `Shader::get_uniform[_data]`) now use `read()`.
//
// Verified: 50/50 consecutive `cargo test` runs locally. The
// contention pattern is identical to a real client driving
// animation uniforms while the renderer renders.
#[test]
fn e2e_set_stress_during_render_last_wins() {
    pollster::block_on(async move {
        let r = Renderer::new();
        let target = r
            .create_texture_target([64u32, 64u32])
            .await
            .expect("texture target");

        let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@group(0) @binding(0) var<uniform> resolution: vec2<f32>;
@group(0) @binding(1) var<uniform> time: f32;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  var out: VOut;
  out.pos = vec4<f32>(p[i], 0., 1.);
  return out;
}
@fragment fn fs_main(_v: VOut) -> @location(0) vec4<f32> {
  return vec4<f32>(0.2, 0.4, 0.6, 1.0);
}
            "#;
        let shader = crate::Shader::new(wgsl).expect("shader");

        // Writer thread: keep updating time; last-wins
        let shader_writer = shader.clone();
        let writer = thread::spawn(move || {
            for i in 0..1000 {
                let t = i as f32 * 0.001;
                let _ = shader_writer.set("time", t);
                // Small pause to interleave with render
                if i % 50 == 0 {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });

        // Render loop; flushes pending automatically
        for _ in 0..1000u32 {
            // Also set resolution each frame; should never fail
            let _ = shader.set("resolution", [64.0f32, 64.0f32]);
            r.render(&shader, &target).expect("render");
        }

        writer.join().expect("writer thread");

        // Ensure any last enqueued writes (due to contention) are applied
        shader.object.flush_pending();

        // After the loop, we expect the last time value (approx 0.999)
        let last: f32 = shader.get("time").expect("time get");
        assert!(last > 0.95 && last <= 1.2, "unexpected last time: {}", last);
    });
}

#[test]
fn sampler_comparison_binds_and_renders_smoke() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");

        let wgsl = r#"
@group(0) @binding(0) var shadowTex: texture_depth_2d;
@group(0) @binding(1) var shadowSamp: sampler_comparison;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
    var uv = array<vec2<f32>, 3>(vec2<f32>(0.,0.), vec2<f32>(1.,0.), vec2<f32>(0.,1.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    out.uv = uv[i];
    return out;
}
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> {
    let s = textureSampleCompare(shadowTex, shadowSamp, v.uv, 0.5);
    return vec4<f32>(s, s, s, 1.0);
}
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");
        let depth = renderer
            .create_depth_texture([8u32, 8u32])
            .await
            .expect("depth");
        shader.set("shadowTex", &depth).expect("set depth");

        renderer.render(&shader, &target).expect("render ok");
    });
}

#[test]
fn render_with_texture_sampler_smoke() {
    pollster::block_on(async move {
        // Create renderer and a small texture target
        let (_adapter, _device, _queue) = create_device_and_queue().await;
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");

        // Simple WGSL sampling a texture
        let wgsl = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
    var uv = array<vec2<f32>, 3>(vec2<f32>(0.,1.), vec2<f32>(2.,1.), vec2<f32>(0.,-1.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    out.uv = uv[i];
    return out;
}
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, v.uv);
}
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");

        // Create a tiny 2x2 RGBA image and upload as texture
        #[rustfmt::skip]
        let pixels: Vec<u8> = vec![
            255, 0, 0,   255,    0,   255, 0,   255,
            0,   0, 255, 255,    255, 255, 255, 255,
        ];
        let tex = renderer
            .create_texture((&pixels[..], [2u32, 2u32]))
            .await
            .expect("texture");
        shader.set("tex", &tex).expect("set tex");

        // Render should succeed without panicking
        renderer.render(&shader, &target).expect("render ok");

        let image: Vec<u8> = target.get_image().await.expect("read back");

        assert_eq!(image.len(), 8 * 8 * 4);
    });
}

// Story: Bind an R16Unorm texture (prepared mip chain) into a shader and
// render. This is the consumer's failing path (FC-BUG report
// 2026-05-04): without TEXTURE_FORMAT_16BIT_NORM in the device candidates,
// the texture is silently invalid and `create_view` cascades into an
// InvalidResource validation error at every frame. With the fix, this
// round-trips cleanly. textureLoad (no filtering) keeps the layout valid
// even on devices that lack 16-bit-norm filtering.
#[test]
fn render_with_r16unorm_texture_smoke() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");

        let wgsl = r#"
@group(0) @binding(0) var height: texture_2d<f32>;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
    var uv = array<vec2<f32>, 3>(vec2<f32>(0.,1.), vec2<f32>(2.,1.), vec2<f32>(0.,-1.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    out.uv = uv[i];
    return out;
}
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> {
    let h = textureLoad(height, vec2<i32>(0, 0), 0).r;
    return vec4<f32>(h, h, h, 1.0);
}
            "#;
        let shader = crate::Shader::new(wgsl).expect("shader");

        // 4×4 R16Unorm raw data — same shape as a consumer's height tile.
        let words: Vec<u16> = (0..16).map(|i| i * 4096).collect();
        let mut bytes: Vec<u8> = Vec::with_capacity(words.len() * 2);
        for w in &words {
            bytes.extend_from_slice(&w.to_le_bytes());
        }
        let chain = crate::texture::Mipmap::build((
            bytes.as_slice(),
            crate::TextureFormat::R16Unorm,
            [4u32, 4u32],
        ))
        .expect("build R16Unorm chain");
        let tex = renderer
            .create_texture(chain)
            .await
            .expect("create R16Unorm texture from prepared chain");
        shader.set("height", &tex).expect("bind R16Unorm");

        renderer
            .render(&shader, &target)
            .expect("render with R16Unorm texture");
    });
}

// Story: Render a simple mesh with position-only vertices using a minimal shader.
#[test]
fn render_with_mesh_positions_only() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");

        let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");
        let pass = crate::Pass::from_shader("mesh", &shader);

        // Build a triangle mesh
        let mesh = crate::Mesh::new();
        use crate::Vertex;
        mesh.add_vertices([
            Vertex::new([-0.5, -0.5, 0.0]),
            Vertex::new([0.5, -0.5, 0.0]),
            Vertex::new([0.0, 0.5, 0.0]),
        ]);
        pass.add_mesh(&mesh).expect("mesh is compatible");

        renderer.render(&pass, &target).expect("render ok");
    });
}

// Story: Render the same triangle twice using two instances with different offsets.
#[test]
fn render_with_mesh_two_instances() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");

        let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos = vec4<f32>(p, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.,1.,0.,1.); }
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");
        let pass = crate::Pass::from_shader("mesh", &shader);

        let mesh = crate::Mesh::new();
        use crate::Vertex;
        mesh.add_vertices([
            Vertex::new([-0.5, -0.5, 0.0]),
            Vertex::new([0.5, -0.5, 0.0]),
            Vertex::new([0.0, 0.5, 0.0]),
        ]);
        // Two instances with different offsets: provide an "offset" property to match the shader
        use crate::VertexData;
        mesh.add_instance(Vertex::new([0.0, 0.0]).set("offset", VertexData::F32x2([0.0, 0.0])));
        mesh.add_instance(Vertex::new([0.25, 0.0]).set("offset", VertexData::F32x2([0.25, 0.0])));

        pass.add_mesh(&mesh).expect("mesh is compatible");
        renderer.render(&pass, &target).expect("render ok");
    });
}

// Story: Vertex-only mapping with vec2 position and uv attribute on vertices.
#[test]
fn ast_mapping_vertex_pos2_and_uv_smoke() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16u32, 16u32])
            .await
            .expect("texture target");

        let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) uv: vec2<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 0.0, 1.0);
  out.uv = uv;
  return out;
}
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(v.uv, 0.0, 1.0); }
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");
        let pass = crate::Pass::from_shader("p", &shader);

        let mesh = crate::Mesh::new();
        use crate::Vertex;
        mesh.add_vertices([
            Vertex::new([-0.5, -0.5]).set("uv", [0.0, 0.0]),
            Vertex::new([0.5, -0.5]).set("uv", [1.0, 0.0]),
            Vertex::new([0.0, 0.5]).set("uv", [0.5, 1.0]),
        ]);
        pass.add_mesh(&mesh).expect("mesh is compatible");

        renderer.render(&pass, &target).expect("render ok");
    });
}

// Story: Vertex color attribute provided per-vertex; fragment returns color.
#[test]
fn ast_mapping_vertex_color_smoke() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16u32, 16u32])
            .await
            .expect("texture target");

        let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) color: vec4<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 1.0);
  out.color = color;
  return out;
}
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> { return v.color; }
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");
        let pass = crate::Pass::from_shader("p", &shader);

        let mesh = crate::Mesh::new();
        use crate::Vertex;
        mesh.add_vertices([
            Vertex::new([-0.5, -0.5, 0.0]).set("color", [1.0, 0.0, 0.0, 1.0]),
            Vertex::new([0.5, -0.5, 0.0]).set("color", [0.0, 1.0, 0.0, 1.0]),
            Vertex::new([0.0, 0.5, 0.0]).set("color", [0.0, 0.0, 1.0, 1.0]),
        ]);
        pass.add_mesh(&mesh).expect("mesh is compatible");

        renderer.render(&pass, &target).expect("render ok");
    });
}

// Story: When a property name exists in both vertex and instance streams, instance is preferred.
// We write per-instance "tint" and verify left/right instances draw with their tint colors.
#[test]
fn ast_mapping_instance_preferred_over_vertex() {
    fn read_pixel(img: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * w + x) * 4) as usize;
        [img[i], img[i + 1], img[i + 2], img[i + 3]]
    }

    pollster::block_on(async move {
        let renderer = Renderer::new();
        let size = [32u32, 32u32];
        let target = renderer
            .create_texture_target(size)
            .await
            .expect("texture target");

        let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) tint: vec4<f32> };
@vertex
fn vs_main(
  @location(0) pos: vec3<f32>,
  @location(1) offset: vec2<f32>,
  @location(2) tint: vec4<f32>
) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos = vec4<f32>(p, 1.0);
  out.tint = tint;
  return out;
}
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> { return v.tint; }
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");
        let pass = crate::Pass::from_shader("p", &shader);

        let mesh = crate::Mesh::new();
        use crate::{Vertex, VertexData};
        // Triangle geometry in clip-space
        mesh.add_vertices([
            Vertex::new([-0.5, -0.5, 0.0]).set("tint", VertexData::F32x4([0.0, 1.0, 0.0, 1.0])), // vertex-level green (should be ignored)
            Vertex::new([0.5, -0.5, 0.0]).set("tint", VertexData::F32x4([0.0, 1.0, 0.0, 1.0])),
            Vertex::new([0.0, 0.5, 0.0]).set("tint", VertexData::F32x4([0.0, 1.0, 0.0, 1.0])),
        ]);
        // Two instances with different offsets and tints
        mesh.add_instance(
            Vertex::new([-0.6, 0.0])
                .set("offset", VertexData::F32x2([-0.6, 0.0]))
                .set("tint", VertexData::F32x4([1.0, 0.0, 0.0, 1.0])), // red
        );
        mesh.add_instance(
            Vertex::new([0.6, 0.0])
                .set("offset", VertexData::F32x2([0.6, 0.0]))
                .set("tint", VertexData::F32x4([0.0, 0.0, 1.0, 1.0])), // blue
        );

        pass.add_mesh(&mesh).expect("mesh is compatible");
        renderer.render(&pass, &target).expect("render ok");

        let img = target.get_image().await.expect("read back");
        let w = size[0];
        // Helper: map NDC [-1,1] to pixel coordinate [0..w-1]
        let ndc_to_px = |x_ndc: f32, w: u32| -> u32 {
            let fx = (x_ndc * 0.5 + 0.5).clamp(0.0, 1.0);
            (fx * (w as f32 - 1.0)).round() as u32
        };
        let ndc_to_py = |y_ndc: f32, h: u32| -> u32 {
            // NDC +Y is up; pixel Y grows downward. Map accordingly.
            let fy = (-(y_ndc) * 0.5 + 0.5).clamp(0.0, 1.0);
            (fy * (h as f32 - 1.0)).round() as u32
        };
        let left_x = ndc_to_px(-0.6, size[0]);
        let right_x = ndc_to_px(0.6, size[0]);
        let y_mid = ndc_to_py(0.0, size[1]);

        let left = read_pixel(&img, w, left_x, y_mid);
        let right = read_pixel(&img, w, right_x, y_mid);

        // Expect roughly red on the left, blue on the right (alpha 255)
        // get_image() returns RGBA8 bytes
        assert!(
            left[0] > 128 && left[1] < 64 && left[2] < 64 && left[3] == 255,
            "left not red-ish (RGBA): {:?}",
            left
        );
        assert!(
            right[2] > 128 && right[1] < 64 && right[0] < 64 && right[3] == 255,
            "right not blue-ish (RGBA): {:?}",
            right
        );
    });
}

// Story: Shader with non-sequential location indices maps properly (uv at @location(3)).
#[test]
fn ast_mapping_reordered_locations_smoke() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16u32, 16u32])
            .await
            .expect("texture target");

        let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(3) uv: vec2<f32>) -> VOut {
  var out: VOut;
  _ = uv; // not used in fragment
  out.pos = vec4<f32>(pos, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.2,0.3,0.4,1.0); }
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");
        let pass = crate::Pass::from_shader("p", &shader);

        let mesh = crate::Mesh::new();
        use crate::Vertex;
        mesh.add_vertices([
            Vertex::new([-0.5, -0.5, 0.0]).set("uv", [0.0, 0.0]),
            Vertex::new([0.5, -0.5, 0.0]).set("uv", [1.0, 0.0]),
            Vertex::new([0.0, 0.5, 0.0]).set("uv", [0.5, 1.0]),
        ]);
        pass.add_mesh(&mesh).expect("mesh is compatible");

        renderer.render(&pass, &target).expect("render ok");
    });
}

// Story: "position" alias name (vec2) maps to mesh position2.
#[test]
fn ast_mapping_position_alias_position_name_smoke() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16u32, 16u32])
            .await
            .expect("texture target");

        let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(position, 0.0, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.1,0.9,0.1,1.0); }
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");
        let pass = crate::Pass::from_shader("p", &shader);

        let mesh = crate::Mesh::new();
        use crate::Vertex;
        mesh.add_vertices([
            Vertex::new([-0.5, -0.5]),
            Vertex::new([0.5, -0.5]),
            Vertex::new([0.0, 0.5]),
        ]);
        pass.add_mesh(&mesh).expect("mesh is compatible");

        renderer.render(&pass, &target).expect("render ok");
    });
}

fn create_test_frame(
    device: &wgpu::Device,
    size: wgpu::Extent3d,
    format: wgpu::TextureFormat,
) -> TestFrame {
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Test Resolve Target"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        view_formats: &[],
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    // Keep texture alive by capturing it in the view's lifetime (wgpu keeps texture alive internally)
    // Explicit binding not necessary here.
    TestFrame { view, format, size }
}

// Story: with padding forced on, a bare f32 uniform still renders the
// expected color, so the padded module is a drop-in for the original.
#[test]
fn padded_uniform_module_renders() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let context = renderer.context(None).await.expect("context");
        context
            .aligned_uniform_bindings
            .store(true, Ordering::Relaxed);

        let shader = crate::Shader::new(
            r#"
@group(0) @binding(0) var<uniform> k: f32;
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  var p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return VOut(vec4<f32>(p[i], 0.0, 1.0));
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(k, 1.0 - k, 0.0, 1.0); }
"#,
        )
        .expect("shader");
        shader.set("k", 1.0f32).expect("set");
        renderer.render(&shader, &target).expect("render");

        let img = target.get_image().await.expect("read back");
        assert!(
            img[0] > 250 && img[1] < 5 && img[3] == 255,
            "padded uniform should render red, got {:?}",
            &img[..4]
        );
    });
}

// Story: a lease returns its texture to the pool, and a texture no recent
// render used leaves the pool, so a target that changed size frees the old one.
#[test]
fn texture_pool_frees_idle_entries() {
    pollster::block_on(async move {
        let (_adapter, device, queue) = create_device_and_queue().await;
        let ctx = RenderContext::new(device, queue);
        let size = wgpu::Extent3d {
            width: 4,
            height: 4,
            depth_or_array_layers: 1,
        };
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let key = TextureKey::new(size, wgpu::TextureFormat::Rgba8Unorm, 1, usage);
        let pooled = || ctx.texture_pool.read().keys().count();

        let lease = Lease::take(&ctx.texture_pool, &ctx.device, key);
        assert_eq!(pooled(), 0, "a leased texture is out of the pool");
        drop(lease);
        assert_eq!(pooled(), 1, "dropping the lease returns it");

        for _ in 0..8 {
            ctx.texture_pool.write().start_render();
        }
        assert_eq!(pooled(), 1, "a few idle renders keep it");
        ctx.texture_pool.write().start_render();
        assert_eq!(pooled(), 0, "a longer idle stretch frees it");
    });
}

/// A pass that draws one PBR triangle, the kind of pass that gets a default depth.
fn model_pass() -> Arc<PassObject> {
    let mesh = crate::Mesh::new();
    let corners = [[-1.0, -1.0, 0.0], [3.0, -1.0, 0.0], [-1.0, 3.0, 0.0]];
    mesh.add_vertices(
        corners
            .map(|corner| crate::Vertex::pbr(corner).set(crate::Vertex::NORMAL, [0.0, 0.0, 1.0])),
    );
    let pass = Pass::new("models");
    pass.add(&crate::Model::new(mesh, crate::Material::pbr()))
        .expect("add failed");
    pass.passes().first().cloned().expect("pass")
}

/// The sizes of the depth textures waiting in the pool.
fn pooled_depths(ctx: &RenderContext) -> Vec<(u32, u32)> {
    let pool = ctx.texture_pool.read();
    pool.keys()
        .filter(|key| key.format == crate::TextureObject::DEPTH_FORMAT)
        .map(|key| (key.width, key.height))
        .collect()
}

/// Records `pass` into a square frame and submits it, returning any validation error.
async fn draw(ctx: &RenderContext, pass: &PassObject, side: u32) -> Option<wgpu::Error> {
    let size = wgpu::Extent3d {
        width: side,
        height: side,
        depth_or_array_layers: 1,
    };
    let frame = create_test_frame(&ctx.device, size, wgpu::TextureFormat::Rgba8Unorm);
    let scope = ctx.device.push_error_scope(wgpu::ErrorFilter::Validation);
    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    let draws = ctx.plan_pass(pass).expect("the pass plans");
    ctx.process_render_pass(&mut encoder, pass, &frame, &draws)
        .expect("render pass failed");
    ctx.queue.submit(Some(encoder.finish()));
    scope.pop().await
}

// Story: a pass that draws models gets a depth texture from the pool, reuses
// it from frame to frame, and lets the old one go after the frame resizes.
#[test]
fn model_pass_reuses_a_pooled_depth() {
    pollster::block_on(async move {
        let (_adapter, device, queue) = create_device_and_queue().await;
        let ctx = RenderContext::new(device, queue);
        let pass = model_pass();
        for _ in 0..3 {
            ctx.texture_pool.write().start_render();
            assert!(draw(&ctx, &pass, 16).await.is_none(), "invalid pass");
        }
        assert_eq!(pooled_depths(&ctx), [(16, 16)], "one depth, reused");

        for _ in 0..9 {
            ctx.texture_pool.write().start_render();
            assert!(draw(&ctx, &pass, 32).await.is_none(), "invalid pass");
        }
        assert_eq!(pooled_depths(&ctx), [(32, 32)], "the old size left");
    });
}

// Story: a model pass whose viewport clips to nothing still returns its pooled depth.
#[test]
fn an_empty_viewport_returns_the_pooled_depth() {
    pollster::block_on(async move {
        let (_adapter, device, queue) = create_device_and_queue().await;
        let ctx = RenderContext::new(device, queue);
        let pass = model_pass();
        *pass.viewport.write() =
            Some(crate::ScreenRegion::new((100, 100), (4, 4)).expect("a region"));
        assert!(draw(&ctx, &pass, 16).await.is_none(), "invalid pass");
        assert_eq!(pooled_depths(&ctx), [(16, 16)], "the lease went back");
    });
}

// Story: passes without models, and passes with a depth target of their own,
// take no depth from the pool.
#[test]
fn only_model_passes_take_a_pooled_depth() {
    pollster::block_on(async move {
        let (_adapter, device, queue) = create_device_and_queue().await;
        let ctx = Arc::new(RenderContext::new(device, queue));
        let plain = Pass::from_shader("plain", &Shader::default());
        let plain = plain.passes().first().cloned().expect("pass");
        assert!(draw(&ctx, &plain, 8).await.is_none(), "invalid pass");
        assert!(
            pooled_depths(&ctx).is_empty(),
            "a plain pass draws in order"
        );

        let size = wgpu::Extent3d {
            width: 8,
            height: 8,
            depth_or_array_layers: 1,
        };
        let object = Arc::new(crate::TextureObject::create_depth_texture(&ctx, size, 1));
        let depth = crate::texture::Registration::create(&ctx, object);
        let pass = model_pass();
        pass.set_depth_target(depth);
        assert!(draw(&ctx, &pass, 8).await.is_none(), "invalid pass");
        assert!(pooled_depths(&ctx).is_empty(), "its own depth target wins");
    });
}

// Story: the default depth takes the color attachment's sample count, from a
// multisampled frame or from a single-sampled pass target alike.
#[test]
fn default_depth_matches_the_color_attachment() {
    pollster::block_on(async move {
        let (adapter, device, queue) = create_device_and_queue().await;
        let ctx = Arc::new(RenderContext::new(device, queue));
        let supports = |format: wgpu::TextureFormat| {
            let flags = adapter.get_texture_format_features(format).flags;
            flags.sample_count_supported(4)
        };
        if !supports(wgpu::TextureFormat::Rgba8Unorm)
            || !supports(crate::TextureObject::DEPTH_FORMAT)
        {
            return;
        }
        ctx.set_sample_count(4);
        let pass = model_pass();
        assert!(draw(&ctx, &pass, 8).await.is_none(), "multisampled frame");

        let size = crate::Size::from([16, 16]);
        let target = crate::TextureTarget::new(ctx.clone(), size, wgpu::TextureFormat::Rgba8Unorm);
        let pass = Pass::new("into a target");
        let mesh = crate::Mesh::new();
        let corners = [[-1.0, -1.0, 0.0], [3.0, -1.0, 0.0], [-1.0, 3.0, 0.0]];
        mesh.add_vertices(
            corners.map(|corner| {
                crate::Vertex::pbr(corner).set(crate::Vertex::NORMAL, [0.0, 0.0, 1.0])
            }),
        );
        pass.add(&crate::Model::new(mesh, crate::Material::pbr()))
            .expect("add failed");
        pass.set_target(&target).expect("set_target failed");
        let pass = pass.passes().first().cloned().expect("pass");
        assert!(
            draw(&ctx, &pass, 8).await.is_none(),
            "single-sampled target"
        );
    });
}

#[test]
fn sample_count_set_and_get() {
    pollster::block_on(async move {
        let (_adapter, device, queue) = create_device_and_queue().await;
        let ctx = RenderContext::new(device, queue);

        assert_eq!(ctx.sample_count(), 1);
        ctx.set_sample_count(0);
        assert_eq!(ctx.sample_count(), 1);
        ctx.set_sample_count(3);
        assert_eq!(ctx.sample_count(), 3);
        ctx.set_sample_count(4);
        assert_eq!(ctx.sample_count(), 4);
    });
}

#[test]
fn depth_msaa_sample_count_matches() {
    pollster::block_on(async move {
        let (adapter, device, queue) = create_device_and_queue().await;
        let ctx = Arc::new(RenderContext::new(device, queue));

        let color_format = wgpu::TextureFormat::Rgba8Unorm;
        let size = wgpu::Extent3d {
            width: 8,
            height: 8,
            depth_or_array_layers: 1,
        };
        let frame = create_test_frame(&ctx.device, size, color_format);

        // Pick supported MSAA > 1 if available
        let flags = adapter.get_texture_format_features(color_format).flags;
        let sc = if flags.sample_count_supported(4) {
            4
        } else if flags.sample_count_supported(2) {
            2
        } else {
            1
        };

        if sc > 1 {
            ctx.set_sample_count(sc);
            // Create a matching-sample depth texture and register it
            let depth_obj = crate::TextureObject::create_depth_texture(&ctx, size, sc);
            let depth_obj = std::sync::Arc::new(depth_obj);
            let depth = crate::texture::Registration::create(&ctx, depth_obj.clone());

            let shader = Shader::default();
            let pass = Pass::from_shader("msaa-depth", &shader);
            // Hold the passes slice to extend its lifetime, then clone the first Arc<PassObject>
            let first_pass = {
                let passes = pass.passes();
                passes.first().cloned().expect("pass")
            };
            first_pass.set_depth_target(depth);

            let mut encoder = ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            let draws = ctx.plan_pass(&first_pass).expect("the pass plans");
            let res = ctx.process_render_pass(&mut encoder, &first_pass, &frame, &draws);
            assert!(
                res.is_ok(),
                "msaa depth render should succeed: {:?}",
                res.err()
            );
        }
    });
}

// Story: A multisampled pass draws only inside its viewport and resolves the clear around it.
#[test]
fn msaa_pass_draws_inside_the_viewport() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16u32, 16u32])
            .await
            .expect("texture target");
        let supported = renderer
            .adapter
            .read()
            .as_ref()
            .expect("adapter")
            .get_texture_format_features(wgpu::TextureFormat::Rgba8Unorm)
            .flags
            .sample_count_supported(4);
        if !supported {
            return;
        }
        renderer
            .context
            .read()
            .as_ref()
            .expect("context")
            .set_sample_count(4);

        let wgsl = r#"
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
    return vec4<f32>(p[i], 0.0, 1.0);
}
@fragment
fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }
            "#;
        let shader = Shader::new(wgsl).expect("shader");
        let pass = Pass::from_shader("msaa-viewport", &shader);
        pass.set_clear_color([0.0, 0.0, 1.0, 1.0])
            .expect("set_clear_color failed");
        pass.set_viewport(crate::ScreenRegion::new((4, 4), (8, 8)).expect("a region"))
            .expect("set_viewport failed");

        renderer.render(&pass, &target).expect("render ok");

        let image = target.get_image().await.expect("read back");
        let pixel = |x: usize, y: usize| {
            let start = (y * 16 + x) * 4;
            [
                image[start],
                image[start + 1],
                image[start + 2],
                image[start + 3],
            ]
        };
        assert_eq!(pixel(4, 4), [255, 0, 0, 255], "inside, top left");
        assert_eq!(pixel(11, 11), [255, 0, 0, 255], "inside, bottom right");
        assert_eq!(pixel(3, 4), [0, 0, 255, 255], "left of the region");
        assert_eq!(pixel(12, 11), [0, 0, 255, 255], "right of the region");
        assert_eq!(pixel(8, 0), [0, 0, 255, 255], "above the region");
    });
}

#[test]
fn pick_sample_count_properties() {
    pollster::block_on(async move {
        let (adapter, _device, _queue) = create_device_and_queue().await;
        let fmt = wgpu::TextureFormat::Rgba8Unorm;

        // wanted = 0 -> 1
        let s0 = device::pick_sample_count(&adapter, 0, fmt);
        assert_eq!(s0, 1);

        // wanted = 5 -> power of two <= 5 and supported or halved down to supported
        let s5 = device::pick_sample_count(&adapter, 5, fmt);
        assert!((1..=5).contains(&s5));
        assert!(s5.is_power_of_two());
    });
}

// Story: AST-driven mapping errors when a needed attribute is missing.
#[test]
fn ast_mapping_missing_attribute_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");

        let wgsl = r#"
struct VOut { @builtin(position) pos_out: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos_out = vec4<f32>(p, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.,0.,1.,1.); }
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");
        let pass = crate::Pass::from_shader("mesh", &shader);

        let mesh = crate::Mesh::new();
        use crate::Vertex;
        mesh.add_vertices([
            Vertex::new([-0.5, -0.5, 0.0]),
            Vertex::new([0.5, -0.5, 0.0]),
            Vertex::new([0.0, 0.5, 0.0]),
        ]);

        // Expect validation at call site: No instance property named "offset"
        let res = pass.add_mesh(&mesh);
        assert!(res.is_err());
        let s = format!("{}", res.unwrap_err());
        assert!(s.contains("No compatible shader exists for this mesh"));

        // The shader reads vertex inputs, so rendering it without a mesh is an error.
        let res = renderer.render(&pass, &target);
        assert!(
            matches!(&res, Err(RendererError::ShaderError(crate::ShaderError::InvalidKey(message))) if message.contains("@location(0) has no mesh")),
            "got {res:?}"
        );
    });
}

// Story: AST-driven mapping errors on type mismatch between shader input and mesh property format.
#[test]
fn ast_mapping_type_mismatch_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");

        let wgsl = r#"
struct VOut { @builtin(position) pos_out: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos_out = vec4<f32>(p, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,1.,0.,1.); }
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");
        let pass = crate::Pass::from_shader("mesh", &shader);

        let mesh = crate::Mesh::new();
        use crate::{Vertex, VertexData};
        mesh.add_vertices([
            Vertex::new([-0.5, -0.5, 0.0]),
            Vertex::new([0.5, -0.5, 0.0]),
            Vertex::new([0.0, 0.5, 0.0]),
        ]);
        // Add instance with wrong-typed "offset" (vec3 instead of vec2)
        mesh.add_instance(
            Vertex::new([0.0, 0.0]).set("offset", VertexData::F32x3([0.0, 0.0, 0.0])),
        );

        // Expect validation at call site: type mismatch for "offset"
        let res = pass.add_mesh(&mesh);
        assert!(res.is_err());
        let s = format!("{}", res.unwrap_err());
        assert!(s.contains("No compatible shader exists for this mesh"));

        // The shader reads vertex inputs, so rendering it without a mesh is an error.
        let res = renderer.render(&pass, &target);
        assert!(
            matches!(&res, Err(RendererError::ShaderError(crate::ShaderError::InvalidKey(message))) if message.contains("@location(0) has no mesh")),
            "got {res:?}"
        );
    });
}

const STRUCT_SHADER: &str = r#"
struct VIn {
  @builtin(vertex_index) index: u32,
  @location(0) pos: vec2<f32>,
  @location(1) color: vec4<f32>,
};
struct VOut { @builtin(position) position: vec4<f32>, @location(0) color: vec4<f32> };
@vertex fn vs_main(input: VIn, @location(2) offset: vec2<f32>) -> VOut {
  return VOut(vec4<f32>(input.pos + offset, 0.0, 1.0), input.color);
}
@fragment fn fs_main(input: VOut) -> @location(0) vec4<f32> { return input.color; }
"#;

// Story: a shader that reads its vertex inputs through a struct argument takes
// a mesh, and each @location member reads the mesh property of its name.
#[test]
fn struct_vertex_inputs_render() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");
        let shader = crate::Shader::new(STRUCT_SHADER).expect("shader");
        let mesh = crate::Mesh::new();
        let green =
            |corner: [f32; 2]| crate::Vertex::new(corner).set("color", [0.0, 1.0, 0.0, 1.0]);
        mesh.add_vertices(TRIANGLE.map(green));
        mesh.add_instances([crate::Instance::new().set("offset", [0.0, 0.0])]);
        let attached = shader.add_mesh(&mesh);

        // Act
        let rendered = renderer.render(&shader, &target);
        let image = target.get_image().await.expect("image");

        // Assert
        assert!(attached.is_ok(), "got {attached:?}");
        assert!(rendered.is_ok(), "got {rendered:?}");
        assert_eq!(
            image[..4],
            [0, 255, 0, 255],
            "the member's color reaches the target"
        );
    });
}

// Story: a fullscreen shader that takes its built-ins through a struct argument
// reads no vertex input, so it renders with no mesh.
#[test]
fn builtin_struct_renders_fullscreen() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");
        let shader = crate::Shader::new(
            r#"
struct VIn { @builtin(vertex_index) index: u32 };
@vertex fn vs_main(input: VIn) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  return vec4<f32>(p[input.index], 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#,
        )
        .expect("shader");

        let rendered = renderer.render(&shader, &target);
        let image = target.get_image().await.expect("image");

        assert!(rendered.is_ok(), "got {rendered:?}");
        assert_eq!(
            image[..4],
            [255, 255, 255, 255],
            "the triangle covers the target"
        );
    });
}

const SIDE: u32 = 16;
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
/// A pixel inside the left strip, and one inside the right strip.
const LEFT: (u32, u32) = (4, 8);
const RIGHT: (u32, u32) = (12, 8);

/// A material that glows red without any light.
fn red_glow() -> crate::Material {
    crate::Material::pbr()
        .base_color([0.0, 0.0, 0.0, 1.0])
        .emissive([1.0, 0.0, 0.0])
}

/// A model whose mesh spans `left` to `right` across the middle of the view.
fn strip(material: &crate::Material, left: f32, right: f32) -> crate::Model {
    let mesh = crate::Mesh::new();
    let corners = [
        [left, -0.5, 0.0],
        [right, -0.5, 0.0],
        [right, 0.5, 0.0],
        [left, -0.5, 0.0],
        [right, 0.5, 0.0],
        [left, 0.5, 0.0],
    ];
    mesh.add_vertices(
        corners
            .map(|corner| crate::Vertex::pbr(corner).set(crate::Vertex::NORMAL, [0.0, 0.0, 1.0])),
    );
    crate::Model::new(mesh, material.clone())
}

fn camera() -> crate::Camera {
    crate::Camera::perspective(1.0, 1.0, 0.1, 100.0)
        .expect("camera")
        .look_at([0.0, 0.0, 3.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
        .expect("camera")
}

/// The RGBA bytes at `(x, y)` of an image `SIDE` pixels wide.
fn pixel(image: &[u8], (x, y): (u32, u32)) -> [u8; 4] {
    let start = ((y * SIDE + x) * 4) as usize;
    [
        image[start],
        image[start + 1],
        image[start + 2],
        image[start + 3],
    ]
}

fn is_red(rgba: [u8; 4]) -> bool {
    rgba[0] > 200 && rgba[1] < 50 && rgba[2] < 50
}

fn is_blue(rgba: [u8; 4]) -> bool {
    rgba == [0, 0, 255, 255]
}

// Story: of two models that share a material, the hidden one draws nothing,
// whether its mesh was attached first or last, and the visible one still draws.
#[test]
fn a_hidden_model_draws_nothing() {
    pollster::block_on(async move {
        use crate::Target;
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([SIDE, SIDE])
            .await
            .expect("texture target");
        let material = red_glow();
        let left = strip(&material, -1.0, -0.2);
        let right = strip(&material, 0.2, 1.0);
        let pass = Pass::new("models");
        pass.set_clear_color(BLUE).expect("set clear color");
        pass.add(&left).expect("add left");
        pass.add(&right).expect("add right");
        pass.add(&camera()).expect("add camera");

        left.set_visible(false);
        renderer
            .render(&pass, &target)
            .expect("render with the first mesh hidden");
        let first_hidden = target.get_image().await.expect("read back");
        left.set_visible(true);
        right.set_visible(false);
        renderer
            .render(&pass, &target)
            .expect("render with the last mesh hidden");
        let last_hidden = target.get_image().await.expect("read back");

        assert!(is_blue(pixel(&first_hidden, LEFT)), "first hidden, left");
        assert!(is_red(pixel(&first_hidden, RIGHT)), "first hidden, right");
        assert!(is_red(pixel(&last_hidden, LEFT)), "last hidden, left");
        assert!(is_blue(pixel(&last_hidden, RIGHT)), "last hidden, right");
    });
}

// Story: two passes that share a material each draw only their own model,
// although the material's shader holds both meshes.
#[test]
fn passes_sharing_a_material_draw_only_their_models() {
    pollster::block_on(async move {
        use crate::Target;
        let renderer = Renderer::new();
        let frame = renderer
            .create_texture_target([SIDE, SIDE])
            .await
            .expect("frame");
        let material = red_glow();
        let mut images = Vec::new();
        let mut passes = Vec::new();
        for (name, model) in [
            ("left", strip(&material, -1.0, -0.2)),
            ("right", strip(&material, 0.2, 1.0)),
        ] {
            let image = renderer
                .create_texture_target([SIDE, SIDE])
                .await
                .expect("image");
            let pass = Pass::new(name);
            pass.set_clear_color(BLUE).expect("set clear color");
            pass.add(&model).expect("add model");
            pass.add(&camera()).expect("add camera");
            pass.set_target(&image).expect("set target");
            images.push(image);
            passes.push(pass);
        }

        renderer
            .render(&passes.iter().collect::<Vec<_>>(), &frame)
            .expect("render");
        let left = images[0].get_image().await.expect("read back");
        let right = images[1].get_image().await.expect("read back");

        assert!(is_red(pixel(&left, LEFT)), "left pass, left strip");
        assert!(is_blue(pixel(&left, RIGHT)), "left pass, right strip");
        assert!(is_blue(pixel(&right, LEFT)), "right pass, left strip");
        assert!(is_red(pixel(&right, RIGHT)), "right pass, right strip");
    });
}

/// Draws its mesh's positions in green and reads no instance attribute.
const PLAIN_SHADER: &str = r#"
@vertex fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
  return vec4<f32>(position, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(0.0, 1.0, 0.0, 1.0); }
"#;

// Story: a shader that draws a hidden Model's mesh, and no Model, still draws it,
// in a pass of its own or in the Model's pass, and so does the mesh's quick view.
#[test]
fn a_plain_shader_draws_a_model_mesh() {
    pollster::block_on(async move {
        use crate::Target;
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([SIDE, SIDE])
            .await
            .expect("texture target");
        let model = strip(&red_glow(), -1.0, -0.2);
        let pass = Pass::new("models");
        pass.set_clear_color(BLUE).expect("set clear color");
        pass.add(&model).expect("add model");
        pass.add(&camera()).expect("add camera");
        model.set_visible(false);
        let plain = crate::Shader::new(PLAIN_SHADER).expect("shader");
        plain.add_mesh(model.mesh()).expect("mesh attaches");
        let own = Pass::new("plain");
        own.set_clear_color(BLUE).expect("set clear color");
        own.add_shader(&plain).expect("add shader");

        renderer.render(&own, &target).expect("render its own pass");
        let alone = target.get_image().await.expect("read back");
        pass.add_shader(&plain).expect("add shader");
        renderer
            .render(&pass, &target)
            .expect("render the Model's pass");
        let shared = target.get_image().await.expect("read back");
        renderer
            .render(model.mesh(), &target)
            .expect("render the quick view");
        let quick = target.get_image().await.expect("read back");

        let green = [0, 255, 0, 255];
        assert_eq!(pixel(&alone, LEFT), green, "a pass of its own");
        assert_eq!(pixel(&shared, LEFT), green, "the Model's pass");
        assert!(
            is_blue(pixel(&shared, RIGHT)),
            "the Model's pass, off the strip"
        );
        assert_eq!(pixel(&quick, LEFT), [255, 255, 255, 255], "the quick view");
    });
}

// Story: a Material's shader rendered without its Models reads Model columns
// that nothing fills, and render names the columns where it drew nothing.
#[test]
fn model_columns_without_a_model_error() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([SIDE, SIDE])
            .await
            .expect("texture target");
        let model = strip(&red_glow(), -1.0, -0.2);
        let pass = Pass::new("models");
        pass.add(&model).expect("add model");

        let res = renderer.render(model.material().shader(), &target);

        let columns = "'model_0', 'model_1', 'model_2' and 'model_3'";
        assert!(
            matches!(&res, Err(RendererError::MeshError(crate::MeshError::InvalidInstance(message)))
                if message.contains(columns) && message.contains("no Model in this pass draws this mesh")),
            "got {res:?}"
        );
    });
}

const OFFSET_SHADER: &str = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@location(0) pos: vec2<f32>, @location(1) offset: vec2<f32>) -> VOut {
  return VOut(vec4<f32>(pos + offset, 0.0, 1.0));
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;

const INDEXED_SHADER: &str = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@location(0) pos: vec2<f32>, @builtin(instance_index) index: u32) -> VOut {
  return VOut(vec4<f32>(pos * f32(index + 1u), 0.0, 1.0));
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;

const TRIANGLE: [[f32; 2]; 3] = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];

// Story: a mesh can't draw more instances than it holds when the shader
// reads instance attributes, and a shader that reads none draws any count.
#[test]
fn instance_count_past_the_rows_errors() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");
        let shader = crate::Shader::new(OFFSET_SHADER).expect("shader");
        let mesh = crate::Mesh::new();
        mesh.add_vertices(TRIANGLE.map(crate::Vertex::new));
        mesh.add_instances(
            [0.0, 1.0].map(|shift| crate::Instance::new().set("offset", [shift, 0.0])),
        );
        mesh.set_instance_count(10);
        shader.add_mesh(&mesh).expect("mesh attaches");
        let indexed = crate::Shader::new(INDEXED_SHADER).expect("shader");
        let unread = crate::Mesh::new();
        unread.add_vertices(TRIANGLE.map(crate::Vertex::new));
        unread.set_instance_count(10);
        indexed.add_mesh(&unread).expect("mesh attaches");

        let past = renderer.render(&shader, &target);
        let indexed = renderer.render(&indexed, &target);

        assert!(
            matches!(&past, Err(RendererError::MeshError(crate::MeshError::InvalidInstance(message))) if message.contains("instance count 10 exceeds the 2")),
            "got {past:?}"
        );
        assert!(indexed.is_ok(), "got {indexed:?}");
    });
}

/// Colors each instance by its index: green for 0, yellow for 2.
const INDEX_COLOR: &str = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) @interpolate(flat) index: u32 };
@vertex fn vs_main(@location(0) pos: vec3<f32>, @builtin(instance_index) index: u32) -> VOut {
  return VOut(vec4<f32>(pos, 1.0), index);
}
@fragment fn fs_main(v: VOut) -> @location(0) vec4<f32> {
  return vec4<f32>(f32(v.index) / 2.0, 1.0, 0.0, 1.0);
}
"#;

/// A triangle that covers the whole view.
const COVER: [[f32; 3]; 3] = [[-1.0, -1.0, 0.0], [3.0, -1.0, 0.0], [-1.0, 3.0, 0.0]];

// Story: instances with no properties draw when the shader reads no instance
// attribute. One draws once, and three draw three times, the third one last.
#[test]
fn instances_without_properties_draw() {
    pollster::block_on(async move {
        use crate::Target;
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([SIDE, SIDE])
            .await
            .expect("texture target");
        let single = crate::Mesh::new();
        single.add_vertices(COVER.map(crate::Vertex::new));
        single.add_instance(crate::Instance::new());
        let triple = crate::Mesh::new();
        triple.add_vertices(COVER.map(crate::Vertex::new));
        triple.add_instances([(); 3].map(|_| crate::Instance::new()));

        let mut pixels = Vec::new();
        for mesh in [single, triple] {
            let shader = crate::Shader::new(INDEX_COLOR).expect("shader");
            shader.add_mesh(&mesh).expect("mesh attaches");
            let pass = Pass::new("instances");
            pass.set_clear_color(BLUE).expect("set clear color");
            pass.add_shader(&shader).expect("add shader");
            renderer.render(&pass, &target).expect("render");
            pixels.push(pixel(&target.get_image().await.expect("read back"), LEFT));
        }

        assert_eq!(pixels[0], [0, 255, 0, 255], "one instance");
        assert_eq!(pixels[1], [255, 255, 0, 255], "three instances");
    });
}

// Story: a Material's shader that also draws a mesh with instances of its own
// draws them, though they hold no properties and no Model drives that mesh.
#[test]
fn a_material_shader_draws_its_own_instances() {
    pollster::block_on(async move {
        use crate::Target;
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([SIDE, SIDE])
            .await
            .expect("texture target");
        let shader = crate::Shader::new(INDEX_COLOR).expect("shader");
        let model = strip(&crate::Material::custom(shader.clone()), 0.2, 1.0);
        let pass = Pass::new("models");
        pass.set_clear_color(BLUE).expect("set clear color");
        pass.add(&model).expect("add model");
        let crowd = crate::Mesh::new();
        crowd.add_vertices(COVER.map(crate::Vertex::pbr));
        crowd.add_instances([(); 3].map(|_| crate::Instance::new()));
        crowd.set_instance_count(3);
        shader.add_mesh(&crowd).expect("mesh attaches");

        renderer.render(&pass, &target).expect("render");
        let image = target.get_image().await.expect("read back");

        assert_eq!(
            pixel(&image, LEFT),
            [255, 255, 0, 255],
            "the third instance"
        );
    });
}

// Story: a mesh whose index list is empty draws nothing, opaque or blended,
// and the pass keeps its clear color.
#[test]
fn an_empty_index_list_draws_nothing() {
    pollster::block_on(async move {
        use crate::Target;
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([SIDE, SIDE])
            .await
            .expect("texture target");
        let opaque = strip(&red_glow(), -1.0, 1.0);
        let blended = strip(&red_glow().alpha_mode(crate::AlphaMode::Blend), -1.0, 1.0);
        let mut images = Vec::new();
        for model in [opaque, blended] {
            model.mesh().set_indices(Vec::new());
            let pass = Pass::new("empty");
            pass.set_clear_color(BLUE).expect("set clear color");
            pass.add(&model).expect("add model");
            pass.add(&camera()).expect("add camera");
            renderer.render(&pass, &target).expect("render");
            images.push(target.get_image().await.expect("read back"));
        }

        assert!(is_blue(pixel(&images[0], LEFT)), "opaque");
        assert!(is_blue(pixel(&images[1], LEFT)), "blended");
    });
}

/// Changes `mesh` at the start of each upload, `left` more times at most.
fn churn(mesh: crate::Mesh, uploads: Arc<std::sync::atomic::AtomicUsize>, left: usize) {
    if left == 0 {
        return;
    }
    crate::testing::race_hooks::once("upload", move || {
        uploads.fetch_add(1, Ordering::SeqCst);
        mesh.set_indices([0, 1, 2]);
        churn(mesh, uploads, left - 1);
    });
}

// Story: a quick view whose mesh changes during every upload fails the render
// with MeshError::Busy after PREVIEW_TRIES uploads and draws none of them.
#[test]
fn a_mesh_that_keeps_changing_fails_its_quick_view() {
    pollster::block_on(async move {
        use crate::error::Classify;
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("texture target");
        let mesh = crate::Mesh::new();
        mesh.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);
        let uploads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        // Twice the limit, so a loop without one fails here instead of hanging.
        churn(mesh.clone(), uploads.clone(), 2 * PREVIEW_TRIES);

        let error = renderer
            .render(&mesh, &target)
            .expect_err("the mesh never holds still");
        // The churn left over drops here, not with the thread's storage.
        crate::testing::race_hooks::once("upload", || {});

        let busy = crate::MeshError::Busy(PREVIEW_TRIES);
        let expected = matches!(&error, RendererError::MeshError(found) if found.to_string() == busy.to_string());
        assert!(expected, "{error:?}");
        assert_eq!(
            uploads.load(Ordering::SeqCst),
            PREVIEW_TRIES,
            "one upload per try"
        );
        assert_eq!(error.classify().name(), "FragmentColorError");
    });
}

// Story: the quick view draws only a pack of the state its check passed, so an
// instance added during the pack and cleared after it never reaches the frame.
#[test]
fn the_quick_view_draws_only_a_checked_pack() {
    pollster::block_on(async move {
        let (_, device, queue) = create_device_and_queue().await;
        let ctx = RenderContext::new(device, queue);
        let mesh = crate::Mesh::new();
        let corners = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
        mesh.add_vertices(corners.map(|corner| crate::Vertex::new(corner).set("color", [1.0; 4])));
        // The quick view reads the instance color before the vertex one.
        mesh.add_instance(crate::Instance::new().set("color", [1.0; 4]));
        let (added, cleared) = (mesh.clone(), mesh.clone());
        crate::testing::race_hooks::once("pack", move || {
            added.add_instance(crate::Instance::new())
        });
        crate::testing::race_hooks::once("packed", move || cleared.clear_instances());

        let draws = ctx.plan_pass(&mesh.pass).expect("the pass plans");

        let MeshDraws::Batched(list) = &draws.shaders[0].draws else {
            panic!("a batched draw");
        };
        assert_eq!(list[0].instances.count, 1, "the drawn instances");
        assert!(
            list[0].instances.buffer.is_none(),
            "no instance buffer after the clear"
        );
    });
}

// Story: a shader keeps layouts for the pack they came from, so a second Renderer
// whose upload packed newer instances under the same generation builds its own.
#[test]
fn kept_layouts_belong_to_their_pack() {
    pollster::block_on(async move {
        let mut contexts = Vec::new();
        for _ in 0..2 {
            let (_, device, queue) = create_device_and_queue().await;
            contexts.push(RenderContext::new(device, queue));
        }
        let shader = crate::Shader::new(
            "@vertex fn vs_main(@location(0) p: vec2<f32>, @location(1) offset: vec2<f32>) -> @builtin(position) vec4<f32> { return vec4<f32>(p + offset, 0.0, 1.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
        )
        .expect("shader");
        let mesh = crate::Mesh::new();
        mesh.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);
        let wide = || {
            crate::Instance::new()
                .set("color", [1.0; 4])
                .set("offset", [0.0, 0.0])
        };
        mesh.add_instances([wide(), wide()]);
        shader.add_mesh(&mesh).expect("attach");
        let stride = |context: &RenderContext| {
            let upload = mesh.object.upload(context).expect("upload");
            let build = || vertex_buffer_layouts(&shader.object, &mesh.object, &upload.schemas);
            let layouts = mesh
                .object
                .fetch_layouts(shader.object.id, &upload.schemas, build)
                .expect("layouts");
            layouts.instance.map(|instance| instance.stride)
        };
        let first = stride(&contexts[0]);
        // The instances change after the second upload read the generation.
        let late = mesh.clone();
        crate::testing::race_hooks::once("upload", move || {
            late.clear_instances();
            late.add_instances([(); 2].map(|_| crate::Instance::new().set("offset", [0.0, 0.0])));
        });
        let second = stride(&contexts[1]);

        assert_eq!(first, Some(24), "color and offset");
        assert_eq!(second, Some(8), "offset alone");
    });
}

// Story: a blended Model draws the buffers the plan uploaded and checked, so a
// vertex added between the plan and the blend phase waits for the next render.
#[test]
fn the_blend_phase_draws_the_planned_buffers() {
    pollster::block_on(async move {
        use crate::Target;
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([SIDE, SIDE])
            .await
            .expect("texture target");
        let model = strip(&red_glow().alpha_mode(crate::AlphaMode::Blend), -1.0, 1.0);
        let pass = Pass::new("blended");
        pass.set_clear_color(BLUE).expect("set clear color");
        pass.add(&model).expect("add model");
        pass.add(&camera()).expect("add camera");
        // A seventh vertex leaves a partial triangle, which a pack refuses.
        let late = model.mesh().clone();
        crate::testing::race_hooks::once("blend phase", move || {
            late.add_vertex(crate::Vertex::pbr([0.0, 0.0, 0.0]));
        });

        let drawn = renderer.render(&pass, &target);
        let image = target.get_image().await.expect("read back");
        let next = renderer.render(&pass, &target);

        assert!(drawn.is_ok(), "{drawn:?}");
        assert!(is_red(pixel(&image, LEFT)), "the planned strip");
        let error = next.expect_err("the next plan packs seven vertices");
        let expected = RendererError::from(crate::MeshError::VertexCount(7));
        assert_eq!(error.to_string(), expected.to_string());
    });
}

/// `strip`, with one more property on each vertex, which widens its stride.
fn wide_strip(material: &crate::Material, left: f32, right: f32) -> crate::Model {
    let narrow = strip(material, left, right);
    let mesh = crate::Mesh::new();
    let vertices = narrow.mesh().object.verts.read().clone();
    mesh.add_vertices(vertices.into_iter().map(|vertex| vertex.set("weight", 1.0)));
    crate::Model::new(mesh, material.clone())
}

// Story: two Models that share a material draw, opaque or blended, though
// their meshes lay out vertices differently, where the second one read the first one's layout.
#[test]
fn models_with_other_layouts_share_a_material() {
    pollster::block_on(async move {
        use crate::Target;
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([SIDE, SIDE])
            .await
            .expect("texture target");
        let mut images = Vec::new();
        for material in [red_glow(), red_glow().alpha_mode(crate::AlphaMode::Blend)] {
            let pass = Pass::new("layouts");
            pass.set_clear_color(BLUE).expect("set clear color");
            pass.add(&strip(&material, -1.0, -0.2)).expect("add left");
            pass.add(&wide_strip(&material, 0.2, 1.0))
                .expect("add right");
            pass.add(&camera()).expect("add camera");
            renderer.render(&pass, &target).expect("render");
            images.push(target.get_image().await.expect("read back"));
        }

        for (image, mode) in images.iter().zip(["opaque", "blended"]) {
            assert!(is_red(pixel(image, LEFT)), "{mode}, left");
            assert!(is_red(pixel(image, RIGHT)), "{mode}, right");
        }
    });
}

// Story: the transforms of more Models than the device's largest buffer holds
// fail the plan, before any frame is taken or any buffer is made.
#[test]
fn model_buffers_stay_within_the_buffer_limit() {
    pollster::block_on(async move {
        let (device, queue) = device::request_small_device(65536).await;
        let ctx = RenderContext::new(device, queue);
        let material = red_glow();
        let mesh = strip(&material, -1.0, 1.0).mesh().clone();
        let pass = Pass::new("crowd");
        // One more Model than 64 KiB of 64-byte transforms holds.
        for _ in 0..1025 {
            pass.add(&crate::Model::new(mesh.clone(), material.clone()))
                .expect("add");
        }

        let planned = ctx.plan_pass(&pass.object);

        assert!(
            matches!(&planned, Err(RendererError::LimitError { item, limit, value: 65600, max: 65536 })
                if item == "the pass's Model instance buffer" && limit == "max_buffer_size"),
            "got {:?}",
            planned.err()
        );
    });
}

/// A shader that reads one input at `location` past the position.
fn read_location(location: u32) -> String {
    format!(
        "@vertex fn vs_main(@location(0) position: vec2<f32>, @location({location}) extra: f32) -> @builtin(position) vec4<f32> {{
  return vec4<f32>(position * extra, 0.0, 1.0);
}}
@fragment fn fs_main() -> @location(0) vec4<f32> {{ return vec4<f32>(1.0); }}"
    )
}

// Story: a vertex input at a location past the device's attribute limit fails
// render on the CPU, before any pipeline builds, and one below the limit draws.
#[test]
fn vertex_locations_stay_below_the_device_limit() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");
        let limit = target.context.device.limits().max_vertex_attributes;
        let mesh = crate::Mesh::new();
        mesh.add_vertices(TRIANGLE.map(|corner| crate::Vertex::new(corner).set("extra", 1.0)));
        let past = crate::Shader::new(read_location(limit)).expect("shader");
        past.add_mesh(&mesh).expect("mesh attaches");
        let last = crate::Shader::new(read_location(limit - 1)).expect("shader");
        last.add_mesh(&mesh).expect("mesh attaches");

        let refused = renderer.render(&past, &target);
        let drawn = renderer.render(&last, &target);

        let expected = format!(
            "shader input 'extra' @location({limit}) needs max_vertex_attributes of at least {}, and the device has {limit}",
            limit + 1
        );
        assert!(
            matches!(&refused, Err(error @ RendererError::LimitError { .. }) if error.to_string() == expected),
            "got {refused:?}"
        );
        assert!(drawn.is_ok(), "got {drawn:?}");
    });
}

// Story: a vertex or instance stride past the device's limit fails render on the
// CPU, before any pipeline builds, though the shader reads few of the attributes.
#[test]
fn strides_stay_within_the_device_limit() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");
        let limit = target
            .context
            .device
            .limits()
            .max_vertex_buffer_array_stride;
        if limit > 1 << 16 {
            eprintln!("the device reports no practical stride limit");
            return;
        }
        // Enough four-float properties to pass the limit.
        let count = limit / 16 + 1;
        let widen = |vertex: crate::Vertex| {
            (0..count).fold(vertex, |vertex, index| {
                vertex.set(&format!("p{index}"), [0.0, 0.0, 0.0, 0.0])
            })
        };
        let vertices = crate::Mesh::new();
        vertices.add_vertices(TRIANGLE.map(|corner| widen(crate::Vertex::new(corner))));
        let plain = crate::Shader::new(INDEXED_SHADER).expect("shader");
        plain.add_mesh(&vertices).expect("mesh attaches");
        let instances = crate::Mesh::new();
        instances.add_vertices(TRIANGLE.map(crate::Vertex::new));
        let instance = widen(crate::Vertex::new([0.0, 0.0])).set("offset", [0.0, 0.0]);
        instances.add_instance(instance);
        let offset = crate::Shader::new(OFFSET_SHADER).expect("shader");
        offset.add_mesh(&instances).expect("mesh attaches");

        let vertex = renderer.render(&plain, &target);
        let instance = renderer.render(&offset, &target);

        let stride = u64::from(count * 16 + 8);
        let message = |stream: &str| {
            format!(
                "the mesh's {stream} stride needs max_vertex_buffer_array_stride of at least {stride}, and the device has {limit}"
            )
        };
        assert!(
            matches!(&vertex, Err(error @ RendererError::LimitError { .. }) if error.to_string() == message("vertex")),
            "got {vertex:?}"
        );
        assert!(
            matches!(&instance, Err(error @ RendererError::LimitError { .. }) if error.to_string() == message("instance")),
            "got {instance:?}"
        );
    });
}

// Story: acquire_frame retries once on Lost/Outdated and returns other errors as-is.
#[test]
fn acquire_frame_exercises_paths() {
    use std::collections::VecDeque;
    struct DummyFrame;
    impl crate::TargetFrame for DummyFrame {
        fn view(&self) -> &wgpu::TextureView {
            panic!("not used")
        }
        fn format(&self) -> wgpu::TextureFormat {
            wgpu::TextureFormat::Rgba8Unorm
        }
        fn size(&self) -> wgpu::Extent3d {
            wgpu::Extent3d::default()
        }
        fn present(self: Box<Self>, _queue: &wgpu::Queue) {}
        fn auto_present(&self) -> bool {
            false
        }
    }
    struct DummyTarget {
        size: crate::Size,
        seq:
            parking_lot::RwLock<VecDeque<Result<Box<dyn crate::TargetFrame>, crate::SurfaceError>>>,
    }
    impl DummyTarget {
        fn new(seq: Vec<Result<Box<dyn crate::TargetFrame>, crate::SurfaceError>>) -> Self {
            Self {
                size: crate::Size::from((1u32, 1u32)),
                seq: parking_lot::RwLock::new(seq.into()),
            }
        }
    }

    impl crate::targets::FrameSource for DummyTarget {
        fn get_current_frame(&self) -> Result<Box<dyn crate::TargetFrame>, crate::SurfaceError> {
            self.seq
                .write()
                .pop_front()
                .unwrap_or_else(|| Ok(Box::new(DummyFrame)))
        }

        fn owner(&self) -> Option<Arc<RenderContext>> {
            None
        }
    }
    impl crate::Target for DummyTarget {
        fn size(&self) -> crate::Size {
            self.size
        }
        fn resize(
            &mut self,
            _s: impl Into<crate::Size>,
        ) -> Result<(), crate::texture::TextureError> {
            Ok(())
        }
        async fn get_image(&self) -> Result<Vec<u8>, crate::texture::TextureError> {
            Ok(Vec::new())
        }
    }

    pollster::block_on(async move {
        let (_adapter, device, queue) = create_device_and_queue().await;
        let ctx = RenderContext::new(device, queue);

        // Case 1: Lost then Ok -> success
        let t1 = DummyTarget::new(vec![
            Err(crate::SurfaceError::Lost),
            Ok(Box::new(DummyFrame)),
        ]);
        let f1 = ctx.acquire_frame(&t1);
        assert!(f1.is_ok());

        // Case 2: OutOfMemory -> error returned
        let t2 = DummyTarget::new(vec![Err(crate::SurfaceError::OutOfMemory)]);
        let f2 = ctx.acquire_frame(&t2);
        assert!(matches!(f2, Err(crate::SurfaceError::OutOfMemory)));

        // Case 3: Outdated then Timeout -> returns second error
        let t3 = DummyTarget::new(vec![
            Err(crate::SurfaceError::Outdated),
            Err(crate::SurfaceError::Timeout),
        ]);
        let f3 = ctx.acquire_frame(&t3);
        assert!(matches!(f3, Err(crate::SurfaceError::Timeout)));
    });
}

// Story: A render made only of compute passes acquires no frame; one with a render pass still does.
#[test]
fn compute_only_render_skips_frame_acquisition() {
    pollster::block_on(async move {
        let (_adapter, device, queue) = create_device_and_queue().await;
        let ctx = RenderContext::new(device, queue);
        let compute =
            Shader::new("@compute @workgroup_size(1)\nfn cs_main() {}\n").expect("compute shader");
        let draw = Shader::default();

        let compute_only = vec![
            Pass::from_shader("first", &compute),
            Pass::from_shader("second", &compute),
        ];
        ctx.render(&compute_only, &FramelessTarget)
            .expect("compute-only render needs no frame");

        let mixed = vec![
            Pass::from_shader("compute", &compute),
            Pass::from_shader("draw", &draw),
        ];
        let result = ctx.render(&mixed, &FramelessTarget);
        assert!(matches!(
            result,
            Err(RendererError::SurfaceError(crate::SurfaceError::Occluded))
        ));
    });
}

// Story: Repeated compute renders reuse one cached pipeline, shared by shaders with the same source.
#[test]
fn compute_pipeline_cache_reuses_pipelines() {
    pollster::block_on(async move {
        let (_adapter, device, queue) = create_device_and_queue().await;
        let ctx = RenderContext::new(device, queue);
        let source = "
@group(0) @binding(0) var<uniform> scale: f32;
@group(0) @binding(1) var<storage, read_write> cache_probe: array<f32, 4>;
@compute @workgroup_size(1)
fn cs_main() { cache_probe[0] = scale; }
";
        let first = Shader::new(source).expect("compute shader");
        let second = Shader::new(source).expect("compute shader");
        let passes = vec![
            Pass::from_shader("first", &first),
            Pass::from_shader("second", &second),
        ];

        for _ in 0..3 {
            ctx.render(&passes, &FramelessTarget)
                .expect("compute render");
        }

        assert_eq!(ctx.compute_pipelines.len(), 1);
    });
}

// Story: Shader uses only group(1) uniform; renderer creates placeholder empty bind group for group 0.
#[test]
fn placeholder_empty_bind_group_for_lower_group() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");

        let wgsl = r#"
@group(1) @binding(0) var<uniform> Tint: vec4<f32>;
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    return out;
}
@fragment
fn main() -> @location(0) vec4<f32> {
    return Tint;
}
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");
        shader
            .set("Tint", [0.2f32, 0.4, 0.6, 1.0])
            .expect("set uniform");

        let res = renderer.render(&shader, &target);
        assert!(
            res.is_ok(),
            "render should succeed with placeholder group 0"
        );
    });
}

// Story: Depth sample_count mismatch vs pass sample_count returns a descriptive error.
#[test]
fn depth_msaa_sample_count_mismatch_errors() {
    pollster::block_on(async move {
        let (_adapter, device, queue) = create_device_and_queue().await;
        let ctx = Arc::new(RenderContext::new(device, queue));

        let color_format = wgpu::TextureFormat::Rgba8Unorm;
        let size = wgpu::Extent3d {
            width: 8,
            height: 8,
            depth_or_array_layers: 1,
        };
        let frame = create_test_frame(&ctx.device, size, color_format);

        // Force pass sample_count = 4 while making depth texture sample_count = 1
        ctx.set_sample_count(4);
        let depth_obj = crate::TextureObject::create_depth_texture(&ctx, size, 1);
        let depth_obj = std::sync::Arc::new(depth_obj);
        let depth = crate::texture::Registration::create(&ctx, depth_obj.clone());

        let shader = Shader::default();
        let pass = Pass::from_shader("msaa-depth-mismatch", &shader);
        let first_pass = {
            let passes = pass.passes();
            passes.first().cloned().expect("pass")
        };
        first_pass.set_depth_target(depth);

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let draws = ctx.plan_pass(&first_pass).expect("the pass plans");
        let res = ctx.process_render_pass(&mut encoder, &first_pass, &frame, &draws);
        assert!(res.is_err());
        let s = format!("{}", res.unwrap_err());
        assert!(s.contains("sample_count"), "unexpected error: {s}");
    });
}

// Story: Compute-only pass builds pipeline and dispatches with a simple uniform.
#[test]
fn compute_pipeline_smoke() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([4u32, 4u32])
            .await
            .expect("texture target");

        let wgsl = r#"
@group(0) @binding(0) var<uniform> U: vec4<f32>;
@compute @workgroup_size(1)
fn cs_main() { _ = U; }
            "#;
        let shader = crate::Shader::new(wgsl).expect("compute shader");
        let pass = Pass::from_shader("c", &shader);
        assert!(pass.is_compute(), "pass should be compute");

        // Set the uniform to ensure a buffer is uploaded
        shader
            .set("U", [1.0f32, 2.0, 3.0, 4.0])
            .expect("set uniform");

        // Render should only run compute path and succeed
        let res = renderer.render(&pass, &target);
        assert!(res.is_ok(), "compute render ok: {:?}", res.err());
    });
}

// Story: Per-pass color target override renders without error (offscreen color target).
#[test]
fn render_to_offscreen_color_target_smoke() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let present_target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("present target");
        let color_target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("offscreen color target");

        let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    return out;
}
@fragment
fn main() -> @location(0) vec4<f32> { return vec4<f32>(0.8, 0.2, 0.1, 1.0); }
            "#;
        let shader = crate::Shader::new(wgsl).expect("shader");
        let pass = Pass::from_shader("off", &shader);
        pass.set_target(&color_target)
            .expect("SAFETY: tests construct color_target via Renderer::create_texture so target.id() resolves");

        let res = renderer.render(&pass, &present_target);
        assert!(
            res.is_ok(),
            "render with per-pass color target ok: {:?}",
            res.err()
        );
    });
}
