package org.fragmentcolor

import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

// Mirrors platforms/python/healthcheck.py against the Android binding.
// Run via `./healthcheck android` on a connected emulator with a working
// Vulkan driver (API 28+).
//
// DOC: This file is the source of truth for Kotlin code snippets shown
// on fragmentcolor.org. Examples between `// DOC: <Object>.<method> (begin)`
// and `// DOC: (end)` markers get extracted at build time and spliced
// into the generated MDX pages.
@RunWith(AndroidJUnit4::class)
class Healthcheck {
    @Test
    fun headlessRenderSmoke() = runBlocking {
        // DOC: Renderer.new (begin)
        val renderer = Renderer()
        // DOC: (end)

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
