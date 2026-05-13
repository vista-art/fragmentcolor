from fragmentcolor import AlphaMode, Material, Renderer

renderer = Renderer()
foliage = Material.pbr(renderer).alpha_mode(AlphaMode.Mask).alpha_cutoff(0.3)

glass = Material.pbr(renderer).base_color([0.9, 0.95, 1.0, 0.25]).alpha_mode(AlphaMode.Blend)

solid = Material.pbr(renderer).alpha_mode(AlphaMode.Opaque)