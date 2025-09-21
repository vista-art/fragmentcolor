use fragmentcolor::{App, Frame, Pass, Shader, run};

// Compute + Render example:
// - Compute pass writes a time-varying pattern into a storage texture via a compute shader
// - Render pass samples that texture and draws to the window
// - Demonstrates that compute stages work and bind groups support COMPUTE visibility

const CS_WGSL: &str = r#"
@group(0) @binding(0) var img: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> res: vec2<u32>;
@group(0) @binding(2) var<uniform> t: f32;

@compute @workgroup_size(16, 16, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
  if (gid.x >= res.x || gid.y >= res.y) { return; }
  let p = vec2<f32>(f32(gid.x)/f32(res.x), f32(gid.y)/f32(res.y));
  let w = 0.5 + 0.5 * sin(6.2831*(p.x*3.0 + p.y*2.0 + t*0.3));
  let col = vec4<f32>(p.x, p.y, w, 1.0);
  textureStore(img, vec2<i32>(i32(gid.x), i32(gid.y)), col);
}
"#;

const FS_WGSL: &str = r#"
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
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
  return textureSample(tex, samp, v.uv);
}
"#;

pub fn on_resize(app: &App, sz: &winit::dpi::PhysicalSize<u32>) {
    app.resize([sz.width, sz.height]);
}

fn main() {
    let mut app = App::new();

    // Create a storage texture (power-of-two helps alignment, but not required)
    let size = [1024u32, 1024u32];
    let tex = pollster::block_on(async {
        app.renderer()
            .create_storage_texture(size, fragmentcolor::TextureFormat::Rgba, None)
            .await
            .expect("create storage texture")
    });

    // Compute shader + pass
    let cs = Shader::new(CS_WGSL).expect("compute shader");
    cs.set("img", &tex).expect("bind img");
    cs.set("res", [size[0], size[1]]).expect("set res");
    cs.set("t", 0.0f32).expect("set t");
    let pass_cs = Pass::from_shader("compute", &cs);
    // Dispatch enough workgroups to cover the texture
    let wx = size[0].div_ceil(16);
    let wy = size[1].div_ceil(16);
    pass_cs.set_compute_dispatch(wx, wy, 1);

    // Render shader + pass (fullscreen sample)
    let fs = Shader::new(FS_WGSL).expect("render shader");
    fs.set("tex", &tex).expect("bind tex");
    let pass_fs = Pass::from_shader("render", &fs);

    // Frame with compute then render
    let mut frame = Frame::new();
    frame.add_pass(&pass_cs);
    frame.add_pass(&pass_fs);

    app.on_resize(on_resize)
        .scene(frame)
        .on_redraw_requested(move |app| {
            // Animate time uniform for compute
            // Simple monotonic time accumulation
            static mut T: f32 = 0.0;
            unsafe {
                T += 0.016;
                let _ = cs.set("t", T);
            }
            // Renderer will execute compute then render in order
            let _ = app; // unused in this closure otherwise
        });

    run(&mut app);
}
