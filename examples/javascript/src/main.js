import "./style.css";
import init, { Renderer, Shader } from "fragmentcolor";

await init();

let canvas = document.getElementById("my-canvas");
const resolution = [canvas.width, canvas.height];
let renderer = new Renderer();
let target = await renderer.create_target(canvas);

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

    // necessary for multipass rendering
    if (dist > r + (border + aa)) {
        discard;
    }

    let circle_sdf = 1.0 - smoothstep(border - aa, border + aa, abs(dist - r));
    return vec4<f32>(circle.color.rgb * circle.color.a, circle.color.a) * circle_sdf;
}
`);

// The library parses the uniforms automatically and exposes their names as keys
shader.set("resolution", resolution);
shader.set("circle.radius", 200.0);
shader.set("circle.color", [1.0, 0.0, 0.0, 0.8]);
shader.set("circle.border", 20.0);

function animate() {
  shader.set("circle.position", [0.0, 0.0]);
  renderer.render(shader, target);

  requestAnimationFrame(animate);
}
animate();
