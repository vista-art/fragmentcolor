//! Catalog gallery for the `sample/` registry category.
//!
//! Renders one 256x256 PNG per sampling helper in
//! `docs/website/public/shaders/sample/`. These functions are pure point /
//! direction generators (low-discrepancy sequences, disk warps, hemisphere
//! mappings, screen-space dither). None of them sample a texture, so we
//! visualize each one by either:
//!   - scattering a fixed number of samples and rendering them as dots
//!     (disk warps, sequences, Vogel disk, Hammersley),
//!   - mapping the input UV through the function and color-coding the
//!     result (1D radical inverses, IGN, sphere/hemisphere directions
//!     projected to 2D).
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_sample
//!
//! Skipped silently on systems without a wgpu-compatible adapter; this is
//! a visual smoke test, not a CI gate.

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

fn compositions() -> Vec<Composition> {
    vec![
        // disk_concentric — Shirley concentric warp from [0,1]^2 → unit disk.
        // Visualization: take a 16x16 stratified grid of UVs, warp each cell to a
        // disk point, render small dots. Pairs with disk_uniform for comparison.
        Composition {
            name: "disk_concentric",
            description: "256 stratified UV cells warped to the unit disk via Shirley's concentric mapping. Dots clustered = stratification preserved.",
            slugs: &["sample/disk_concentric"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.2;
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let circle = step(length(p), 1.0) * 0.06;
    var col = bg + vec3<f32>(circle);
    let n = 16u;
    for (var iy: u32 = 0u; iy < n; iy = iy + 1u) {
        for (var ix: u32 = 0u; ix < n; ix = ix + 1u) {
            let rnd = vec2<f32>((f32(ix) + 0.5) / f32(n), (f32(iy) + 0.5) / f32(n));
            let s = disk_concentric(rnd);
            let d = length(p - s);
            let m = 1.0 - smoothstep(0.022, 0.034, d);
            col = mix(col, vec3<f32>(0.55, 0.85, 1.0), m);
        }
    }
    return vec4<f32>(col, 1.0);
}
"#,
        },
        // disk_uniform — sqrt(r) polar warp. Visualizes the same 16x16 grid for
        // visual contrast against disk_concentric.
        Composition {
            name: "disk_uniform",
            description: "256 stratified UV cells warped to the unit disk via the sqrt-radius polar warp. Even areal density, but stratification axes are warped.",
            slugs: &["sample/disk_uniform"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.2;
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let circle = step(length(p), 1.0) * 0.06;
    var col = bg + vec3<f32>(circle);
    let n = 16u;
    for (var iy: u32 = 0u; iy < n; iy = iy + 1u) {
        for (var ix: u32 = 0u; ix < n; ix = ix + 1u) {
            let rnd = vec2<f32>((f32(ix) + 0.5) / f32(n), (f32(iy) + 0.5) / f32(n));
            let s = disk_uniform(rnd);
            let d = length(p - s);
            let m = 1.0 - smoothstep(0.022, 0.034, d);
            col = mix(col, vec3<f32>(1.0, 0.75, 0.45), m);
        }
    }
    return vec4<f32>(col, 1.0);
}
"#,
        },
        // halton — 1D low-discrepancy sequence. Visualize first 256 samples on the
        // x axis as a row of bars. We map the fragment's column to an index, then
        // take halton(i, 2) and check whether the row corresponds to that y.
        Composition {
            name: "halton",
            description: "First 64 Halton samples (base 2 horizontal, base 3 vertical) plotted as 2D dots — classic low-discrepancy coverage.",
            slugs: &["sample/halton"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv;
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let grid_x = step(0.99, fract(p.x * 8.0)) + step(0.99, fract(p.y * 8.0));
    var col = bg + vec3<f32>(grid_x * 0.04);
    let n = 64u;
    for (var i: u32 = 1u; i <= n; i = i + 1u) {
        let s = vec2<f32>(halton(i, 2u), halton(i, 3u));
        let d = length(p - s);
        let m = 1.0 - smoothstep(0.012, 0.018, d);
        col = mix(col, vec3<f32>(0.95, 0.55, 0.85), m);
    }
    return vec4<f32>(col, 1.0);
}
"#,
        },
        // hammersley — 2D version using van der Corput. Visualize as dots over [0,1]^2.
        Composition {
            name: "hammersley",
            description: "First 64 Hammersley samples plotted as 2D dots — equivalent to (i/n, van_der_corput(i)).",
            slugs: &["sample/hammersley"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv;
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let grid_x = step(0.99, fract(p.x * 8.0)) + step(0.99, fract(p.y * 8.0));
    var col = bg + vec3<f32>(grid_x * 0.04);
    let n = 64u;
    for (var i: u32 = 0u; i < n; i = i + 1u) {
        let s = hammersley(i, n);
        let d = length(p - s);
        let m = 1.0 - smoothstep(0.012, 0.018, d);
        col = mix(col, vec3<f32>(0.55, 0.85, 0.75), m);
    }
    return vec4<f32>(col, 1.0);
}
"#,
        },
        // hemisphere_cosine — 3D direction. Project to 2D by orthographic xy.
        // 256 samples over a fixed normal (+z). Color by z (cosine weight visible).
        Composition {
            name: "hemisphere_cosine",
            description: "256 cosine-weighted hemisphere samples around n = +z, projected onto the xy plane and colored by cos(theta). Density biases toward the pole.",
            slugs: &["sample/hemisphere_cosine"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.2;
    let bg = vec3<f32>(0.08, 0.10, 0.16);
    let circle = step(length(p), 1.0) * 0.06;
    var col = bg + vec3<f32>(circle);
    let n_axis = vec3<f32>(0.0, 0.0, 1.0);
    let n = 16u;
    for (var iy: u32 = 0u; iy < n; iy = iy + 1u) {
        for (var ix: u32 = 0u; ix < n; ix = ix + 1u) {
            let rnd = vec2<f32>((f32(ix) + 0.5) / f32(n), (f32(iy) + 0.5) / f32(n));
            let dir = hemisphere_cosine(rnd, n_axis);
            let pt = dir.xy;
            let d = length(p - pt);
            let m = 1.0 - smoothstep(0.022, 0.034, d);
            let warm = vec3<f32>(0.95, 0.65, 0.45);
            let cool = vec3<f32>(0.45, 0.55, 0.95);
            let dot_col = mix(cool, warm, dir.z);
            col = mix(col, dot_col, m);
        }
    }
    return vec4<f32>(col, 1.0);
}
"#,
        },
        // hemisphere_uniform — 3D direction, equal-area mapping. Color by z so the
        // visible bias toward the equator (compared to cosine) is obvious.
        Composition {
            name: "hemisphere_uniform",
            description: "256 uniform hemisphere samples around n = +z, projected onto xy and colored by cos(theta). Areal density even, more samples near the rim than cosine.",
            slugs: &["sample/hemisphere_uniform"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.2;
    let bg = vec3<f32>(0.08, 0.10, 0.16);
    let circle = step(length(p), 1.0) * 0.06;
    var col = bg + vec3<f32>(circle);
    let n_axis = vec3<f32>(0.0, 0.0, 1.0);
    let n = 16u;
    for (var iy: u32 = 0u; iy < n; iy = iy + 1u) {
        for (var ix: u32 = 0u; ix < n; ix = ix + 1u) {
            let rnd = vec2<f32>((f32(ix) + 0.5) / f32(n), (f32(iy) + 0.5) / f32(n));
            let dir = hemisphere_uniform(rnd, n_axis);
            let pt = dir.xy;
            let d = length(p - pt);
            let m = 1.0 - smoothstep(0.022, 0.034, d);
            let warm = vec3<f32>(0.95, 0.85, 0.45);
            let cool = vec3<f32>(0.30, 0.45, 0.85);
            let dot_col = mix(cool, warm, dir.z);
            col = mix(col, dot_col, m);
        }
    }
    return vec4<f32>(col, 1.0);
}
"#,
        },
        // interleaved_gradient_noise — pixel-coord dither field. Visualize as
        // raw noise over the full image with the IGN function called per fragment.
        Composition {
            name: "interleaved_gradient_noise",
            description: "Jorge Jimenez's IGN evaluated per pixel — the characteristic diagonal hash pattern used as a cheap dither / sample-jitter source.",
            slugs: &["sample/interleaved_gradient_noise"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pix = in.uv * 256.0;
    let n = interleaved_gradient_noise(pix);
    let warm = vec3<f32>(0.95, 0.55, 0.30);
    let cool = vec3<f32>(0.10, 0.25, 0.55);
    return vec4<f32>(mix(cool, warm, n), 1.0);
}
"#,
        },
        // sphere_uniform — full sphere direction. Project to xy, color by z so the
        // poles read brightest/darkest.
        Composition {
            name: "sphere_uniform",
            description: "256 uniform unit-sphere samples projected onto xy and colored by z (front = warm, back = cool). Even density across the visible disk.",
            slugs: &["sample/sphere_uniform"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.2;
    let bg = vec3<f32>(0.08, 0.10, 0.16);
    let circle = step(length(p), 1.0) * 0.06;
    var col = bg + vec3<f32>(circle);
    let n = 16u;
    for (var iy: u32 = 0u; iy < n; iy = iy + 1u) {
        for (var ix: u32 = 0u; ix < n; ix = ix + 1u) {
            let rnd = vec2<f32>((f32(ix) + 0.5) / f32(n), (f32(iy) + 0.5) / f32(n));
            let dir = sphere_uniform(rnd);
            let pt = dir.xy;
            let d = length(p - pt);
            let m = 1.0 - smoothstep(0.022, 0.034, d);
            let front = vec3<f32>(0.95, 0.55, 0.85);
            let back = vec3<f32>(0.20, 0.30, 0.55);
            let dot_col = mix(back, front, dir.z * 0.5 + 0.5);
            col = mix(col, dot_col, m);
        }
    }
    return vec4<f32>(col, 1.0);
}
"#,
        },
        // van_der_corput — base-2 radical inverse. Visualize as a 2D plot:
        // x = i / N, y = van_der_corput(i). The dots trace the reverse-bit pattern.
        Composition {
            name: "van_der_corput",
            description: "First 64 base-2 van der Corput values plotted as (i / n, van_der_corput(i)) — the bit-reversed 1D low-discrepancy sequence.",
            slugs: &["sample/van_der_corput"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv;
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let grid = step(0.99, fract(p.x * 8.0)) + step(0.99, fract(p.y * 8.0));
    var col = bg + vec3<f32>(grid * 0.04);
    let n = 64u;
    for (var i: u32 = 0u; i < n; i = i + 1u) {
        let s = vec2<f32>(f32(i) / f32(n), van_der_corput(i));
        let d = length(p - s);
        let m = 1.0 - smoothstep(0.012, 0.018, d);
        col = mix(col, vec3<f32>(1.0, 0.85, 0.40), m);
    }
    return vec4<f32>(col, 1.0);
}
"#,
        },
        // vogel_disk — sunflower disk pattern. Visualize 128 dots; the spiral and
        // even angular spacing should be unmistakable.
        Composition {
            name: "vogel_disk",
            description: "128 Vogel sunflower disk samples — golden-angle spiral with sqrt-r radius gives even areal coverage.",
            slugs: &["sample/vogel_disk"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.2;
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let circle = step(length(p), 1.0) * 0.06;
    var col = bg + vec3<f32>(circle);
    let n = 128u;
    for (var i: u32 = 0u; i < n; i = i + 1u) {
        let s = vogel_disk(i, n);
        let d = length(p - s);
        let m = 1.0 - smoothstep(0.022, 0.032, d);
        col = mix(col, vec3<f32>(0.85, 0.55, 1.0), m);
    }
    return vec4<f32>(col, 1.0);
}
"#,
        },
    ]
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
    println!("     {}", comp.description);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let out_dir = std::path::Path::new("out/gallery_sample");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering sample gallery into {}/", out_dir.display());

        let mut failed = 0usize;
        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  FAIL {} - {}", comp.name, e);
                failed += 1;
            }
        }
        if failed > 0 {
            return Err(format!("{} composition(s) failed", failed).into());
        }
        Ok(())
    })
}
