import FragmentColor

let sun = try Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
let bulb = try Light.point([0.0, 2.5, 0.0], [1.0, 1.0, 1.0])
let torch = try Light.spot([0.0, 1.8, 1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0])