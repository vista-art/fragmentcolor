from fragmentcolor import Pass, Renderer

renderer = Renderer()
depth = renderer.create_depth_texture([800, 600])
rpass = Pass("scene")
rpass.set_depth_target(depth)

# The window grew. The pass keeps the same depth texture at the new size.
depth.resize([1024, 768])
