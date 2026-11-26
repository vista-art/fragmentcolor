from fragmentcolor import AlphaMode, PbrMaterial, Renderer

renderer = Renderer()
# Leaf cards: thin, single-quad geometry; needs both sides + alpha cut-out.
leaf = PbrMaterial().double_sided(True).alpha_mode(AlphaMode.Mask).alpha_cutoff(0.5)

# The default is single-sided, with back-face culling on.
solid_mesh = PbrMaterial().double_sided(False)