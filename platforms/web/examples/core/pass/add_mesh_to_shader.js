import { Shader, Mesh, Vertex, Pass } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertex([0.0, 0.0]);
const shader = Shader.fromMesh(mesh);
const pass = new Pass("pass"); pass.addShader(shader);

pass.addMeshToShader(mesh, shader);
