import { Material, Mesh, Model, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertices([ Vertex.pbr([0.0, 0.5, 0.0]), Vertex.pbr([-0.5, -0.5, 0.0]), Vertex.pbr([0.5, -0.5, 0.0]), ]);
const model = new Model(mesh, Material.pbr());

// Models start visible; toggle with `set_visible`.
const visible_now = model.visible();