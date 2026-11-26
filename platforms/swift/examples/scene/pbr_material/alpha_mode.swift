import FragmentColor

let foliage = try PbrMaterial().alphaMode(AlphaMode.mask).alphaCutoff(0.3)

let glass = try PbrMaterial().baseColor([0.9, 0.95, 1.0, 0.25]).alphaMode(AlphaMode.blend)

let solid = PbrMaterial().alphaMode(AlphaMode.opaque)