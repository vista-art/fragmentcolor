from fragmentcolor import {Mesh, Vertex}
m = Mesh()
m.add_vertices([
    Vertex.new([-0.01, -0.01]),
    Vertex.new([ 0.01, -0.01]),
    Vertex.new([ 0.00,  0.01]),
])
# draw one million instances
m.set_instance_count(1_000_000)