from fragmentcolor import PbrMaterial, Renderer

renderer = Renderer()
lava = PbrMaterial().base_color([0.1, 0.05, 0.0, 1.0]).emissive([1.5, 0.4, 0.1])