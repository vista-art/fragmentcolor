from fragmentcolor import Frame, Pass

depth_pass = Pass("depth_prepass")
lighting_pass = Pass("lighting")

frame = Frame()
frame.add_pass(depth_pass)
frame.add_pass(lighting_pass)

# Ensure depth prepass runs before lighting
frame.connect(depth_pass, lighting_pass)
from fragmentcolor import Frame, Pass

geometry_pass = Pass("geometry")
shadow_pass = Pass("shadows")
lighting_pass = Pass("lighting")
post_process = Pass("post_processing")

frame = Frame()

# Add all passes
frame.add_pass(geometry_pass)
frame.add_pass(shadow_pass)
frame.add_pass(lighting_pass)
frame.add_pass(post_process)

# Build dependency chain
frame.connect(geometry_pass, shadow_pass)
frame.connect(shadow_pass, lighting_pass)
frame.connect(lighting_pass, post_process)