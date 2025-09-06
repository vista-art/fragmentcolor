import init, { Renderer, Shader, Pass, Frame } from "../pkg/fragmentcolor.js";

await init();

// DOC: Renderer.constructor (begin)
const renderer = new Renderer();
// DOC: (end)
// DOC: Renderer.create_texture_target (begin)
const target = await renderer.createTextureTarget([64, 64]);
// DOC: (end)

// DOC: Shader.constructor (begin)
const shader = new Shader(`
struct VertexOutput {
    @builtin(position) coords: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    const vertices = array(
        vec2( -1., -1.),
        vec2(  3., -1.),
        vec2( -1.,  3.)
    );
    return VertexOutput(vec4<f32>(vertices[in_vertex_index], 0.0, 1.0));
}

struct Circle {
    position: vec2<f32>,
    radius: f32,
    border: f32,
    color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> circle: Circle;

@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@fragment
fn main(pixel: VertexOutput) -> @location(0) vec4<f32> {
    let normalized_coords = pixel.coords.xy / resolution;
    var uv = -1.0 + 2.0 * normalized_coords;
    if (resolution.x > resolution.y) {
        uv.x *= resolution.x / resolution.y;
    } else {
        uv.y *= resolution.y / resolution.x;
    }
    let circle_pos = circle.position / resolution;
    let dist = distance(uv, circle_pos);
    let r = circle.radius / min(resolution.x, resolution.y);
    let aa = 2. / min(resolution.x, resolution.y);
    let border = circle.border / min(resolution.x, resolution.y);

    if (dist > r + (border + aa)) { discard; }

    let circle_sdf = 1.0 - smoothstep(border - aa, border + aa, abs(dist - r));
    let a = circle.color.a * circle_sdf;
    return vec4<f32>(circle.color.rgb * a, a);
}
`);
// DOC: (end)

// DOC: Shader.set (begin)
shader.set("resolution", [64.0, 64.0]);
shader.set("circle.radius", 10.0);
shader.set("circle.color", [1.0, 0.0, 0.0, 0.8]);
shader.set("circle.border", 2.0);
shader.set("circle.position", [0.0, 0.0]);
// DOC: (end)

// DOC: Renderer.render (begin)
renderer.render(shader, target);
// DOC: (end)

// DOC: Pass.constructor (begin)
const rpass = new Pass("single pass");
// DOC: (end)
// DOC: Pass.add_shader (begin)
rpass.add_shader(shader);
// DOC: (end)
renderer.render(rpass, target);

// DOC: Frame.constructor (begin)
const frame = new Frame();
// DOC: (end)
// DOC: Frame.add_pass (begin)
frame.add_pass(rpass);
// DOC: (end)
renderer.render(frame, target);

// Additional API coverage for docs
// DOC: Shader.get (begin)
const _radius = shader.get("circle.radius");
// DOC: (end)
// DOC: Shader.list_uniforms (begin)
const _uniforms = shader.listUniforms();
// DOC: (end)
// DOC: Shader.list_keys (begin)
const _keys = shader.listKeys();
// DOC: (end)

console.log("Headless JS render completed successfully");
