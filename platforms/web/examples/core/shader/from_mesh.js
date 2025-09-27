import { Mesh, Shader, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertex(Vertex.new([0.0, 0.0, 0.0]));
const shader = Shader.fromMesh(mesh);
