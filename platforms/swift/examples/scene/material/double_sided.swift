import FragmentColor

let renderer = Renderer()
// Leaf cards: thin, single-quad geometry; needs both sides + alpha cut-out.
let leaf = try await Material.pbr(renderer).doubleSided(true).alphaMode(AlphaMode.mask).alphaCutoff(0.5)

// Default is single-sided — back-face culling on.
let solid_mesh = try await Material.pbr(renderer).doubleSided(false)