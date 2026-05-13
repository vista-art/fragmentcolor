import FragmentColor

let renderer = Renderer()
let foliage = try await Material.pbr(renderer).alphaMode(AlphaMode.mask).alphaCutoff(0.3)

let glass = try await Material.pbr(renderer).baseColor([0.9, 0.95, 1.0, 0.25]).alphaMode(AlphaMode.blend)

let solid = try await Material.pbr(renderer).alphaMode(AlphaMode.opaque)