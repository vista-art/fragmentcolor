from fragmentcolor import Light

lamp = Light.point([0.0, 1.0, 0.0], [1.0, 0.95, 0.8]).set_intensity(12.0)
scale = lamp.intensity()