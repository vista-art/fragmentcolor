package org.fragmentcolor

import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.runBlocking
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue
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
class Healthcheck : ExampleCase() {
    @Test
    fun headlessRenderSmoke() = runBlocking {
        runHeadlessRenderSmoke()
    }

    // A size with layers reaches the check the other languages run: a texture
    // target has one layer, so a depth of 2 throws.
    @Test
    fun textureTargetRefusesLayers() = runBlocking {
        val renderer = Renderer()
        assertFailsWith<FragmentColorException> { renderer.createTextureTarget(listOf(8u, 8u, 2u)) }
        Unit
    }

    // Shader.set shapes each Kotlin value to the uniform's declared WGSL type.
    // Setting and reading uniforms touches no GPU, so this runs on any emulator.
    // A region or viewport with a side of 0, or a far edge past UInt.MAX_VALUE,
    // throws instead of shrinking, and a clear color is a list of floats or doubles.
    @Test
    fun regionsAndViewportsThrow() {
        val pass = Pass("regions")
        assertFailsWith<FragmentColorException> { ScreenRegion(listOf(0u, 0u), listOf(0u, 4u)) }
        assertFailsWith<FragmentColorException> { ScreenRegion(listOf(UInt.MAX_VALUE, 0u), listOf(5u, 5u)) }
        assertFailsWith<FragmentColorException> { pass.setViewport(listOf(4u, 0u)) }
        assertFailsWith<FragmentColorException> { pass.setViewport(listOf(1u, 2u, 3u)) }
        pass.setViewport(listOf(0u, 0u, 4u, 4u))
        pass.setClearColor(floatArrayOf(0.1f, 0.2f, 0.3f))
        pass.setClearColor(doubleArrayOf(0.1, 0.2, 0.3, 1.0))
        pass.setClearColor(listOf(0.1, 0.2, 0.3))
    }

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

    // Vertex and Instance numbers take the type the shader declares when the mesh attaches.
    // Attaching reads no GPU, so this runs on any emulator.
    @Test
    fun vertexNumbersTakeShaderTypes() {
        val source = """
            @vertex fn vs_main(
              @location(0) position: vec2<f32>,
              @location(1) id: u32,
              @location(2) cell: vec2<i32>,
            ) -> @builtin(position) vec4<f32> {
              return vec4f(position, f32(id) + f32(cell.x), 1.);
            }
            @fragment fn main() -> @location(0) vec4<f32> { return vec4f(1., 0., 0., 1.); }
            """.trimIndent()

        val vertices = Mesh()
        vertices.addVertex(Vertex(floatArrayOf(0f, 0f)).set("id", 7).set("cell", intArrayOf(-1, 2)))
        Shader(source).addMesh(vertices)

        val instanced = Mesh()
        instanced.addVertex(Vertex(floatArrayOf(0f, 0f)).set("cell", intArrayOf(-1, 2)))
        instanced.addInstance(mapOf("id" to listOf(7)))
        Shader(source).addMesh(instanced)

        val negative = Mesh()
        negative.addVertex(Vertex(floatArrayOf(0f, 0f)).set("id", 7).set("cell", intArrayOf(0, 0)))
        negative.addVertex(Vertex(floatArrayOf(1f, 0f)).set("id", -1).set("cell", intArrayOf(0, 0)))
        val error = assertFailsWith<FragmentColorException> { Shader(source).addMesh(negative) }
        assertTrue(error.message.orEmpty().contains("expected u32, got -1 at vertex 1"), error.message)

        assertFailsWith<FragmentColorException> { Instance().set("offset", listOf(1.0, 2.0, 3.0, 4.0, 5.0)) }
        assertFailsWith<FragmentColorException> { Instance().set("offset", listOf<Double>()) }
    }

    private suspend fun runHeadlessRenderSmoke() {
        // DOC: Renderer.new (begin)
        val renderer = Renderer()
        // DOC: (end)

        // DOC: Renderer.create_texture_target (begin)
        val target = renderer.createTextureTarget(listOf(32u, 64u))
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
