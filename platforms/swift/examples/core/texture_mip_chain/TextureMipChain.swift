import FragmentColor

let renderer = Renderer()
// Encoded image bytes the caller has on hand (could come off a worker).
let png = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
    // ... full PNG body ...
]
let chain = TextureMipChain.prepare((png, TextureFormat.rgba8UnormSrgb))

// Hand the chain to the unified create_texture entry - same vocabulary as
// every other texture path; From<TextureMipChain> selects the GPU-only
// upload internally.
let texture = try await renderer.createTexture(chain)