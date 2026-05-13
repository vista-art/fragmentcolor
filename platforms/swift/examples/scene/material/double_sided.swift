import FragmentColor

// Leaf cards: thin, single-quad geometry; needs both sides + alpha cut-out.
let leaf = Material.pbr()?.doubleSided(true).alphaMode(AlphaMode.mask).alphaCutoff(0.5)

// Default is single-sided — back-face culling on.
let solid_mesh = Material.pbr()?.doubleSided(false)