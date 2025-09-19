import { Pass, Shader } from "fragmentcolor";
import { {Mesh, Vertex, Position}, Shader } from "fragmentcolor";

const shader = Shader.default();
const pass = new Pass("p"); pass.addShader(shader);
const mesh = new Mesh();
mesh.addVertex(Vertex.fromPosition(Position.Pos2([0.0, 0.0])));
pass.addMesh(mesh);