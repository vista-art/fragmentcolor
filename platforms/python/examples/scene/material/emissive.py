from fragmentcolor import Material, Renderer

renderer = Renderer()
lava = Material.pbr().base_color([0.1, 0.05, 0.0, 1.0]).emissive([1.5, 0.4, 0.1])