import { Mesh, Model, PbrMaterial, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertices([ Vertex.pbr([0.0, 0.5, 0.0]), Vertex.pbr([-0.5, -0.5, 0.0]), Vertex.pbr([0.5, -0.5, 0.0]), ]);
const detail = new Model(mesh, new PbrMaterial());

// Far from the camera, hide the detail Model.
detail.setVisible(false);
// Closer again, show it.
detail.setVisible(true);