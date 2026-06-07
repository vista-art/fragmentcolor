import { Mesh, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertices([ new Vertex([-0.5, -0.5]), new Vertex([ 0.5, -0.5]), new Vertex([ 0.0,  0.5]), ]);
mesh.setIndices([0, 1, 2]);
mesh.clearIndices(); // back to auto-derived dedup;