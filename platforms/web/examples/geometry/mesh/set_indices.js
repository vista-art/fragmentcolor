import { Mesh, Vertex } from "fragmentcolor";

// A quad split into two triangles via explicit indexing. The four corners
// happen to carry distinct UVs (only positions repeat), so we keep them
// all and reference each by index.
const mesh = new Mesh();
mesh.addVertices([ new Vertex([-0.5, -0.5]).set("uv", [0.0, 0.0]), new Vertex([ 0.5, -0.5]).set("uv", [1.0, 0.0]), new Vertex([ 0.5,  0.5]).set("uv", [1.0, 1.0]), new Vertex([-0.5,  0.5]).set("uv", [0.0, 1.0]), ]);
mesh.setIndices([0, 1, 2, 0, 2, 3]);