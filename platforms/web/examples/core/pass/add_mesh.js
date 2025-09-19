import { Pass, Shader } from "fragmentcolor";
import { {Mesh, Vertex}, Shader } from "fragmentcolor";

const shader = Shader.default();
const pass = new Pass("p"); pass.addShader(shader);
const mesh = new Mesh();
mesh.addVertex(Vertex.from([0.0, 0.0]));
pass.addMesh(mesh);