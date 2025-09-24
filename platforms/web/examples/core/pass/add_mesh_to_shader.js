import { Pass, Shader, Mesh, Vertex } from "fragmentcolor";

const shader = Shader.default();
const pass = new Pass("p"); pass.addShader(shader);

const mesh = new Mesh();
mesh.addVertex(Vertex.new([0.0, 0.0]));

pass.addMeshToShader(mesh, shader);