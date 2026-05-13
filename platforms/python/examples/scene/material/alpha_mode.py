from fragmentcolor import AlphaMode, Material

foliage = Material.pbr().alpha_mode(AlphaMode.Mask).alpha_cutoff(0.3)

glass = Material.pbr().base_color([0.9, 0.95, 1.0, 0.25]).alpha_mode(AlphaMode.Blend)

solid = Material.pbr().alpha_mode(AlphaMode.Opaque)