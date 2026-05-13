import FragmentColor

let renderer = Renderer()
let glow = renderer.createTexture([
    255, 0, 0, 255,
    255,   0, 0, 255,
    255,   0, 0, 255,
    255,   0, 0, 255,
try await ])
let mat = Material.pbr()?.emissive([0.8, 0.0, 0.0]).emissiveTexture(glow)