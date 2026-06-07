from fragmentcolor import Mesh, Vertex

# Build a triangle; override only what the mesh actually carries — NORMAL
# / COLOR0 / UV1 / TANGENT use their identity defaults from Vertex.pbr.
mesh = Mesh()
mesh.add_vertex(Vertex.pbr([ 0.0,  0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]))
mesh.add_vertex(Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]))
mesh.add_vertex(Vertex.pbr([ 0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]))