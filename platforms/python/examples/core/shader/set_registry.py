
from fragmentcolor import Shader

# Point at your own mirror of the registry
Shader.set_registry("https://cdn.example.com/shaders/")

# Now `sdf2d/circle` resolves to https://cdn.example.com/shaders/sdf2d/circle.wgsl
# (Skipping the actual fetch in this doctest)