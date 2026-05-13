import FragmentColor

let renderer = Renderer()
let red = try await Material.pbr(renderer).baseColor([1.0, 0.2, 0.2, 1.0])