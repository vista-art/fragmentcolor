from fragmentcolor import Material, Mesh, TextureSlot, Vertex

mesh = Mesh()
mesh.add_vertices([
    Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]).set(Vertex.UV1, [0.25, 0.75]),
    Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]).set(Vertex.UV1, [0.0, 0.5]),
    Vertex.pbr([0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]).set(Vertex.UV1, [0.5, 0.5]),
])

# The occlusion map follows the second UV layout.
material = Material.pbr().texture_uv(TextureSlot.Occlusion, 1)