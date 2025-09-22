from fragmentcolor import Mesh, Vertex
m = Mesh()
m.add_vertices([
    Vertex([-0.01, -0.01]),
    Vertex([ 0.01, -0.01]),
    Vertex([ 0.00,  0.01]),
])
# draw one million instances
m.set_instance_count(1_000_000)