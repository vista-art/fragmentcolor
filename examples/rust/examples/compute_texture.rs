use fragmentcolor::{App, Frame, Pass, Renderer, Shader, run};

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

fn on_resize(app: &App, sz: &winit::dpi::PhysicalSize<u32>) {
    app.resize([sz.width, sz.height]);
}

fn draw(app: &App) {
    // Update time based on wall-clock
    use std::sync::OnceLock;
    use std::time::Instant;
    static START: OnceLock<Instant> = OnceLock::new();
    let t = START.get_or_init(Instant::now).elapsed().as_secs_f32();

    if let Some(cs) = app.get::<Shader>("shader.compute") {
        let _ = cs.set("t", t);
    }

    let id = app.primary_window_id();
    if let (Some(frame), Some(_target)) = (app.get::<Frame>("frame.main"), app.window_size(id)) {
        let r = app.get_renderer();
        let _ = app.with_target(id, |tgt| r.render(&*frame, tgt));
    }
}

fn setup(
    app: &App,
    windows: Vec<std::sync::Arc<winit::window::Window>>,
) -> std::pin::Pin<
    Box<
        dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + '_,
    >,
> {
    Box::pin(async move {
        // Create a storage texture (power-of-two helps alignment, but not required)
        let size = [1024u32, 1024u32];
        let tex = app
            .get_renderer()
            .create_storage_texture(size, fragmentcolor::TextureFormat::Rgba, None)
            .await?;

        // Compute shader + pass
        let cs = Shader::new(CS_WGSL)?;
        cs.set("img", &tex)?;
        cs.set("res", [size[0], size[1]])?;
        cs.set("t", 0.0f32)?;
        let pass_cs = Pass::from_shader("compute", &cs);
        // Dispatch enough workgroups to cover the texture
        let wx = size[0].div_ceil(16);
        let wy = size[1].div_ceil(16);
        pass_cs.set_compute_dispatch(wx, wy, 1);

        // Render shader + pass (fullscreen sample)
        let fs = Shader::new(FS_WGSL)?;
        fs.set("tex", &tex)?;
        let pass_fs = Pass::from_shader("render", &fs);

        // Frame with compute then render
        let mut frame = Frame::new();
        frame.add_pass(&pass_cs);
        frame.add_pass(&pass_fs);

        app.add("shader.compute", cs);
        app.add("frame.main", frame);

        // Create targets for all windows
        for win in windows {
            let target = app.get_renderer().create_target(win.clone()).await?;
            app.add_target(win.id(), target);
        }

        Ok(())
    })
}

fn main() {
    let renderer = Renderer::new();
    let mut app = App::new(renderer);

    app.on_start(setup)
        .on_resize(on_resize)
        .on_redraw_requested(draw);

    run(&mut app);
}
