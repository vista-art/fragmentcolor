from fragmentcolor import Light, Material

material = Material.pbr()
sun = Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9])
sun.bind(material.shader())
