
struct Globals {
    view_projection: mat4x4<f32>,
    resolution: vec2<f32>,
    antialiaser: f32,
    time: f32,
};
@group(0) @binding(0)
var<uniform> globals: Globals;

@group(0) @binding(1)
var texture_sampler: sampler;

struct Locals {
    position: vec4<f32>,
    scale: vec4<f32>,
    rotation: vec4<f32>,
    bounds: vec4<f32>,
    texture: vec4<f32>,
};
@group(1) @binding(0)
var<uniform> object: Locals;

@group(1) @binding(1)
var image: texture_2d<f32>;

fn rotation(axis: vec3<f32>, quat: vec4<f32>) -> vec3<f32> {
    return axis + 2.0 * cross(quat.xyz, cross(quat.xyz, axis) + quat.w * axis);
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coords: vec2<f32>,
};

@vertex
fn main_vs(@builtin(vertex_index) index: u32) -> VertexOutput {
    let texture_coords = vec2<f32>(
        f32(i32(index) / 2),
        f32(i32(index) & 1),
    );
    let axis = vec3<f32>(
        mix(object.bounds.xw, object.bounds.zy, texture_coords),
        0.0
    );
    let world = object.scale.xyz * rotation(axis, object.rotation) + object.position.xyz;

    let width = globals.resolution.x;
    let height = globals.resolution.y;

    let position = globals.view_projection * vec4<f32>(world, 1.0);

    let texture_coords_sub = mix(object.texture.xy, object.texture.zw, texture_coords);
    return VertexOutput(position, texture_coords_sub);
}

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


// @fragment
// fn main_fs(@location(0) texture_coords: vec2<f32>) -> @location(0) vec4<f32> {
//     let sample = textureSample(image, sam, texture_coords);

//     // pre-multiply the alpha
//     return sample * vec4<f32>(sample.aaa, 1.0);
// }

@fragment
fn main_fs(pixel: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(image, texture_sampler, pixel.texture_coords);
    return sample * vec4<f32>(sample.aaa, 1.0);

    // let aa = globals.antialiaser;

    // let border = object.scale.z;
    // let radius = object.position.z;
    // var position = object.position.xy;

    // let normalized = pixel.position.xy / globals.resolution;
    // var uv = (normalized * 2.0) - vec2(1.0);

    // uv.x *= globals.resolution.x / globals.resolution.y;
    // position.x *= globals.resolution.x / globals.resolution.y;

    // //let dist = distance(uv, position);
    // let dist = length(position);
    // let alpha = (1.0 - smoothstep(border - aa, border + aa, abs(dist - radius))) * object.color.a;

    // return vec4<f32>(pixel.texture_coords.x, pixel.texture_coords.y, 1.0, 1.0);
    //vec4<f32>(object.color.rgb, alpha); //object.color; //texture_color * vec4<f32>(object.color.rgb, alpha);
}

// @fragment
// fn fs_main(in: VertexOutput) -> FragmentOutput {
//     var out: FragmentOutput;
//     let aa = globals.antialiaser;

//     let radius = object.bounds;
//     let border = object.scale.z;
//     var position = object.position.xy;

//     let normalized = in.position.xy / globals.resolution;
//     var uv = (normalized * 2.0) - vec2(1.0);

//     uv.x *= globals.resolution.x / globals.resolution.y;
//     position.x *= globals.resolution.x / globals.resolution.y;

//     let dist = distance(uv, position);
//     let alpha = (1.0 - smoothstep(border - aa, border + aa, abs(dist - radius))) * circle.color.a;

//     out.color = vec4<f32>(circle.color.rgb, alpha);

//     return out;
// }
