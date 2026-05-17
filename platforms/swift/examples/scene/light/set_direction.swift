import FragmentColor

let sun = Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
sun.setDirection([0.3, -0.8, -0.5])

// Point lights have no direction — the call errors.
let lamp = Light.point([0.0, 2.0, 0.0], [1.0, 1.0, 1.0])
let result = lamp.setDirection([0.0, -1.0, 0.0])