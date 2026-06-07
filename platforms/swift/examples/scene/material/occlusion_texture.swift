import FragmentColor

let renderer = Renderer()
let ao_bytes = [
    220, 0, 0, 255,
    180,   0, 0, 255,
    200,   0, 0, 255,
    160,   0, 0, 255,
]
let ao = try await renderer.createTexture((ao_bytes, [2, 2]))
let mat = Material.pbr().occlusionTexture(ao)