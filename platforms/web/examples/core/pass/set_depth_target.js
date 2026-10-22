import { Renderer, Pass, Shader, Mesh } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64, 64]);

// One depth attachment shared across the 3D-content pass.
const depth = await renderer.createDepthTexture([64, 64]);

const mesh = new Mesh();
mesh.addVertex([0.0, 0.0, 0.0]);
mesh.addVertex([1.0, 0.0, 0.0]);
mesh.addVertex([0.0, 1.0, 0.0]);
mesh.addVertex([1.0, 1.0, 0.0]);
const shader = Shader.fromMesh(mesh);
const pass = Pass.fromShader("blobs", shader);

// Depth test on: closer fragments win, and the pass writes the depth
// buffer, so later draws in the same pass see it.
pass.setDepthTarget(depth);

renderer.render(pass, target);