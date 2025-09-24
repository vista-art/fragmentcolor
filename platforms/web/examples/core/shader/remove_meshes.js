import { Pass, Shader, Mesh, Vertex } from "fragmentcolor";

const shader = Shader.default();
const pass = new Pass("p"); pass.addShader(shader);

const m1 = new Mesh();
m1.addVertex(Vertex.new([0.0, 0.0]));
const m2 = new Mesh();
m2.addVertex(Vertex.new([0.5, 0.0]));

shader.addMesh(m1);
shader.addMesh(m2);

shader.removeMeshes([m1, m2]);