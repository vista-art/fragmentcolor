import FragmentColor

let renderer = Renderer()
let depth = try await renderer.createDepthTexture([800, 600])
let pass = Pass("scene")
try pass.setDepthTarget(depth)

// The window grew. The pass keeps the same depth texture at the new size.
try depth.resize([1024, 768])