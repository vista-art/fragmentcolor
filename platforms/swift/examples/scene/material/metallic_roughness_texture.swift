import FragmentColor

let renderer = Renderer()
let mr_map_bytes = [
    0, 200, 50, 255,
    0,   240, 80, 255,
    0,   180, 30, 255,
    0,   220, 60, 255,
]
let mr_map = try await renderer.createTexture((mr_map_bytes, [2, 2]))
let mat = Material.pbr().metallicRoughnessTexture(mr_map)