import FragmentColor

let renderer = Renderer()
let ao = renderer.createTexture([
    220, 0, 0, 255,
    180,   0, 0, 255,
    200,   0, 0, 255,
    160,   0, 0, 255,
try await ])
let mat = try await Material.pbr(renderer).occlusionTexture(ao)