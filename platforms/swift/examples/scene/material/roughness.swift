import FragmentColor

let renderer = Renderer()
let satin = try await Material.pbr(renderer).roughness(0.35)