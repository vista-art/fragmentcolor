import { Mesh, Model, UnlitMaterial, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertices([ Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]), Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]), Vertex.pbr([0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]), ]);

const badge = new UnlitMaterial().baseColor([1.0, 0.8, 0.1, 1.0]);
const model = new Model(mesh, badge);