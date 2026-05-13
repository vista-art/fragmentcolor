import FragmentColor

let renderer = Renderer()
let lava = try await Material.pbr(renderer).baseColor([0.1, 0.05, 0.0, 1.0]).emissive([1.5, 0.4, 0.1])