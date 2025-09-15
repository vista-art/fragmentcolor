
from fragmentcolor import Renderer, Shader
renderer = Renderer()
shader = Shader.default()

bytes = std.fs.read("./examples/assets/image.png").unwrap()
texture = renderer.create_texture(bytes)

shader.set("texture", texture).unwrap()
