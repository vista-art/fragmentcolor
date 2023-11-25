#define_import_path shapes

// more on https://iquilezles.org/articles/distfunctions2d/
fn sd_circle(p: vec2<f32>, uv: vec2<f32>, radius: f32) -> f32 {
    let distance = distance(p, uv);
    return distance - radius;
}

fn sd_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let ba = b - a;
    let pa = p - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    let q = pa - h * ba;
    let d = length(q);
    return d - r;
}

fn sd_box(p: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0);
}
