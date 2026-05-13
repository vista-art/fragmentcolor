from fragmentcolor import Material, Renderer

# Direct uniform access for a custom field that isn't covered by the
# Material setters or by Camera / Light.
renderer = Renderer()
material = Material.pbr(renderer)
material.shader().set("material.alpha_cutoff", 0.25_)