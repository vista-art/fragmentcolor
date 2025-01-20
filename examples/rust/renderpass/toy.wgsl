struct Window {
    resolution: vec2<f32>,
    antialiaser: f32,

    time: f32, // <=


    // NOT INJECTED HERE YET
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

struct Locals {
    position: vec4<f32>,
    rotation: vec4<f32>,
    scale: vec4<f32>,
    color: vec4<f32>,
    bounds: vec4<f32>,
    radius: f32,
    border: f32,
    padding: f32, // unused; needed for alignment
    sdf_flags: f32,
    texture_uv: vec4<f32>,
}
@group(1) @binding(0)
var<uniform> object: Locals;

@group(1) @binding(1)
var texture: texture_2d<f32>;

@group(1) @binding(2)
var texture_sampler: sampler;

////// Vertex Shader //////

struct VertexOutput {
    @builtin(position) frag_coord: vec4<f32>,
    @location(0) texture_coord: vec2<f32>,
};

@vertex
fn main_vs(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Produces a fullscreen quad with normalized UV coordinates
    // Vertex 0: (0, 0) // texture top-left
    // Vertex 1: (1, 0) // texture top-right
    // Vertex 2: (0, 1) // texture bottom-left
    // Vertex 3: (1, 1) // texture bottom-right
    let x = f32(i32((vertex_index << 1u) & 2u));
    let y = f32(i32(vertex_index & 2u));
    let texture_uv = vec2<f32>(x, y);

    // SDF flag 0.0 means this object is a textured Sprite.
    // Otherwise, it's a calculated shape from a SDF function.
    if object.sdf_flags == 1.0 {
        
        // Sprite's X and Y position in pixels
        let sprite_position = object.position;

        // Normalizes the sprite's position and size
        let clip_position = (globals.view_projection * sprite_position);

        var scale: vec2<f32>;
        let texture_aspect = object.texture_uv.z / object.texture_uv.w;
        let window_aspect = window.resolution.x / window.resolution.y;

        //@TODO Make this Configurable

        // Full Image
        if texture_aspect > window_aspect {
            scale = vec2<f32>(1.0, window_aspect / texture_aspect);
        } else {
            scale = vec2<f32>(texture_aspect / window_aspect, 1.0);
        };

        // Fullscreen
        // if texture_aspect > window_aspect {
        //     scale = vec2<f32>(window_aspect / texture_aspect, 1.0);
        // } else {
        //     scale = vec2<f32>(1.0, texture_aspect / window_aspect);
        // }

        // the full size of the Sprite's texture in pixels
        let texture_size = object.texture_uv.zw;

        // Sprite's clip region in pixels. Defines the sprite's
        // size and the section of the texture it will display
        let sprite_clip_origin = object.bounds.xy;
        let sprite_clip_size = object.bounds.zw;

        // Normalizes Texture's UV coordinates for the clip region
        let texture_uv_min = sprite_clip_origin / texture_size;
        let texture_uv_max = (sprite_clip_origin + sprite_clip_size) / texture_size;
        let adjusted_uv = mix(texture_uv_min, texture_uv_max, texture_uv);

        let flipped_uv = adjusted_uv * vec2<f32>(1.0, -1.0) + vec2<f32>(0.0, 1.0);

        // adjusted_uv.x - 1.0 (left of the screen: sprite renders fully)
        // adjusted_uv.y + 0.0 (center of the screen)
        // adjusted_uv.x + 1.0 (right of the screen: sprite gets clipped out and disappears)

        // adjusted_uv.y + 0.0 (top of the screen)

        // Adjust vertex positions
        let tx = texture_uv.x * 2.0 - 1.0;
        let ty = texture_uv.y * 2.0 - 1.0;

        let ndc = vec2<f32>(2.0 * texture_uv - vec2<f32>(1.0, 1.0));
        let out = vec4<f32>(ndc, 0.0, 1.0);

        //let out = vec2<f32>(tx, ty) * scale;

        // Fullscreen but stretched
        // let out = 2.0 * texture_uv - vec2<f32>(1.0, 1.0);
        return VertexOutput(
            //
            // from: https://gpuweb.github.io/gpuweb/#clip-volume
            //
            // 23.3.4. Primitive Clipping 
            // Vertex shaders have to produce a built-in position (of type vec4<f32>), 
            // which denotes the clip position of a vertex in clip space coordinates.
            //
            // Clip space coordinates have four dimensions: (x, y, z, w)
            //
            // - Clip space coordinates are used for the the clip position of a vertex
            //   (i.e. the position output of a vertex shader), and for the clip volume.
            // 
            // - Normalized device coordinates and clip space coordinates are related as follows: 
            //
            //      If the clip volume is  p = (p.x,         p.y,         p.z,         p.w),
            //           then its NDC are      (p.x ÷ p.w,   p.y ÷ p.w,   p.z ÷ p.w).

            // vec4<f32>(adjusted_uv.x, adjusted_uv.y, 0.0, 1.0),
            vec4<f32>(out.xy * scale, 0.0, 1.0),

            // UV coordinates are used to sample textures, and have two dimensions (u, v):
            //  -  Both range from 0.0 to 1.0.
            //  -  (0.0, 0.0) is in the first texel in texture memory address order.
            //  -  (1.0, 1.0) is in the last texel texture memory address order.
            vec2<f32>(flipped_uv.x, flipped_uv.y)
        );
    } else {
        // builds a fullscreen quad compatible with Shadertoy
        let out = 2.0 * texture_uv - vec2<f32>(1.0, 1.0);

        return VertexOutput(
            vec4<f32>(out.x, out.y, 0.0, 1.0),
            texture_uv * vec2<f32>(1.0, -1.0) + vec2<f32>(0.0, 1.0)
        );
    }
}

@fragment
fn main_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    if object.sdf_flags == 0.0 {
        // Clamp the texture coordinates
        let texture_color = textureSample(texture, texture_sampler, in.texture_coord);

        // CLAMP_TO_BORDER is not supported in legacy WebGL
        // we have to do it here
        var alpha = 1.0;
        if in.texture_coord.x > 1.0 || in.texture_coord.y > 1.0 || in.texture_coord.x < 0.0 || in.texture_coord.y < 0.0 {
            alpha = 0.0;
        }

        // Premultiply the texture's Alpha
        return texture_color * vec4<f32>(texture_color.aaa, alpha);
    }

    // if object.sdf_flags == 1.0 {
    //     let texture_color = textureSample(texture, texture_sampler, in.texture_coord.xy);
    //     return texture_color * vec4<f32>(texture_color.aaa, 1.0);
    // }

    var distance = 0.0;
    let p = in.frag_coord.xy;           //     / window.resolution;
    let position = object.position.xy;  //     / window.resolution;
    let bounds = object.bounds.zw;      //     / window.resolution;
    let radius = object.radius;              //    / min(window.resolution.x, window.resolution.y);
    let border = object.border;              //     / min(window.resolution.x, window.resolution.y);

    if object.sdf_flags == 1.0 {
        distance = sd_circle(p, position, radius);
    } else if object.sdf_flags == 2.0 {
        distance = sd_rectangle(p, position, bounds);

        // @TODO Rotation is not implemented yet
        //
        // normalizes the bounds
        // let bounds = object.bounds.zw / window.resolution;
        //
        // let width = vec2<f32>(object.position.x + bounds.x, object.position.y);
        // let height = bounds.y;
        //
        // The 'a' and 'b' arguments are the start and end of a line segment the box is built on top of. 
        // The 'th' argument is the width or thickness of the box. 
        // distance = sd_oriented_box(in.frag_coord.xy, object.position.xy, width, height);
    } else if object.sdf_flags == 3.0 {

        // c  : center of box
        // he : half-extents of the box, vec2(width*0.5,height*0.5)
        // u  : orientation vector, sin-cos pair of rotation angle
        // r  : rounding radius
        // s  : stroke width
        let c = object.position.xy;
        let he = object.bounds.zw / 2.0;
        let u = vec2<f32>(cos(0.0), sin(0.0));
        let r = 1.0;
        let s = object.border;
        distance = distance_box(p, c, he, u) - r;

        //distance = sd_segment(in.frag_coord.xy, object.position.xy, object.bounds.zw, 10.0);
    } else {
        return shadertoy(in.frag_coord);
    }

    let aa = window.antialiaser;
    let alpha = (1.0 - smoothstep(border - aa, border + aa, abs(distance))) * object.color.a;

    return vec4<f32>(object.color.rgb, alpha);
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

//
// For all functions below,
// - `p` is the Pixels's (x, y) coordinates

fn sd_circle(p: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    let distance = length(p - center);
    return distance - radius;
}

fn sd_rectangle(p: vec2<f32>, center: vec2<f32>, dimensions: vec2<f32>) -> f32 {
    let size = abs(p - center) - dimensions;
    return length(max(size, vec2<f32>(0.0)) + min(max(size.x, size.y), 0.0));
}

// The 'a' and 'b' arguments are the start and end of a line segment the box is built on top of. 
// The 'th' argument is the width or thickness of the box. 
fn sd_oriented_box(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, th: f32) -> f32 {
    let l = length(b - a);
    let d = (b - a) / l;
    var q = (p - (a + b) * 0.5);
    q = mat2x2(d.x, -d.y, d.y, d.x) * q;
    q = abs(q) - vec2(l, th) * 0.5;
    let dist = length(max(q, vec2<f32>(0.0)));
    return dist + min(max(q.x, q.y), 0.0);
}

fn sd_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let ba = b - a;
    let pa = p - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    let q = pa - h * ba;
    let d = length(q);
    return d - r;
}

