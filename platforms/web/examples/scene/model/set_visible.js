import { Material, Mesh, Model, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertices([ Vertex.pbr([0.0, 0.5, 0.0]), Vertex.pbr([-0.5, -0.5, 0.0]), Vertex.pbr([0.5, -0.5, 0.0]), ]);
const detail = new Model(mesh, Material.pbr());

// Far from the camera, hide the detail Model.
detail.setVisible(false);
// Closer again, show it.
detail.setVisible(true);