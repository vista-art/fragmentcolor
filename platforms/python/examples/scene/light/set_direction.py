from fragmentcolor import Light

sun = Light.directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
# Reorient to a late-afternoon angle.
sun.set_direction([0.7, -0.5, -0.5])