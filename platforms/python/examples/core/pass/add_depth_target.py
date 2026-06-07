from fragmentcolor import Renderer, Pass, Shader, Mesh

renderer = Renderer()
target = renderer.create_texture_target([64, 64])

# One depth attachment shared across the 3D-content pass.
depth = renderer.create_depth_texture([64, 64])

mesh = Mesh()
mesh.add_vertex([0.0, 0.0, 0.0])
mesh.add_vertex([1.0, 0.0, 0.0])
mesh.add_vertex([0.0, 1.0, 0.0])
mesh.add_vertex([1.0, 1.0, 0.0])
shader = Shader.from_mesh(mesh)
rpass = Pass("blobs"); rpass.add_shader(shader)

# Depth-test on — closer fragments win, the rpass writes to the depth
# buffer so subsequent draws within the same rpass see the depth.
rpass.add_depth_target(depth)

renderer.render(rpass, target)