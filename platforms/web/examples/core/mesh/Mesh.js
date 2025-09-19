import { {Mesh, Vertex, Position, VertexValue} } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertex(Vertex.fromPosition(Position.Pos3([0.0, 0.5, 0.0])));
mesh.addVertex(Vertex.fromPosition(Position.Pos3([-0.5, -0.5, 0.0])));
mesh.addVertex(Vertex.fromPosition(Position.Pos3([0.5, -0.5, 0.0])));