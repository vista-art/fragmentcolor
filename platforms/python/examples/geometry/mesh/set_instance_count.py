from fragmentcolor import Mesh
m = Mesh()
m.add_vertices([
    [-0.01, -0.01],
    [ 0.01, -0.01],
    [ 0.00,  0.01],
])
# Draw one million instances, fetching per-particle data from a storage buffer.
m.set_instance_count(1_000_000)