from fragmentcolor import AlphaMode, Material, Renderer

renderer = Renderer()
# Leaf cards: thin, single-quad geometry; needs both sides + alpha cut-out.
leaf = Material.pbr(renderer).double_sided(true).alpha_mode(AlphaMode.Mask).alpha_cutoff(0.5)

# Default is single-sided — back-face culling on.
solid_mesh = Material.pbr(renderer).double_sided(false)