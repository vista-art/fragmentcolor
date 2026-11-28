from fragmentcolor import PbrMaterial

# Raw pixels with their size upload when the scene loads.
tint = [255, 128, 0, 255]
material = PbrMaterial()
map = material.base_color_map().texture(tint, size=[1, 1])