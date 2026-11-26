from fragmentcolor import Mesh, Model, PbrMaterial, Renderer, Vertex

renderer = Renderer()
mesh = Mesh()
mesh.add_vertices([
    Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
    Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]),
    Vertex.pbr([0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]),
])

model = Model(mesh, PbrMaterial())
model.rotate([0.0, 1.0, 0.0], 1.571) # 90° around Y