from fragmentcolor import UnlitMaterial

# Raw pixels with their size upload when the scene loads.
pixels = [255, 128, 0, 255]
material = UnlitMaterial()
map = material.base_color_map().texture(pixels, size=[1, 1])