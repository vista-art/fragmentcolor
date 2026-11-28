import FragmentColor

let renderer = Renderer()
let lava = try PbrMaterial().baseColor([0.1, 0.05, 0.0, 1.0]).emissive([1.5, 0.4, 0.1])