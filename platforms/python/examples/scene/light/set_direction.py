from fragmentcolor import Light

sun = Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
sun.set_direction([0.3, -0.8, -0.5])

# Point lights have no direction — the call errors.
lamp = Light.point([0.0, 2.0, 0.0], [1.0, 1.0, 1.0])
result = lamp.set_direction([0.0, -1.0, 0.0])