from fragmentcolor import Material, Renderer

renderer = Renderer()
red = Material.pbr(renderer).base_color([1.0, 0.2, 0.2, 1.0])