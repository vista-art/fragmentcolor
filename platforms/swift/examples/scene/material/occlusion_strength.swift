import FragmentColor

let renderer = Renderer()
let crevices = try await Material.pbr(renderer).occlusionStrength(0.8)