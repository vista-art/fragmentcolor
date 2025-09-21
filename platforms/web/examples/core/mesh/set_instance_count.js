import { {Mesh, Vertex} } from "fragmentcolor";
const m = new Mesh();
m.addVertices([;
    Vertex.new([-0.01, -0.01]),;
    Vertex.new([ 0.01, -0.01]),;
    Vertex.new([ 0.00,  0.01]),;
]);
// draw one million instances;
m.setInstanceCount(1_000_000);