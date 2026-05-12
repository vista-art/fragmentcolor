import FragmentColor

let renderer = Renderer()
let normal_map = renderer.createTexture([
    128, 128, 255, 255,
    128,   128, 255, 255,
    128,   128, 255, 255,
    128,   128, 255, 255,
try await ])
let mat = Material.pbr().normalTexture(normal_map).normalScale(1.2)