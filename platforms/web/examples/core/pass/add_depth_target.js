import { Renderer, Pass, Shader, Mesh } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64, 64]);

// Create a depth texture usable as a per-pass attachment
const depth = await renderer.createDepthTexture([64, 64]);

const mesh = new Mesh();
mesh.addVertex([0.0, 0.0, 0.0]);
mesh.addVertex([1.0, 0.0, 0.0]);
mesh.addVertex([0.0, 1.0, 0.0]);
mesh.addVertex([1.0, 1.0, 0.0]);
const shader = Shader.fromMesh(mesh);
const pass = new Pass("scene"); pass.addShader(shader);

// Attach depth texture to enable depth testing.
// Pipeline will include a matching depth-stencil state
pass.addDepthTarget(depth);

// Render as usual
renderer.render(pass, target);