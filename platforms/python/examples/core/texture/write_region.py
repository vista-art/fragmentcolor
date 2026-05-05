from fragmentcolor import Renderer, TextureFormat

renderer = Renderer()
texture = renderer.create_storage_texture([64, 32], TextureFormat.Rgba)
bytes_data = bytes(64 * 32 * 4)

# Simple sub-rectangle update.
texture.write_region(bytes_data, [0, 0, 64, 32])

# Explicit data layout (advanced).
texture.write_region(bytes_data, {"x": 0, "y": 0, "width": 64, "height": 32, "bytes_per_row": 256, "rows_per_image": 32})