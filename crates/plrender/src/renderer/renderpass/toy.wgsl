///// Shared Shader Code /////
struct Window {
    resolution: vec2<f32>,
    antialiaser: f32,
    time: f32,
    fps: f32,
    frame_delta: f32,
    cursor: vec2<f32>,
    drag_start: vec2<f32>,
    drag_end: vec2<f32>,
    mouse_left_pressed: f32,
    mouse_left_clicked: f32,
};

@group(0) @binding(0)
var<uniform> window: Window;

struct Globals {
    view_projection: mat4x4<f32>,
}
@group(0) @binding(1)
var<uniform> globals: Globals;

// just in case...
// @group(0) @binding(2)
// var texture_sampler: sampler;


////// Vertex Shader //////

// did not exist in original
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coords: vec2<f32>,
};

@vertex // Shadertoy Vertex Shader (fullwindow quad)
fn main_vs(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let x = f32(i32((vertex_index << 1u) & 2u));
    let y = f32(i32(vertex_index & 2u));
    let uv = vec2<f32>(x, y);
    let out = 2.0 * uv - vec2<f32>(1.0, 1.0);
    // original but with new output



    return VertexOutput(
        vec4<f32>(out.x, out.y, 0.0, 1.0),
        uv * vec2<f32>(1.0, -1.0) + vec2<f32>(0.0, 1.0)
    );
}
// original (unmodified; takes builtin input only)
// fn main_vs(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
//     let x = f32(i32((vertex_index << 1u) & 2u));
//     let y = f32(i32(vertex_index & 2u));
//     let uv = vec2<f32>(x, y);
//     let out = 2.0 * uv - vec2<f32>(1.0, 1.0);
//     return vec4<f32>(out.x, out.y, 0.0, 1.0);
// }

//////////


struct Locals {
    position: vec4<f32>,
    rotation: vec4<f32>,
    scale: vec4<f32>,
    color: vec4<f32>,
    bounds: vec4<f32>,
    radius: f32,
    border: f32,
    padding: f32, // unused; needed for alignment
    sdf_flags: u32,
    texture_uv: vec4<f32>,
}
@group(1) @binding(0)
var<uniform> object: Locals;

@group(1) @binding(1)
var texture: texture_2d<f32>;

@group(1) @binding(2)
var texture_sampler: sampler;

@fragment
fn fs_main_default(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    // return vec4<f32>(cos(window.time), sin(window.time), 1.0 - cos(window.time), 1.0);

    let uv = frag_coord.xy / window.resolution;
    let half = vec3<f32>(0.5, 0.5, 0.5);
    let time = vec3<f32>(window.time, window.time, window.time);
    let col: vec3<f32> = half + half * cos(time + uv.xyx + vec3<f32>(0.0, 2.0, 4.0));
    return vec4<f32>(col.x, col.y, col.z, 1.0);
}

@fragment
fn main_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    // IF TEXTURE
    let texture_color = textureSample(texture, texture_sampler, in.texture_coords);
    // return sample * vec4<f32>(sample.aaa, 1.0);

    // IF SHAPE
    let aa = window.antialiaser;

    let radius = object.radius;
    let border = object.border;
    var position = object.position.xy;

    let normalized = in.position.xy / window.resolution;
    var uv = (normalized * 2.0) - vec2(1.0);

    uv.x *= window.resolution.x / window.resolution.y;
    position.x *= window.resolution.x / window.resolution.y;

    let dist = distance(uv, position);
    let abs = abs(dist - radius);
    let alpha = (1.0 - smoothstep(border - aa, border + aa, abs(dist - radius))) * object.color.a;

    let circle_color = vec4<f32>(object.color.rgb, alpha);

    let blend = mix(texture_color, circle_color, alpha);

    return blend;
}


fn fullscreen_texture(texture_coords: vec2<f32>) -> vec4<f32> {
    // IF TEXTURE
    let sample = textureSample(texture, texture_sampler, texture_coords);
    return sample * vec4<f32>(sample.aaa, 1.0);
}

////// Fragment Shader //////



// NOTE: This area is reserved for Shader composition.
//       All Objects containing custom shaders will
//       inject their code here.
//
//       Custom shaders are currently hardcoded below,
//       but will eventually come from the shader composer.
//
// WHAT: This currently contains signed distance functions.
//       Objects can choose one of them by using bitflags.
//       This allows for different shapes to be rendererd
//       in the same draw call as instances.
//
//////////// START OF SHADER COMPOSITION PART ////////////

// more on https://iquilezles.org/articles/distfunctions2d
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
//////////// END OF SHADER COMPOSITION PART ////////////






fn min2(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        min(a.x, b.x),
        min(a.y, b.y)
    );
}

