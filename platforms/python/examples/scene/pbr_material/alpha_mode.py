from fragmentcolor import AlphaMode, PbrMaterial

foliage = PbrMaterial().alpha_mode(AlphaMode.Mask).alpha_cutoff(0.3)

glass = PbrMaterial().base_color([0.9, 0.95, 1.0, 0.25]).alpha_mode(AlphaMode.Blend)

solid = PbrMaterial().alpha_mode(AlphaMode.Opaque)