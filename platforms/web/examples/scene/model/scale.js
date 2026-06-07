import { Material, Mesh, Model, Renderer, Vertex } from "fragmentcolor";

const renderer = new Renderer();
const mesh = new Mesh();
mesh.addVertex( new Vertex([0.0, 0.0, 0.0]) .set("normal", [0.0, 1.0, 0.0]) .set("uv0", [0.0, 0.0]), );

const model = new Model(mesh, Material.pbr());
model.scale([2.0, 2.0, 2.0]);