fn sd_line(position: vec2<f32>, line_start: vec2<f32>, line_end: vec2<f32>, radius: f32) -> f32 {
    let length = line_end - line_start;
    let pa = position - line_start;
    let h = clamp(dot(pa, length) / dot(length, length), 0.0, 1.0);
    let q = pa - h * length;
    let d = length(q);
    return d - radius;
}


/////////// From: 

fn skew(v: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(-v.y, v.x);
} 

fn distance_aabb(p: vec2<f32>, he: vec2<f32>) -> f32 {
    var d: vec2<f32> = abs(p) - he;
    return length(max(d, vec2<f32>(0.))) + min(max(d.x, d.y), 0.);
} 

fn distance_box(p: vec2<f32>, c: vec2<f32>, he: vec2<f32>, u: vec2<f32>) -> f32 {
    var p_var = p;
    var m: mat2x2<f32> = transpose(mat2x2<f32>(u, skew(u)));
    p_var = p_var - c;
    p_var = m * p_var;
    return distance_aabb(p_var, he);
} 

fn stroke(d: f32, s: f32) -> f32 {
    return abs(d) - s;
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

    if p.y > 0.0 {
        r.x = r.x;
    } else {
        r.x = r.x;
    }

    let q = abs(p) - b + vec2<f32>(r.x, r.x);

    return min(max(q.x, q.y), 0.0) + length(max2(q, splat2(0.0))) - r.x;
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


//
// IT WORKS!!!!!!!!
//
// The Shader below is adapted from ShaderToy.
// I will use it as a test for the shader composition.
//
// @fragment
fn shadertoy(frag_coord: vec4<f32>) -> vec4<f32> {

    // Maybe some pre-processing here

    return shadertoy_main_image(frag_coord);
}

// 
// Paste your ShaderToy code here
//
fn shadertoy_main_image(frag_coord: vec4<f32>) -> vec4<f32> {

    // Assigning to ShaderToy variables for convenience
    let fragCoord = frag_coord.xy;
    let iResolution = window.resolution;
    let iTime = window.time;
    let iTimeDelta = window.frame_delta;
    let iFrameRate = window.fps;
    let iFrame = 0.0;
    // Channels are not supported for now (needs a Sampler public API)
    // Low-priority planned feature.
    // let iChannelTime = [0.0, 0.0, 0.0, 0.0];
    // let iChannelResolution = [vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.0)];
    let iMouse = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let iDate = vec4<f32>(0.0, 0.0, 0.0, 0.0);





    let p = (splat2(2.0) * frag_coord.xy - window.resolution) / splat2(window.resolution.y);

    let si = vec2<f32>(0.9, 0.6);

    let ra = splat4(0.3) + splat4(0.3) * cos(splat4(window.time) * vec4<f32>(0.0, 1.0, 2.0, 3.0));
    let d = sd_round_box(p, si, ra);
    let col = splat3(1.0) - sign(d) * vec3<f32>(0.1, 0.4, 0.7);
    let col1 = col * splat3(1.0) - splat3(exp(-3.0 * abs(d)));
    let col2 = col1 * splat3(0.8) * splat3(0.2) * cos(150.0 * d);
    let col3 = mix3(col2, splat3(1.0), 1.0 - smoothstep(0.0, 0.02, abs(d)));

    let fragColor = vec4<f32>(col3, 1.0);


    // ___________ END OF COMPOSITION PART ___________
    ////////////////////////////////////////////////////////////////////////////

    // We return fragColor with the same name as ShaderToy
    return fragColor;
}
