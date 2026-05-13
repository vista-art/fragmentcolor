import FragmentColor

let renderer = Renderer()
let foliage = try await Material.pbr(renderer).alphaCutoff(0.3)