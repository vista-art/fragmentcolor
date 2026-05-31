//! Catalog gallery for the `easing/` registry category.
//!
//! Renders one 256x256 PNG per easing curve in
//! `docs/website/public/shaders/easing/`. Each entry pulls a single registry
//! slug into a fullscreen fragment that plots the curve `y = ease(uv.x)` on a
//! dark grid with a soft accent line.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_easing

use fragmentcolor::{Renderer, Shader, Target};

const SIZE: u32 = 256;

/// Standard fullscreen vertex stage that emits UVs in [0, 1].
const FULLSCREEN_VS: &str = r#"
struct VertexOutput {
    @builtin(position) coords: vec4<f32>,
    @location(0) uv: vec2<f32>,
}
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
    var pts = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    let p = pts[i];
    let uv = vec2<f32>((p.x + 1.0) * 0.5, 1.0 - (p.y + 1.0) * 0.5);
    return VertexOutput(vec4<f32>(p, 0.0, 1.0), uv);
}
"#;

struct Composition {
    name: &'static str,
    description: &'static str,
    slugs: &'static [&'static str],
    fragment: &'static str,
}

/// Plot template — `CALL` is the WGSL expression that evaluates `y` from `t`,
/// `R`, `G`, `B` are the accent color components for the curve.
fn plot_fragment(call: &str, accent: (f32, f32, f32)) -> String {
    format!(
        r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {{
    let uv = in.uv;
    let t = uv.x;
    let y = {call};
    let plot_y = 1.0 - uv.y;

    // grid background
    let grid = step(0.97, max(fract(uv.x * 10.0), fract(uv.y * 10.0)));
    let bg = mix(vec3<f32>(0.10, 0.12, 0.18), vec3<f32>(0.18, 0.20, 0.26), grid);

    // 0..1 reference rectangle outline
    let frame_y = step(0.05, uv.y) * step(uv.y, 0.95);
    let frame_x = step(0.05, uv.x) * step(uv.x, 0.95);

    // curve as a soft 2-pixel-wide line
    let dist = abs(plot_y - y);
    let curve = 1.0 - smoothstep(0.0, 0.012, dist);
    let accent = vec3<f32>({r}, {g}, {b});

    return vec4<f32>(mix(bg, accent, curve * frame_y * frame_x), 1.0);
}}
"#,
        call = call,
        r = accent.0,
        g = accent.1,
        b = accent.2,
    )
}

// Accent palette: warm for "in", cool for "out", neutral for "in_out", linear muted.
const WARM_A: (f32, f32, f32) = (0.95, 0.55, 0.30); // amber
const WARM_B: (f32, f32, f32) = (0.95, 0.65, 0.40); // peach
const WARM_C: (f32, f32, f32) = (0.92, 0.45, 0.35); // coral
const WARM_D: (f32, f32, f32) = (0.95, 0.75, 0.30); // gold
const WARM_E: (f32, f32, f32) = (0.85, 0.50, 0.55); // rose
const COOL_A: (f32, f32, f32) = (0.40, 0.75, 0.95); // sky
const COOL_B: (f32, f32, f32) = (0.55, 0.85, 0.75); // mint
const COOL_C: (f32, f32, f32) = (0.50, 0.65, 0.95); // periwinkle
const COOL_D: (f32, f32, f32) = (0.45, 0.85, 0.90); // teal
const COOL_E: (f32, f32, f32) = (0.65, 0.60, 0.95); // lavender
const NEUT_A: (f32, f32, f32) = (0.85, 0.85, 0.90);
const NEUT_B: (f32, f32, f32) = (0.80, 0.85, 0.95);
const NEUT_C: (f32, f32, f32) = (0.90, 0.90, 0.85);
const NEUT_D: (f32, f32, f32) = (0.85, 0.90, 0.95);
const NEUT_E: (f32, f32, f32) = (0.92, 0.85, 0.90);
const LINEAR_C: (f32, f32, f32) = (0.75, 0.80, 0.85);

