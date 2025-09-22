import { Pass, Shader } from "fragmentcolor";
import { Mesh, Vertex } from "fragmentcolor";

const shader = Shader.default();
const pass = new Pass("p"); pass.addShader(shader);
const mesh = new Mesh();
mesh.addVertex(Vertex.new([0.0, 0.0]));
pass.addMesh(mesh);