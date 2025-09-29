from fragmentcolor import Frame, Pass

shadow = Pass("shadow")
main = Pass("main")

frame = Frame()
frame.add_pass(shadow)
frame.add_pass(main)
frame.connect(shadow, main)

# Main pass is a leaf, so it can present
frame.present(main)
from fragmentcolor import Frame, Pass

geometry = Pass("geometry")
lighting = Pass("lighting")
post_fx = Pass("post_effects")

frame = Frame()
frame.add_pass(geometry)
frame.add_pass(lighting)
frame.add_pass(post_fx)

# Build pipeline
frame.connect(geometry, lighting)
frame.connect(lighting, post_fx)

# Final post-effects pass presents to screen
frame.present(post_fx)