from fragmentcolor import Renderer, TextureFormat, TextureWriteOptions

renderer = Renderer()
texture = renderer.create_storage_texture([64, 32], TextureFormat.Rgba, None)
id = texture.id()
frame = bytes(64 * 32 * 4)
opt = TextureWriteOptions.whole().with_bytes_per_row(256).with_rows_per_image(32)

renderer.update_texture_with(id, frame, opt)