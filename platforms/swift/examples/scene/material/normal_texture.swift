import FragmentColor

let renderer = Renderer()
let normal_map_bytes = [
    128, 128, 255, 255,
    128,   128, 255, 255,
    128,   128, 255, 255,
    128,   128, 255, 255,
]
let normal_map = try await renderer.createTexture((normal_map_bytes, [2, 2]))
let mat = Material.pbr().normalTexture(normal_map).normalScale(1.2)