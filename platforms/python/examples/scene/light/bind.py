from fragmentcolor import Light, Material, Renderer

renderer = Renderer()
material = Material.pbr(renderer)
sun = Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9])
sun.bind(material.shader())
