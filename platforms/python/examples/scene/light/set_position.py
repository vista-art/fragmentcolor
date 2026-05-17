from fragmentcolor import Light

lamp = Light.point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])
lamp.set_position([3.0, 1.5, -2.0])

# Directional lights have no position — the call errors.
sun = Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
result = sun.set_position([0.0, 0.0, 0.0])