fn min3(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        min(a.x, b.x),
        min(a.y, b.y),
        min(a.z, b.z)
    );
}

fn min4(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        min(a.x, b.x),
        min(a.y, b.y),
        min(a.z, b.z),
        min(a.w, b.w)
    );
}

fn max2(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        max(a.x, b.x),
        max(a.y, b.y)
    );
}

fn max3(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        max(a.x, b.x),
        max(a.y, b.y),
        max(a.z, b.z)
    );
}

fn max4(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        max(a.x, b.x),
        max(a.y, b.y),
        max(a.z, b.z),
        max(a.w, b.w)
    );
}

fn splat2(v: f32) -> vec2<f32> {
    return vec2<f32>(v, v);
}

fn splat3(v: f32) -> vec3<f32> {
    return vec3<f32>(v, v, v);
}

fn splat4(v: f32) -> vec4<f32> {
    return vec4<f32>(v, v, v, v);
}

fn mix3(a: vec3<f32>, b: vec3<f32>, d: f32) -> vec3<f32> {
    return vec3<f32>(
        mix(a.x, b.x, d),
        mix(a.y, b.y, d),
        mix(a.z, b.z, d)
    );
}

fn sd_round_box(p: vec2<f32>, b: vec2<f32>, in_r: vec4<f32>) -> f32 {
    var r: vec4<f32>;

    r = in_r;

    if p.x > 0.0 {
        r.x = r.x;
        r.y = r.y;
    } else {
        r.x = r.z;
        r.y = r.w;
    };
// r.x  = (p.y>0.0)?r.x  : r.y;
    if p.y > 0.0 {
        r.x = r.x;
    } else {
        r.x = r.x;
    }
    //     r.xy = (p.x>0.0)?r.xy : r.zw;
//     r.x  = (p.y>0.0)?r.x  : r.y;
    let q = abs(p) - b + vec2<f32>(r.x, r.x);
//     vec2 q = abs(p)-b+r.x;
//     return min(max(q.x,q.y),0.0) + length(max(q,0.0)) - r.x;
    return min(max(q.x, q.y), 0.0) + length(max2(q, splat2(0.0))) - r.x;
    // return 0.0;
}


//
// IT WORKS!!!!!!!!
//
// The Shader below is adapted from ShaderToy.
// I will use it as a text for the shader composition.
//
// @fragment
fn shadertoy(frag_coord: vec4<f32>) -> vec4<f32> {
    // return vec4<f32>(cos(window.time), sin(window.time), 1.0 - cos(window.time), 1.0);

    // let uv = frag_coord.xy / window.resolution;
    // let half = vec3<f32>(0.5, 0.5, 0.5);
    // let time = vec3<f32>(window.time, window.time, window.time);
    // let col: vec3<f32> = half + half * cos(time + uv.xyx + vec3<f32>(0.0, 2.0, 4.0));
    // return vec4<f32>(col.x, col.y, col.z, 1.0);

    // let p =
    // let two = vec2<f32>(2.0, 2.0);
    let p = (splat2(2.0) * frag_coord.xy - window.resolution) / splat2(window.resolution.y);

    let si = vec2<f32>(0.9, 0.6);
	// vec2 si = vec2(0.9,0.6);
    // vec4 ra = 0.3 + 0.3*cos( 2.0*iTime + vec4(0,1,2,3) );
    // let third = vec4<f32>(0.3, 0.3, 0.3, 0.3);
    // let t = vec2<f32>()
    let ra = splat4(0.3) + splat4(0.3) * cos(splat4(window.time) * vec4<f32>(0.0, 1.0, 2.0, 3.0));
    let d = sd_round_box(p, si, ra);
	// float d = sdRoundBox( p, si, ra );
    let col = splat3(1.0) - sign(d) * vec3<f32>(0.1, 0.4, 0.7);
    let col1 = col * splat3(1.0) - splat3(exp(-3.0 * abs(d)));
    let col2 = col1 * splat3(0.8) * splat3(0.2) * cos(150.0 * d);
    let col3 = mix3(col2, splat3(1.0), 1.0 - smoothstep(0.0, 0.02, abs(d)));

    // vec3 col = vec3(1.0) - sign(d)*vec3(0.1,0.4,0.7);
	// col *= 1.0 - exp(-3.0*abs(d));
	// col *= 0.8 + 0.2*cos(150.0*d);
	// col = mix( col, vec3(1.0), 1.0-smoothstep(0.0,0.02,abs(d)) );

	// fragColor = vec4(col,1.0);
    // return vec4<f32>(0.5, 0.5, 0.5, 0.5);
    return vec4<f32>(col3, 1.0);
}
