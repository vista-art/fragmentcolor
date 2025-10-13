use fragmentcolor::{App, Frame, Pass, Renderer, SetupResult, Shader, call, run};
use std::sync::Arc;
use std::time::Instant;
use winit::dpi::PhysicalSize;
use winit::window::Window;

const SWIRL_WGSL: &str = r#"
// Fullscreen swirl palette demo (shader-only)
// Ported from a Shadertoy-style fragment to WGSL

struct VOut { @builtin(position) pos: vec4<f32> };

@group(0) @binding(0) var<uniform> resolution: vec2<f32>;
@group(0) @binding(1) var<uniform> time: f32;

const TAU: f32 = 6.283185307179586;

fn pal(t: f32, brightness: vec3<f32>, contrast: vec3<f32>, oscillation: vec3<f32>, phase: vec3<f32>) -> vec3<f32> {
  return brightness + contrast * cos(TAU * (oscillation * t + phase));
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  var p = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 3.0, -1.0),
    vec2<f32>(-1.0,  3.0)
  );
  return VOut(vec4<f32>(p[i], 0.0, 1.0));
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
  // Pixel coords -> normalized 0..1
  let frag = v.pos.xy;
  let normCoord = frag / resolution;

  // Map to -1..1 and fix aspect on x
  var uv = -1.0 + 2.0 * normCoord;
  uv.x = uv.x * (resolution.x / resolution.y);

  let slowTime = time * 0.25; // slower motion like iTime/4

  // Basic patterns
  let radius = length(uv);
  let rings = sin(slowTime - radius * 15.0);
  let angle = atan2(uv.y, uv.x);
  let radar = sin(angle + slowTime);
  let swirl = sin(rings + radar + slowTime);

  // Palette parameters (animated)
  let brightnessBlend = 0.5 * (sin(time + length(uv * 20.0)) + 1.0);
  let contrastBlend = 0.5 * (sin(slowTime) + 1.0);
  let brightness = vec3<f32>(0.1 + brightnessBlend * (0.7 - 0.1));
  let contrast   = vec3<f32>(0.2 + contrastBlend * (0.5 - 0.2));
  let oscillation = vec3<f32>(0.4, 0.5 * (sin(slowTime) + 1.0), 0.2);
  let phase = vec3<f32>(0.7, 0.4, 0.1);

  let color = pal(swirl, brightness, contrast, oscillation, phase);
  return vec4<f32>(color, 1.0);
}
"#;

fn on_resize(app: &App, new_size: &PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
}

async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    let shader = Shader::new(SWIRL_WGSL)?;
    let pass = Pass::from_shader("swirl", &shader);

    let mut frame = Frame::new();
    frame.add_pass(&pass);

    app.add("shader.swirl", shader);
    app.add("frame.main", frame);

    for win in windows {
        let target = app.get_renderer().create_target(win.clone()).await?;
        app.add_target(win.id(), target);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let renderer = Renderer::new();
    let mut app = App::new(renderer);

    // Start time for animation
    let start = Instant::now();

    app.on_start(call!(setup))
        .on_resize(on_resize)
        .on_draw(move |app, id, _| {
            if let (Some(shader), Some(frame)) = (app.get::<Shader>("shader.swirl"), app.get::<Frame>("frame.main")) {
                if let Some(size) = app.window_size(id) {
                    let _ = shader.set("resolution", [size.width as f32, size.height as f32]);
                }
                let t = start.elapsed().as_secs_f32();
                let _ = shader.set("time", t);

                let r = app.get_renderer();
                let _ = app.with_target(id, |target| r.render(&*frame, target));
            }
        });

    run(&mut app);
    Ok(())
}
