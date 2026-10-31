import { Mesh, Shader } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertices([ [-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0], ]);
const shader = Shader.fromMesh(mesh);
