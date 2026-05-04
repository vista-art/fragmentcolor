
from fragmentcolor import Shader

# Single URL
shader = Shader.fetch("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl")

# Registry slug
shader2 = Shader.fetch("sdf2d/circle")
