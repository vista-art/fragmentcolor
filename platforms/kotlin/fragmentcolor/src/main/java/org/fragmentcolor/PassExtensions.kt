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

/** Set the clear color from `[r, g, b]` or `[r, g, b, a]`, each channel passed to the GPU as given. */
fun Pass.setClearColor(color: FloatArray) = setClearColor(color.toList())

/** Set the clear color from `[r, g, b]` or `[r, g, b, a]` as doubles. */
@JvmName("setClearColorDoubles")
fun Pass.setClearColor(color: List<Double>) = setClearColor(color.map { it.toFloat() })

/** Set the clear color from `[r, g, b]` or `[r, g, b, a]` as doubles. */
fun Pass.setClearColor(color: DoubleArray) = setClearColor(color.map { it.toFloat() })

// ── Viewport ──────────────────────────────────────────────────────────────────

/**
 * Set the viewport from `[x, y, width, height]`, or `[width, height]` at the origin.
 * Throws [FragmentColorException] for any other length, and for a width or height of 0.
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
 * Throws [FragmentColorException] unless each holds two numbers, for a side of 0,
 * and for a far edge past [UInt.MAX_VALUE].
 */
operator fun ScreenRegion.Companion.invoke(origin: List<UInt>, size: List<UInt>): ScreenRegion {
    if (origin.size != 2 || size.size != 2) {
        throw FragmentColorException.Render("ScreenRegion: origin and size hold 2 numbers each")
    }
    val (x, y) = origin
    val (width, height) = size
    if (width == 0u || height == 0u) {
        throw FragmentColorException.Render(
            "ScreenRegion: a region needs a width and a height of at least 1, got $width by $height",
        )
    }
    if (x > UInt.MAX_VALUE - width || y > UInt.MAX_VALUE - height) {
        throw FragmentColorException.Render(
            "ScreenRegion: a region at $x, $y sized $width by $height reaches past ${UInt.MAX_VALUE}",
        )
    }
    return ScreenRegion(minX = x, minY = y, maxX = x + width, maxY = y + height)
}
