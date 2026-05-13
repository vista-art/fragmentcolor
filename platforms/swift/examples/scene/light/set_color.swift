import FragmentColor

let lamp = Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
// Warm-tinted bulb after the user toggles the warm-light switch.
lamp.setColor([1.0, 0.85, 0.7])