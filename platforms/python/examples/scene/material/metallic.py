from fragmentcolor import Material, Renderer

renderer = Renderer()
chrome = Material.pbr().metallic(1.0).roughness(0.05)