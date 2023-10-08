// @TODO this global part will be included by the pre-processor
// --------------

// #define_import_path common

/// Common WGSL shared among all PLRender shaders.
/// PLRender prepends this file onto every shader at runtime.


/// GLOBALS: @group(0)

/// Camera projection matrix
struct Camera {
    view_projection: mat4x4<f32>,
};

/// Screen resolution and antialiasing factor
struct Screen {
    resolution: vec2<f32>,
    antialiaser: f32,
    _padding: f32,
};

/// Elapsed time and delta time
// struct Time {
//     elapsed: f32,
//     delta: f32,
// };

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> screen: Screen;
//@group(0) @binding(2) var<uniform> time: Time;


/// LOCALS: @group(1)

/// Transforms that are changed per object.
/// Converts this object into scene space.
struct Transform {
    position: vec4<f32>,
    rotation: vec4<f32>,
    scale: vec4<f32>,
};

// The object properties
struct Object {
    transform: Transform,
    bounds: vec4<f32>,
    color: vec4<f32>,
    radius: f32,
    border: f32,
}

@group(1) @binding(0) var<uniform> object: Object;

/// The vertex format shared among most shaders.
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texture_uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

/// The instance buffer
struct InstanceInput {
    @location(5) transform_matrix_row_0: vec4<f32>,
    @location(6) transform_matrix_row_1: vec4<f32>,
    @location(7) transform_matrix_row_2: vec4<f32>,
    @location(8) transform_matrix_row_3: vec4<f32>,
    @location(9) normal_matrix_row_0: vec3<f32>,
    @location(10) normal_matrix_row_1: vec3<f32>,
    @location(11) normal_matrix_row_2: vec3<f32>,
}

/// Vertex output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
};

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let instance_transform_matrix = mat4x4<f32>(
        instance.transform_matrix_row_0,
        instance.transform_matrix_row_1,
        instance.transform_matrix_row_2,
        instance.transform_matrix_row_3,
    );
    let instamce_normal_matrix = mat3x3<f32>(
        instance.normal_matrix_row_0,
        instance.normal_matrix_row_1,
        instance.normal_matrix_row_2,
    );

    var world_position = instance_transform_matrix * (vec4<f32>(vertex.position, 1.0) + object.transform.position);
    out.position = camera.view_projection * world_position;
    out.world_position = world_position.xyz;
    out.world_normal = instamce_normal_matrix * vertex.normal;
    out.uv = vertex.texture_uv;

    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

struct Circle {
    position: vec2<f32>,
    radius: f32,
    border: f32,
    color: vec4<f32>,
};
@group(1) @binding(0)
var<uniform> circle: Circle;

fn sd_circle(p: vec2<f32>, uv: vec2<f32>, radius: f32) -> f32 {
    let distance = distance(p, uv);
    return distance - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    let aa = screen.antialiaser;

    let radius = circle.radius;
    let border = circle.border;
    var position = circle.position.xy;

    let normalized = in.position.xy / screen.resolution;
    var uv = (normalized * 2.0) - vec2(1.0);

    uv.x *= screen.resolution.x / screen.resolution.y;
    position.x *= screen.resolution.x / screen.resolution.y;

    let dist = distance(uv, position);
    let alpha = (1.0 - smoothstep(border - aa, border + aa, abs(dist - radius))) * circle.color.a;

    out.color = vec4<f32>(circle.color.rgb, alpha);

    return out;
}
