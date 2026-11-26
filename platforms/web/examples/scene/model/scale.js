import { Mesh, Model, PbrMaterial, Renderer, Vertex } from "fragmentcolor";

const renderer = new Renderer();
const mesh = new Mesh();
mesh.addVertices([ Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]), Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]), Vertex.pbr([0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]), ]);

const model = new Model(mesh, new PbrMaterial());
model.scale([2.0, 2.0, 2.0]);