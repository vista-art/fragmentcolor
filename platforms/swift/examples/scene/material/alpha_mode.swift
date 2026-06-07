import FragmentColor

let foliage = Material.pbr().alphaMode(AlphaMode.mask).alphaCutoff(0.3)

let glass = try Material.pbr().baseColor([0.9, 0.95, 1.0, 0.25]).alphaMode(AlphaMode.blend)

let solid = Material.pbr().alphaMode(AlphaMode.opaque)