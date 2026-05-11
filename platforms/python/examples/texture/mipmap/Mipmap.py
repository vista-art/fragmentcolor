from fragmentcolor import Renderer, TextureFormat, Mipmap

renderer = Renderer()
# Minimal 1x1 RGBA raw pixel bytes.
pixels = [255, 0, 0, 255]
chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, [1, 1])

# Hand the chain to the unified create_texture entry.
texture = renderer.create_texture(chain)