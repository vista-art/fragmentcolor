from fragmentcolor import Light

torch = Light.spot([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).set_cone_angles(0.2, 0.5)
inner = torch.inner_cone_angle()