from fragmentcolor import Material, Renderer

renderer = Renderer()
crevices = Material.pbr(renderer).occlusion_strength(0.8)