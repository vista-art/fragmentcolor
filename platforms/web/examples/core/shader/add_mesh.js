import { Pass, Shader, Mesh, Vertex } from "fragmentcolor";

const shader = Shader.default();
const pass = new Pass("p"); pass.addShader(shader);

const mesh = new Mesh();
mesh.addVertex(Vertex.new([0.0, 0.0]));

// Attach mesh to this shader (errors if incompatible);
shader.addMesh(mesh).expect("mesh is compatible");

// Renderer will draw the mesh when rendering this pass.;
// Each Shader represents a RenderPipeline or ComputePipeline;
// in the GPU. Adding multiple meshes to it will draw all meshes;
// and all its instances in the same Pipeline.;