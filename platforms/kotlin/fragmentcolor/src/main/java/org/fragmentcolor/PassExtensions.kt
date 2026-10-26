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

/** Declare a [Scene]'s passes as dependencies of this pass. */
fun Pass.require(scene: Scene) {
    require(listOf(RenderableHandle.Scene(scene)))
}

/** Declare multiple [Pass] objects as ordered dependencies. */
@JvmName("requirePasses")
fun Pass.require(passes: List<Pass>) {
    require(passes.map { RenderableHandle.Pass(it) })
}

// ── Scene objects (add) ───────────────────────────────────────────────────────

// The mobile binding takes a SceneObjectHandle enum (Model / Camera / Light)
// and dispatches internally; these overloads let callers pass the concrete
// types directly so example code reads pass.add(model) instead of
// pass.add(SceneObjectHandle.Model(model)). Each returns the pass, so calls chain.

/** Attach a [Model] to the pass. */
fun Pass.add(model: Model): Pass = add(SceneObjectHandle.Model(model))

/** Attach a [Camera] to the pass. */
fun Pass.add(camera: Camera): Pass = add(SceneObjectHandle.Camera(camera))

/** Attach a [Light] to the pass. */
fun Pass.add(light: Light): Pass = add(SceneObjectHandle.Light(light))

// ── Targets ───────────────────────────────────────────────────────────────────

/** Set the color attachment target for this pass. */
fun Pass.setTarget(target: TextureTarget) {
    setTarget(TargetHandle.Texture(target))
}

/** Render into a [Texture], such as one from `Renderer.createTexture`. */
fun Pass.setTarget(texture: Texture) {
    setTargetTexture(texture)
}

/** Set the depth attachment target for this pass. */
fun Pass.setDepthTarget(target: TextureTarget) {
    setDepthTarget(TargetHandle.Texture(target))
}

/**
 * Attach a depth [Texture] (created via `Renderer.createDepthTexture`) as the
 * depth-stencil attachment for this pass.
 */
fun Pass.setDepthTarget(texture: Texture) {
    setDepthTexture(texture)
}

// ── Clear color ──────────────────────────────────────────────────────────────

/** Set the clear color as separate linear RGBA components, each passed to the GPU as given. */
fun Pass.setClearColor(r: Float, g: Float, b: Float, a: Float = 1.0f) {
    setClearColor(listOf(r, g, b, a))
}

// ── Viewport ──────────────────────────────────────────────────────────────────

/**
 * Set the viewport from `[x, y, width, height]`, or `[width, height]` at the origin.
 * Throws [FragmentColorException] for any other length.
 */
fun Pass.setViewport(rect: List<UInt>) {
    val region = when (rect.size) {
        2 -> ScreenRegion(listOf(0u, 0u), rect)
        4 -> ScreenRegion(rect.subList(0, 2), rect.subList(2, 4))
        else -> throw FragmentColorException.Render(
            "setViewport: expected [width, height] or [x, y, width, height], got ${rect.size} numbers",
        )
    }
    setViewport(region)
}

/**
 * A region at [origin] `[x, y]` that is [size] `[width, height]` large,
 * as Rust's `ScreenRegion::new(origin, size)` reads them.
 * Throws [FragmentColorException] unless each holds two numbers.
 */
operator fun ScreenRegion.Companion.invoke(origin: List<UInt>, size: List<UInt>): ScreenRegion {
    if (origin.size != 2 || size.size != 2) {
        throw FragmentColorException.Render("ScreenRegion: origin and size hold 2 numbers each")
    }
    val (x, y) = origin
    val (width, height) = size
    return ScreenRegion(
        minX = x,
        minY = y,
        maxX = saturatingAdd(x, width),
        maxY = saturatingAdd(y, height),
    )
}

/** `a + b`, or [UInt.MAX_VALUE] when the sum doesn't fit. */
private fun saturatingAdd(a: UInt, b: UInt): UInt = if (a > UInt.MAX_VALUE - b) UInt.MAX_VALUE else a + b
