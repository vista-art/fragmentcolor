from fragmentcolor import Shader

# Full registry URL.
shader = Shader.fetch("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl")

# Equivalent shorthand using the registry slug.
shader2 = Shader.fetch("sdf2d/circle")