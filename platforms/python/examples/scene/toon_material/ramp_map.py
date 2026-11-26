from fragmentcolor import ToonMaterial

# Deep blue shadows, a warm middle tone and full light, one texel each.
ramp = [
    20, 30, 90, 255, 20, 30, 90, 255,
    200, 140, 110, 255, 255, 255, 255, 255,
]
material = ToonMaterial()
map = material.ramp_map().texture(ramp, size=[4, 1])