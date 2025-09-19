from fragmentcolor import {Mesh, Vertex, Position}
m = Mesh()
m.add_instances([
  Vertex.from_position(Position.Pos2([0.0, 0.0])),
  Vertex.from_position(Position.Pos2([1.0, 1.0])),
])