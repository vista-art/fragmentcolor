from fragmentcolor import Mesh, Model, ToonMaterial, Vertex

mesh = Mesh()
mesh.add_vertices([
    Vertex.pbr([0.0, 0.5, 0.0]),
    Vertex.pbr([-0.5, -0.5, 0.0]),
    Vertex.pbr([0.5, -0.5, 0.0]),
])

hero = ToonMaterial().base_color([0.9, 0.3, 0.2, 1.0]).bands(2).highlight([1.0, 1.0, 1.0]).rim([0.3, 0.5, 1.0])
model = Model(mesh, hero)