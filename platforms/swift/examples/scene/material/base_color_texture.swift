import FragmentColor

let renderer = Renderer()
let texture = renderer.createTexture([
    255, 200, 120, 255,
    255,  240, 180, 255,
    230,  180, 100, 255,
    255,  220, 150, 255,
try await ])
let mat = try await Material.pbr(renderer).baseColorTexture(texture)