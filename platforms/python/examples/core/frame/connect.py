from fragmentcolor import Frame, Pass

# Example 1: simple dependency via require (lighting depends on depth)
depth_pass = Pass("depth_prepass")
lighting_pass = Pass("lighting").require(depth_pass)

frame1 = Frame()
frame1.add_pass(depth_pass)
frame1.add_pass(lighting_pass)

# Example 2: chain using require
geometry_pass = Pass("geometry")
shadow_pass = Pass("shadows").require(geometry_pass)
lighting_pass2 = Pass("lighting").require(shadow_pass)
post_process = Pass("post_processing").require(lighting_pass2)

frame2 = Frame()
frame2.add_pass(geometry_pass)
frame2.add_pass(shadow_pass)
frame2.add_pass(lighting_pass2)
frame2.add_pass(post_process)
