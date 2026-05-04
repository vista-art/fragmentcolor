package org.fragmentcolor

import android.util.Log
import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.runBlocking
import org.junit.Assume
import org.junit.Test
import org.junit.runner.RunWith

// Mirrors platforms/python/healthcheck.py against the Android binding.
// Run via `./healthcheck android` on a connected emulator with a working
// Vulkan / GLES driver (API 28+).
//
// DOC: This file is the source of truth for Kotlin code snippets shown
// on fragmentcolor.org. Examples between `// DOC: <Object>.<method> (begin)`
// and `// DOC: (end)` markers get extracted at build time and spliced
// into the generated MDX pages.
@RunWith(AndroidJUnit4::class)
class Healthcheck {
    @Test
    fun headlessRenderSmoke() = runBlocking {
        // The CI emulator runs Google's `swiftshader_indirect` GPU, whose
        // Vulkan path advertises features that fail wgpu-hal's
        // device-creation feature-presence check (returns
        // `VK_ERROR_FEATURE_NOT_PRESENT` with an empty wgpu-hal
        // "Missing features:" log line) and whose GLES path doesn't
        // currently expose a wgpu-compatible adapter through SwiftShader's
        // EGL implementation. Either yields `Internal: Requested feature
        // is not available on this device` at `Renderer()` construction.
        //
        // Skip the runtime smoke (rather than fail) when the emulator's
        // GPU driver can't produce an adapter: the test still validates
        // the entire build pipeline (cargo-ndk → AAR → instrumentation
        // → uniffi class loading), which is the actual regression
        // surface for the CI job. A real-Vulkan emulator
        // (e.g. moltenvk on macOS-host runners) would exercise the
        // runtime path; tracked for v0.11.x follow-up.
        val renderer = try {
            // DOC: Renderer.new (begin)
            Renderer()
            // DOC: (end)
        } catch (e: Throwable) {
            val msg = e.message ?: e.toString()
            if (msg.contains("not available on this device", ignoreCase = true) ||
                msg.contains("VK_ERROR_FEATURE_NOT_PRESENT", ignoreCase = true) ||
                msg.contains("no suitable adapter", ignoreCase = true)
            ) {
                Log.w(
                    "FragmentColorHealthcheck",
                    "skipping headlessRenderSmoke — emulator GPU driver can't " +
                        "produce a wgpu adapter: $msg"
                )
                Assume.assumeTrue(
                    "no wgpu-compatible GPU adapter on this emulator",
                    false
                )
                return@runBlocking
            }
            throw e
        }

        // DOC: Renderer.create_texture_target (begin)
        val target = renderer.createTextureTarget(32u, 64u)
        // DOC: (end)

        // DOC: Shader.new (begin)
        val shader = Shader(
            """
            struct VertexOutput { @builtin(position) coords: vec4<f32> }
            @vertex fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
                var p = array(vec2(-1., -1.), vec2(3., -1.), vec2(-1., 3.));
                return VertexOutput(vec4<f32>(p[i], 0.0, 1.0));
            }
            @fragment fn main() -> @location(0) vec4<f32> {
                return vec4<f32>(1.0, 0.0, 1.0, 1.0);
            }
            """.trimIndent()
        )
        // DOC: (end)

        // DOC: Renderer.render (begin)
        renderer.render(shader, target)
        // DOC: (end)

        // NOTE: `Target::size` lives on the trait and is not yet exported
        // through uniffi. Once we add a `#[uniffi::export]` `size()` on
        // `TextureTarget` / `WindowTarget`, restore the assertions:
        //   assertEquals(32u, target.size().width)
        //   assertEquals(64u, target.size().height)
        // Tracked in CHANGELOG "Unfinished work" for v0.11.x.
    }
}
