import { Mesh, Vertex } from "fragmentcolor";

// A quad split into two triangles via explicit indexing. The four corners
// happen to carry distinct UVs (only positions repeat), so we keep them
// all and reference each by index.
const mesh = new Mesh();
const uv00 = [0.0, 0.0];
const uv10 = [1.0, 0.0];
const uv11 = [1.0, 1.0];
const uv01 = [0.0, 1.0];
mesh.addVertices([ new Vertex([-0.5, -0.5]).set("uv", uv00), new Vertex([ 0.5, -0.5]).set("uv", uv10), new Vertex([ 0.5,  0.5]).set("uv", uv11), new Vertex([-0.5,  0.5]).set("uv", uv01), ]);
mesh.setIndices([0, 1, 2, 0, 2, 3]);