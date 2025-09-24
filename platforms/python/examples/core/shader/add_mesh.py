from fragmentcolor import Pass, Shader, Mesh, Vertex

shader = Shader.default()
rpass = Pass("p"); rpass.add_shader(shader)

mesh = Mesh()
mesh.add_vertex(Vertex([0.0, 0.0]))

# Attach mesh to this shader
shader.add_mesh(mesh)

# Renderer will draw the mesh when rendering this rpass.
# Each Shader represents a RenderPipeline or ComputePipeline
# in the GPU. Adding multiple meshes to it will draw all meshes
# and all its instances in the same Pipeline.