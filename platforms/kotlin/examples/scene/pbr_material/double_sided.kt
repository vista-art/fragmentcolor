import org.fragmentcolor.*

val renderer = Renderer()
// Leaf cards: thin, single-quad geometry; needs both sides + alpha cut-out.
val leaf = PbrMaterial().doubleSided(true).alphaMode(AlphaMode.MASK).alphaCutoff(0.5f)

// The default is single-sided, with back-face culling on.
val solid_mesh = PbrMaterial().doubleSided(false)