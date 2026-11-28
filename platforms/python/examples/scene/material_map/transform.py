from fragmentcolor import PbrMaterial

# Every map spans the surface twice, and the normal map eight times.
tiles = PbrMaterial().uv_transform([0.0, 0.0], [2.0, 2.0], 0.0)
normals = tiles.normal_map().transform([0.0, 0.0], [8.0, 8.0], 0.0)