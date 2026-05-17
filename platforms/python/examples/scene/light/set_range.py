from fragmentcolor import Light

lamp = Light.point([0.0, 2.0, 0.0], [1.0, 1.0, 1.0])
lamp.set_range(8.0)
negative = lamp.set_range(-1.0)

# Directional lights have no range — the call errors.
sun = Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
unsupported = sun.set_range(5.0)