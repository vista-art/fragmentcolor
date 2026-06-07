from fragmentcolor import Light

lamp = Light.point([0.0, 2.0, 0.0], [1.0, 1.0, 1.0])

# Warm-tint the lamp later — every Pass that absorbed `lamp` sees the
# new color on the next render.
lamp.set_color([1.0, 0.7, 0.4])