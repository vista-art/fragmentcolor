import FragmentColor

let lamp = Light.point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])
lamp.setPosition([3.0, 1.5, -2.0])

// Directional lights have no position — the call errors.
let sun = Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
let result = sun.setPosition([0.0, 0.0, 0.0])