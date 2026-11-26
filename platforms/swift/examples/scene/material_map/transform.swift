import FragmentColor

// Every map spans the surface twice, and the normal map eight times.
let tiles = try PbrMaterial().uvTransform([0.0, 0.0], [2.0, 2.0], 0.0)
let normals = try tiles.normalMap().transform([0.0, 0.0], [8.0, 8.0], 0.0)