package org.fragmentcolor

import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.runBlocking
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import org.junit.Test
import org.junit.runner.RunWith

// Mirrors platforms/python/healthcheck.py against the Android binding.
// Run via `./fc healthcheck android` on a connected emulator with a working
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
        runHeadlessRenderSmoke()
    }

    // Shader.set shapes each Kotlin value to the uniform's declared WGSL type.
    // Setting and reading uniforms touches no GPU, so this runs on any emulator.
    @Test
    fun shaderSetShapesValues() {
        val shader = Shader(
            """
            struct Particles { items: array<vec4<f32>, 2> };
            @group(0) @binding(0) var<uniform> count: u32;
            @group(0) @binding(1) var<uniform> cell: vec2<i32>;
            @group(0) @binding(2) var<uniform> rot: mat2x2<f32>;
            @group(0) @binding(3) var<uniform> offset: vec2<f32>;
            @group(0) @binding(4) var<storage, read> particles: Particles;
            @vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
              let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
              return vec4f(p[i], 0., 1.);
            }
            @fragment fn main() -> @location(0) vec4<f32> {
              let x = f32(count) + f32(cell.x) + rot[0].x + offset.x + particles.items[0].x;
              return vec4f(x, 0., 0., 1.);
            }
            """.trimIndent()
        )

        shader.set("count", 3)
        assertEquals(UniformData.UInt(3u), shader.get("count"))
        shader.set("cell", intArrayOf(2, -1))
        assertEquals(UniformData.IVec2(listOf(2, -1)), shader.get("cell"))
        shader.set("rot", floatArrayOf(1f, 2f, 3f, 4f))
        assertEquals(UniformData.Mat2(listOf(1f, 2f, 3f, 4f)), shader.get("rot"))
        shader.set("offset", intArrayOf(5, 6))
        assertEquals(UniformData.Vec2(listOf(5f, 6f)), shader.get("offset"))

        assertFailsWith<FragmentColorException> { shader.set("offset", floatArrayOf(1f, 2f, 3f)) }
        assertEquals(UniformData.Vec2(listOf(5f, 6f)), shader.get("offset"))
        assertFailsWith<FragmentColorException> { shader.set("count", -1) }
        assertFailsWith<FragmentColorException> { shader.set("offset", ByteArray(8)) }

        val blob = ByteArray(32) { it.toByte() }
        shader.set("particles", blob)
        val (bytes) = shader.get("particles") as UniformData.Bytes
        assertContentEquals(blob, bytes)
    }

    private suspend fun runHeadlessRenderSmoke() {
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
