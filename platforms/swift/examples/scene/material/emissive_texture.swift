import FragmentColor

let renderer = Renderer()
let glow_bytes = [
    255, 0, 0, 255,
    255,   0, 0, 255,
    255,   0, 0, 255,
    255,   0, 0, 255,
]
let glow = try await renderer.createTexture((glow_bytes, [2, 2]))
let mat = try Material.pbr().emissive([0.8, 0.0, 0.0]).emissiveTexture(glow)