fn compositions() -> Vec<Composition> {
    // Each entry: (slug, fn-call expression, accent). We Box::leak the
    // generated fragment string so the Composition can hold a 'static &str.
    let specs: &[(&str, &str, &str, (f32, f32, f32), &str)] = &[
        (
            "in_back",
            "easing/in_back",
            "in_back(t)",
            WARM_C,
            "Cubic with overshoot before start (Penner constants).",
        ),
        (
            "in_cubic",
            "easing/in_cubic",
            "in_cubic(t)",
            WARM_A,
            "t^3 — slow start, steep finish.",
        ),
        (
            "in_expo",
            "easing/in_expo",
            "in_expo(t)",
            WARM_D,
            "2^(10*(t-1)) with snap at t = 0.",
        ),
        (
            "in_out_cubic",
            "easing/in_out_cubic",
            "in_out_cubic(t)",
            NEUT_A,
            "Cubic curve mirrored at t = 0.5.",
        ),
        (
            "in_out_expo",
            "easing/in_out_expo",
            "in_out_expo(t)",
            NEUT_B,
            "Symmetric exponential, snaps at endpoints.",
        ),
        (
            "in_out_quad",
            "easing/in_out_quad",
            "in_out_quad(t)",
            NEUT_C,
            "Quadratic curve mirrored at t = 0.5.",
        ),
        (
            "in_out_quart",
            "easing/in_out_quart",
            "in_out_quart(t)",
            NEUT_D,
            "Quartic curve mirrored at t = 0.5.",
        ),
        (
            "in_out_sine",
            "easing/in_out_sine",
            "in_out_sine(t)",
            NEUT_E,
            "-(cos(pi*t) - 1) / 2 — smooth S-curve.",
        ),
        (
            "in_quad",
            "easing/in_quad",
            "in_quad(t)",
            WARM_B,
            "t^2 — gentle slow start.",
        ),
        (
            "in_quart",
            "easing/in_quart",
            "in_quart(t)",
            WARM_E,
            "t^4 — very slow start.",
        ),
        (
            "in_sine",
            "easing/in_sine",
            "in_sine(t)",
            WARM_B,
            "1 - cos(t * pi/2) — quarter-sine ramp in.",
        ),
        (
            "linear",
            "easing/linear",
            "linear(t)",
            LINEAR_C,
            "Identity curve — no easing.",
        ),
        (
            "out_back",
            "easing/out_back",
            "out_back(t)",
            COOL_C,
            "Overshoots past 1 then settles.",
        ),
        (
            "out_bounce",
            "easing/out_bounce",
            "out_bounce(t)",
            COOL_E,
            "Three-step Penner bounce-out.",
        ),
        (
            "out_cubic",
            "easing/out_cubic",
            "out_cubic(t)",
            COOL_A,
            "1 - (1 - t)^3 — fast start, smooth landing.",
        ),
        (
            "out_elastic",
            "easing/out_elastic",
            "out_elastic(t)",
            COOL_E,
            "Damped oscillation that settles to 1.",
        ),
        (
            "out_expo",
            "easing/out_expo",
            "out_expo(t)",
            COOL_D,
            "1 - 2^(-10t) with snap at t = 1.",
        ),
        (
            "out_quad",
            "easing/out_quad",
            "out_quad(t)",
            COOL_B,
            "1 - (1 - t)^2 — gentle landing.",
        ),
        (
            "out_quart",
            "easing/out_quart",
            "out_quart(t)",
            COOL_A,
            "1 - (1 - t)^4 — very gentle landing.",
        ),
        (
            "out_sine",
            "easing/out_sine",
            "out_sine(t)",
            COOL_B,
            "sin(t * pi/2) — quarter-sine ramp out.",
        ),
    ];

    let mut out = Vec::with_capacity(specs.len());
    for (name, slug, call, accent, desc) in specs.iter().copied() {
        let fragment: &'static str = Box::leak(plot_fragment(call, accent).into_boxed_str());
        let slugs: &'static [&'static str] = Box::leak(vec![slug].into_boxed_slice());
        out.push(Composition {
            name,
            description: desc,
            slugs,
            fragment,
        });
    }
    out
}

async fn render_one(
    renderer: &Renderer,
    out_dir: &std::path::Path,
    comp: &Composition,
) -> Result<(), Box<dyn std::error::Error>> {
    let target = renderer.create_texture_target([SIZE, SIZE]).await?;

    let mut parts: Vec<&str> = comp.slugs.to_vec();
    let body = format!("{}{}", FULLSCREEN_VS, comp.fragment);
    parts.push(&body);
    let shader = Shader::new(parts.as_slice())?;

    renderer.render(&shader, &target)?;
    let bytes = target.get_image().await;
    if bytes.is_empty() {
        return Err(format!("readback returned empty buffer for {}", comp.name).into());
    }
    let img = image::RgbaImage::from_vec(SIZE, SIZE, bytes)
        .ok_or("failed to wrap pixel bytes as RgbaImage")?;
    let path = out_dir.join(format!("{}.png", comp.name));
    img.save(&path)?;
    println!("  ok {} -> {}", comp.name, path.display());
    println!("    {}", comp.description);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let out_dir = std::path::Path::new("out/gallery_easing");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering easing gallery into {}/", out_dir.display());

        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  fail {} - {}", comp.name, e);
            }
        }
        Ok(())
    })
}
