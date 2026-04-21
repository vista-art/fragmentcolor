import FragmentColor
let renderer = Renderer()
let texture = try await renderer.createStorageTexture([64, 32], TextureFormat.Rgba, nil)
let id = *texture.id()
let frame = [0u8; 64 * 32 * 4]
let opt = TextureWriteOptions.whole().withBytesPerRow(256).withRowsPerImage(32)

renderer.updateTextureWith(id, frame, opt)