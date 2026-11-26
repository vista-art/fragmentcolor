import { Mesh, Model, ToonMaterial, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertices([ Vertex.pbr([0.0, 0.5, 0.0]), Vertex.pbr([-0.5, -0.5, 0.0]), Vertex.pbr([0.5, -0.5, 0.0]), ]);

const hero = new ToonMaterial().baseColor([0.9, 0.3, 0.2, 1.0]).bands(2).highlight([1.0, 1.0, 1.0]).rim([0.3, 0.5, 1.0]);
const model = new Model(mesh, hero);