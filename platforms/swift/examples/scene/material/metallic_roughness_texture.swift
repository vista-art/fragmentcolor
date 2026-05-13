import FragmentColor

let renderer = Renderer()
let mr_map = renderer.createTexture([
    0, 200, 50, 255,
    0,   240, 80, 255,
    0,   180, 30, 255,
    0,   220, 60, 255,
try await ])
let mat = try await Material.pbr(renderer).metallicRoughnessTexture(mr_map)