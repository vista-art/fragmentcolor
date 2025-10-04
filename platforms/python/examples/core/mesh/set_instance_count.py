from fragmentcolor import Mesh
m = Mesh()
m.add_vertices([
    [-0.01, -0.01],
    [ 0.01, -0.01],
    [ 0.00,  0.01],
])
# draw one million instances
m.set_instance_count(1_000_000)