import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture([64, 32], TextureFormat.Rgba, null)
val region_bytes = [0u8; 64 * 32 * 4]
val opt = TextureWriteOptions.whole().withBytesPerRow(256).withRowsPerImage(32)
texture.writeWith(region_bytes, opt)