from fragmentcolor import Frame, Pass

shadow = Pass("shadow")
main = Pass("main")

frame = Frame()
frame.add_pass(shadow)
frame.add_pass(main)
main.require(shadow)

# Main pass can present
frame.present(main)
from fragmentcolor import Frame, Pass

geometry = Pass("geometry")
lighting = Pass("lighting")
post_fx = Pass("post_effects")

frame = Frame()
frame.add_pass(geometry)
frame.add_pass(lighting)
frame.add_pass(post_fx)

# Build pipeline using Pass.require
lighting.require(geometry)
post_fx.require(lighting)

# Final post-effects pass presents to screen
frame.present(post_fx)