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
const pass = new Pass("blobs"); pass.addShader(shader);

// Depth-test on — closer fragments win, the pass writes to the depth
// buffer so subsequent draws within the same pass see the depth.
pass.addDepthTarget(depth);

renderer.render(pass, target);