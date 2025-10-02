from fragmentcolor import Renderer, Pass, Shader, Mesh

renderer = Renderer()
target = renderer.create_texture_target([64, 64])

# Create a depth texture usable as a per-pass attachment
depth = renderer.create_depth_texture([64, 64])

mesh = Mesh()
mesh.add_vertex([0.0, 0.0, 0.0])
mesh.add_vertex([1.0, 0.0, 0.0])
mesh.add_vertex([0.0, 1.0, 0.0])
mesh.add_vertex([1.0, 1.0, 0.0])
shader = Shader.from_mesh(mesh)
rpass = Pass("scene"); rpass.add_shader(shader)

# Attach depth texture to enable depth testing.
# Pipeline will include a matching depth-stencil state
rpass.add_depth_target(depth)

# Render as usual
renderer.render(rpass, target)