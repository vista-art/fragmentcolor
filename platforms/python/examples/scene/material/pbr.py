from fragmentcolor import Material, Renderer

renderer = Renderer()
bronze = Material.pbr().base_color([0.8, 0.5, 0.2, 1.0]).metallic(1.0).roughness(0.3)
