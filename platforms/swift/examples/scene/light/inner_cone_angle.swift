import FragmentColor

let torch = try Light.spot([0.0, 1.8, 1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).setConeAngles(0.15, 0.4)
let lamp = try Light.point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])