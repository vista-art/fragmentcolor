import { Pass, Shader, Mesh, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertex(Vertex.new([0.0, 0.0]));

const shader = Shader.fromMesh(mesh);
const pass = new Pass("pass"); pass.addShader(shader);

pass.addMesh(mesh).expect("mesh is compatible");