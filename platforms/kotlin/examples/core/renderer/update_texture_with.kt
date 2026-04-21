import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture([64, 32], TextureFormat.Rgba, null)
val id = *texture.id()
val frame = [0u8; 64 * 32 * 4]
val opt = TextureWriteOptions.whole().withBytesPerRow(256).withRowsPerImage(32)

renderer.updateTextureWith(id, frame, opt)