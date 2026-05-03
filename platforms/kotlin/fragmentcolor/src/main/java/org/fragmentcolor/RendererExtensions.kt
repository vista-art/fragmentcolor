package org.fragmentcolor

import android.view.Surface

// Idiomatic Kotlin wrappers on top of the uniffi-generated API. Matches
// the method names used by the JavaScript and Python bindings so calling
// code looks identical across every supported platform.

// The raw JNI entry point lives in `libfragmentcolor.so` and is exposed
// via a private helper class so the shared library is loaded exactly once.
private object RendererJni {
    init {
        System.loadLibrary("fragmentcolor")
    }

    @JvmStatic
    external fun create_window_target_from_surface(surface: Surface): Long
}

/**
 * Build a [MobileWindowTarget] from an [android.view.Surface]. Wraps the
 * raw JNI entry point that uniffi cannot expose directly because it needs
 * access to the `JNIEnv*` pointer. Pairs with the iOS
 * `Renderer.createTarget(layer:)` extension so the same call site compiles
 * on both platforms.
 */
fun Renderer.createTarget(surface: Surface): MobileWindowTarget {
    val ptr = RendererJni.create_window_target_from_surface(surface)
    require(ptr != 0L) { "Failed to create WindowTarget from Surface" }
    return MobileWindowTarget(UniffiWithHandle, ptr)
}

/**
 * Single overloaded `render(...)` family that matches the spelling used by
 * the JavaScript and Python bindings. The uniffi layer exports one concrete
 * `render(renderable, target)` method that takes `RenderableHandle` +
 * `TargetHandle` enums — these extensions wrap the native types into the
 * matching variants invisibly so callers just write
 * `renderer.render(shader, target)` (or `pass`, `mesh`, `passList`).
 */
fun Renderer.render(shader: Shader, target: MobileWindowTarget) {
    render(RenderableHandle.Shader(shader), TargetHandle.Window(target))
}

fun Renderer.render(shader: Shader, target: MobileTextureTarget) {
    render(RenderableHandle.Shader(shader), TargetHandle.Texture(target))
}

fun Renderer.render(pass: Pass, target: MobileWindowTarget) {
    render(RenderableHandle.Pass(pass), TargetHandle.Window(target))
}

fun Renderer.render(pass: Pass, target: MobileTextureTarget) {
    render(RenderableHandle.Pass(pass), TargetHandle.Texture(target))
}

fun Renderer.render(mesh: Mesh, target: MobileWindowTarget) {
    render(RenderableHandle.Mesh(mesh), TargetHandle.Window(target))
}

fun Renderer.render(mesh: Mesh, target: MobileTextureTarget) {
    render(RenderableHandle.Mesh(mesh), TargetHandle.Texture(target))
}

fun Renderer.render(passes: List<Pass>, target: MobileWindowTarget) {
    render(RenderableHandle.Passes(passes), TargetHandle.Window(target))
}

fun Renderer.render(passes: List<Pass>, target: MobileTextureTarget) {
    render(RenderableHandle.Passes(passes), TargetHandle.Texture(target))
}
