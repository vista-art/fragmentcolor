import FragmentColor

let renderer = Renderer()
let detailed = try await Material.pbr(renderer).normalScale(1.5)