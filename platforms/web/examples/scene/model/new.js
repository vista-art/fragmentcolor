import { Material, Mesh, Model, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertex( new Vertex([0.0, 0.0, 0.0]) .set("normal", [0.0, 1.0, 0.0]) .set("uv0", [0.0, 0.0]), );

const model = new Model(mesh, Material.pbr());
