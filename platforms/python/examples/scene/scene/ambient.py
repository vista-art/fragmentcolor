from fragmentcolor import Light, Material, Mesh, Model, Renderer, Scene, Vertex

renderer = Renderer()
target = renderer.create_texture_target([64, 64])

mesh = Mesh()
mesh.add_vertex(
    Vertex([0.0, 0.5, 0.0]).set(Vertex.NORMAL, [0.0, 0.0, 1.0]).set(Vertex.UV0, [0.5, 1.0]).set(Vertex.COLOR0, [1.0, 1.0, 1.0, 1.0]).set(Vertex.UV1, [0.0, 0.0]),
)

scene = Scene()
# Warm dusk ambient — applies to every Material added below.
scene.ambient([0.06, 0.04, 0.03])
scene.add(Model(mesh, Material.pbr()))
scene.add(Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]))

renderer.render(scene, target)