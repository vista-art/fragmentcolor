import org.fragmentcolor.*

val renderer = Renderer()
// Leaf cards: thin, single-quad geometry; needs both sides + alpha cut-out.
val leaf = Material.pbr()?.doubleSided(true).alphaMode(AlphaMode.Mask).alphaCutoff(0.5)

// Default is single-sided — back-face culling on.
val solid_mesh = Material.pbr()?.doubleSided(false)