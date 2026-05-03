from fragmentcolor import Renderer, TextureFormat, TextureMipChain

renderer = Renderer()
# Minimal 1x1 RGBA raw pixel bytes.
pixels = [255, 0, 0, 255]
chain = TextureMipChain.prepare(pixels, TextureFormat.Rgba8UnormSrgb, [1, 1])

# Hand the chain to the unified create_texture entry - same vocabulary as
# every other texture path; From<TextureMipChain> selects the GPU-only
# upload internally.
texture = renderer.create_texture(chain)
