from fragmentcolor import Light, LightKind

sun = Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
bulb = Light.point([0.0, 2.5, 0.0], [1.0, 1.0, 1.0])
torch = Light.spot([0.0, 1.8, 1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0])