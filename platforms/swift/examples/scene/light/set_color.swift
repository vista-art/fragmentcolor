import FragmentColor

let lamp = Light.point([0.0, 2.0, 0.0], [1.0, 1.0, 1.0])

// Warm-tint the lamp later — every Pass that absorbed """lamp""" sees the
// color on the next render.
lamp.setColor([1.0, 0.7, 0.4])