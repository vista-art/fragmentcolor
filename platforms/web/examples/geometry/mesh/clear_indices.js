import { Mesh, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertices([ Vertex.new([-0.5, -0.5]), Vertex.new([ 0.5, -0.5]), Vertex.new([ 0.0,  0.5]), ]);
mesh.setIndices([0, 1, 2]);
mesh.clearIndices(); // back to auto-derived dedup;