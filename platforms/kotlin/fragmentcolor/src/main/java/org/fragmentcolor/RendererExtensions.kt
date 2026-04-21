package org.fragmentcolor

import android.view.Surface
import com.sun.jna.Pointer

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
 * Build a [WindowTarget] from an [android.view.Surface]. Wraps the raw JNI
 * entry point that uniffi cannot expose directly because it needs access
 * to the `JNIEnv*` pointer. Pairs with the iOS `Renderer.createTarget(layer:)`
 * extension so the same call site compiles on both platforms.
 */
fun Renderer.createTarget(surface: Surface): WindowTarget {
    val ptr = RendererJni.create_window_target_from_surface(surface)
    require(ptr != 0L) { "Failed to create WindowTarget from Surface" }
    return WindowTarget(Pointer(ptr))
}

/**
 * Render a [Shader] into a [WindowTarget]. Single overloaded render(...)
 * dispatch that matches the spelling used by the JavaScript and Python
 * bindings. The uniffi layer exports one concrete method per
 * (renderable × target) combination — see the module docs in
 * `src/renderer/platform/mobile/mod.rs` for the rationale.
 */
fun Renderer.render(shader: Shader, target: WindowTarget) {
    renderShader(shader, target)
}

fun Renderer.render(shader: Shader, target: TextureTarget) {
    renderShaderToTexture(shader, target)
}
