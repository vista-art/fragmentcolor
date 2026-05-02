import org.fragmentcolor.*

val renderer = Renderer()
// Encoded image bytes the caller has on hand (could come off a worker).
val png = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
    // ... full PNG body ...
]
val chain = TextureMipChain.prepare((png, TextureFormat.Rgba8UnormSrgb))

// Hand the chain to the unified create_texture entry - same vocabulary as
// every other texture path; From<TextureMipChain> selects the GPU-only
// upload internally.
val texture = renderer.createTexture(chain)