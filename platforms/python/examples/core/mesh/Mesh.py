from fragmentcolor import {Mesh, Vertex, Position, VertexValue}

mesh = Mesh()
mesh.add_vertex(Vertex.from_position(Position.Pos3([0.0, 0.5, 0.0])))
mesh.add_vertex(Vertex.from_position(Position.Pos3([-0.5, -0.5, 0.0])))
mesh.add_vertex(Vertex.from_position(Position.Pos3([0.5, -0.5, 0.0])))