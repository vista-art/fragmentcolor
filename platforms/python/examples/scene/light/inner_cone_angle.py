from fragmentcolor import Light

torch = Light.spot([0.0, 1.8, 1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).set_cone_angles(0.15, 0.4)
lamp = Light.point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])