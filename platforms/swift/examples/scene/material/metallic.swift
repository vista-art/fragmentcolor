import FragmentColor

let renderer = Renderer()
let chrome = try await Material.pbr(renderer).metallic(1.0).roughness(0.05)