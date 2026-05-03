package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Pass API.
// Provides natural overloads so callers never have to construct
// RenderableHandle or TargetHandle variants by hand.

// ── Dependencies (require) ────────────────────────────────────────────────────

/** Declare a [Shader] as a dependency of this pass. */
fun Pass.require(shader: Shader) {
    require(listOf(RenderableHandle.Shader(shader)))
}

/** Declare another [Pass] as a dependency of this pass. */
fun Pass.require(pass: Pass) {
    require(listOf(RenderableHandle.Pass(pass)))
}

/** Declare a [Mesh] as a dependency of this pass. */
fun Pass.require(mesh: Mesh) {
    require(listOf(RenderableHandle.Mesh(mesh)))
}

/** Declare multiple [Pass] objects as ordered dependencies. */
fun Pass.require(passes: List<Pass>) {
    require(passes.map { RenderableHandle.Pass(it) })
}

/** Declare a heterogeneous list of renderables as dependencies. */
fun Pass.require(deps: List<RenderableHandle>) {
    require(deps)
}

// ── Targets ───────────────────────────────────────────────────────────────────

/** Set the colour attachment target for this pass. */
fun Pass.addTarget(target: TextureTarget) {
    addTarget(TargetHandle.Texture(target))
}

/** Set the depth attachment target for this pass. */
fun Pass.addDepthTarget(target: TextureTarget) {
    addDepthTarget(TargetHandle.Texture(target))
}

// ── Clear colour ──────────────────────────────────────────────────────────────

/** Set the clear colour as separate RGBA components (0..1 linear space). */
fun Pass.setClearColor(r: Float, g: Float, b: Float, a: Float = 1.0f) {
    setClearColor(listOf(r, g, b, a))
}

/** Set the clear colour from a float list (`[r, g, b]` or `[r, g, b, a]`). */
fun Pass.setClearColor(rgba: List<Float>) {
    setClearColor(rgba)
}
