from fragmentcolor import Material, Renderer

renderer = Renderer()
foliage = Material.pbr(renderer).alpha_cutoff(0.3)