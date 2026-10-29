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

    // A pipeline the GPU refuses throws FragmentColorException.Device, with its kind, the
    // step's label and the device's own detail. An error scope catches it, so the
    // GPU error log stays empty, and toString() carries the full text.
    @Test
    fun refusedPipelineThrowsDevice() = runBlocking {
        val renderer = Renderer()
        val target = renderer.createTextureTarget(listOf(8u, 8u))
        // The fragment stage reads @location(0), which the vertex stage never writes.
        val unfed = Shader(
            """
            struct VOut { @builtin(position) pos: vec4<f32> };
            @vertex fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
              let p = array<vec2<f32>, 3>(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
              return VOut(vec4f(p[i], 0., 1.));
            }
            @fragment fn fs_main(@location(0) tint: vec4<f32>) -> @location(0) vec4<f32> { return tint; }
            """.trimIndent()
        )
        val error = assertFailsWith<FragmentColorException.Device> { renderer.render(unfed, target) }
        assertEquals(ErrorKind.VALIDATION, error.kind)
        assertEquals("render pipeline", error.label)
        assertTrue(error.detail.isNotEmpty(), "Device detail is empty")
        assertEquals("Device error (validation) in 'render pipeline': ${error.detail}", error.toString())
    }

    // A size with layers reaches the check the other languages run: a texture
    // target has one layer, so a depth of 2 throws.
    @Test
    fun textureTargetRefusesLayers() = runBlocking {
        val renderer = Renderer()
        assertFailsWith<FragmentColorException> { renderer.createTextureTarget(listOf(8u, 8u, 2u)) }
        Unit
    }

    // render refuses a pass that samples its own target, and a target another
    // Renderer created, before any GPU work. The foreign target stopped the app inside wgpu.
    @Test
    fun passChecksThrow() = runBlocking {
        val renderer = Renderer()
        val target = renderer.createTextureTarget(listOf(8u, 8u))
        val feedback = Shader(
            """
            @group(0) @binding(0) var source: texture_2d<f32>;
            @group(0) @binding(1) var soft: sampler;
            @vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
              let p = array<vec2<f32>, 3>(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
              return vec4f(p[i], 0., 1.);
            }
            @fragment fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
              return textureSample(source, soft, pos.xy / 8.0);
            }
            """.trimIndent()
        )
        feedback.set("source", target.texture())
        val conflict = assertFailsWith<FragmentColorException.Render> { renderer.render(feedback, target) }
        assertTrue(conflict.toString().contains("Texture conflict"), conflict.toString())

        val solid = Shader(
            """
            @vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
              let p = array<vec2<f32>, 3>(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
              return vec4f(p[i], 0., 1.);
            }
            @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4f(1., 0., 0., 1.); }
            """.trimIndent()
        )
        val foreign = Renderer().createTextureTarget(listOf(8u, 8u))
        val owner = assertFailsWith<FragmentColorException.Render> { renderer.render(solid, foreign) }
        assertTrue(owner.toString().contains("another Renderer"), owner.toString())
        renderer.render(solid, target)
    }

    // One shader draws meshes with other layouts, and a second Renderer draws them too,
    // where it stopped the app inside wgpu. render refuses indices past the vertices,
    // and addMesh a vertex property named position.
    @Test
    fun meshChecks() = runBlocking {
        val source = """
            struct VOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec4<f32> };
            @vertex fn vs_main(@location(0) position: vec2<f32>, @location(1) color: vec4<f32>) -> VOut {
              return VOut(vec4f(position, 0., 1.), color);
            }
            @fragment fn fs_main(v: VOut) -> @location(0) vec4<f32> { return v.color; }
            """.trimIndent()
        val corners = listOf(listOf(-1f, -1f), listOf(3f, -1f), listOf(-1f, 3f))
        val shader = Shader(source)
        val painted = Mesh()
        for (corner in corners) {
            painted.addVertex(Vertex(corner).set("color", listOf(1.0, 0.0, 0.0, 1.0)))
        }
        val tinted = Mesh()
        tinted.addVertices(corners)
        tinted.addInstance(mapOf("color" to listOf(0.0, 0.0, 1.0, 1.0)))
        shader.addMesh(painted)
        shader.addMesh(tinted)

        val renderer = Renderer()
        val target = renderer.createTextureTarget(listOf(8u, 8u))
        renderer.render(shader, target)
        val second = Renderer()
        second.render(shader, second.createTextureTarget(listOf(8u, 8u)))

        painted.setIndices(listOf(0u, 1u, 5u))
        val past = assertFailsWith<FragmentColorException.Render> { renderer.render(shader, target) }
        assertTrue(past.toString().contains("points past the last of 3 vertices"), past.toString())
        painted.setIndices(listOf(0u, 1u, 2u))
        renderer.render(shader, target)

        val named = Mesh()
        named.addVertex(
            Vertex(listOf(0f, 0f)).set("position", listOf(5.0, 5.0)).set("color", listOf(1.0, 1.0, 1.0, 1.0))
        )
        val error = assertFailsWith<FragmentColorException> { Shader(source).addMesh(named) }
        assertTrue(error.message.orEmpty().contains("property named 'position'"), error.message)
        assertFailsWith<FragmentColorException> { Shader.fromMesh(Mesh()) }
        assertFailsWith<FragmentColorException> { Shader.fromMesh(named) }

        // No device takes 65 vertex attributes, so an input at @location(64) is past every limit.
        val far = Shader(
            """
            @vertex fn vs_main(@location(0) position: vec2<f32>, @location(64) extra: f32) -> @builtin(position) vec4<f32> { return vec4<f32>(position * extra, 0.0, 1.0); }
            @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
            """.trimIndent()
        )
        val reaching = Mesh()
        for (corner in corners) {
            reaching.addVertex(Vertex(corner).set("extra", 1.0))
        }
        far.addMesh(reaching)
        val limit = assertFailsWith<FragmentColorException.Limit> { renderer.render(far, target) }
        assertEquals("max_vertex_attributes", limit.limit)
        assertEquals(65uL, limit.value)
    }

    // render refuses a shader the device can't run before any GPU work, with a case
    // that names the kind of refusal and carries its fields.
    @Test
    fun shaderRefusalsThrowTypedCases() = runBlocking {
        val renderer = Renderer()
        val target = renderer.createTextureTarget(listOf(8u, 8u))
        val far = Shader(
            """
            @group(100) @binding(0) var<uniform> tint: vec4<f32>;
            @vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
              let p = array<vec2<f32>, 3>(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
              return vec4f(p[i], 0., 1.);
            }
            @fragment fn fs_main() -> @location(0) vec4<f32> { return tint; }
            """.trimIndent()
        )
        val limit = assertFailsWith<FragmentColorException.Limit> { renderer.render(far, target) }
        assertEquals("max_bind_groups", limit.limit)
        assertEquals(101uL, limit.value)
        assertTrue(limit.max < 101uL && limit.item.startsWith("'tint'"), limit.toString())
        val storage = Shader(
            """
            @group(0) @binding(0) var img: texture_storage_2d<bgra8unorm, write>;
            @compute @workgroup_size(1) fn cs_main() { textureStore(img, vec2<i32>(0), vec4<f32>(1.0)); }
            """.trimIndent()
        )
        val feature = assertFailsWith<FragmentColorException.Feature> { renderer.render(storage, target) }
        assertEquals("'img' at @group(0) @binding(0)", feature.item)
        assertEquals("WriteOnly storage access to Bgra8Unorm", feature.feature)
        val unset = Shader(
            """
            @group(0) @binding(0) var source: texture_2d<f32>;
            @group(0) @binding(1) var soft: sampler;
            @vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
              let p = array<vec2<f32>, 3>(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
              return vec4f(p[i], 0., 1.);
            }
            @fragment fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
              return textureSample(source, soft, pos.xy / 8.0);
            }
            """.trimIndent()
        )
        val binding = assertFailsWith<FragmentColorException.Binding> { renderer.render(unset, target) }
        assertEquals("source", binding.uniform)
        assertEquals("a texture", binding.expected)
        assertEquals("none, since it was never set", binding.found)
        val lone = Shader("@fragment fn main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }")
        val entry = assertFailsWith<FragmentColorException.Entry> { renderer.render(lone, target) }
        assertEquals("@vertex", entry.stage)
    }

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

    // A color or vector item that is NaN, an infinity or past the Float range
    // throws, and the message names the call, as in every binding.
    @Test
    fun floatVectorsRefuseNonFiniteNumbers() {
        val material = Material.pbr()
        val cases: List<Pair<String, () -> Unit>> = listOf(
            "Light.point: position" to { Light.point(doubleArrayOf(1e39, 0.0, 0.0), doubleArrayOf(1.0, 1.0, 1.0)) },
            "Light.point: position" to { Light.point(listOf(Float.NaN, 0f, 0f), listOf(1f, 1f, 1f)) },
            "Light.setColor" to { Light.point(listOf(0f, 1f, 0f), listOf(1f, 1f, 1f)).setColor(listOf(Double.POSITIVE_INFINITY, 0.0, 0.0)) },
            "Pass.setClearColor" to { Pass("p").setClearColor(doubleArrayOf(0.0, -1e39, 0.0)) },
            "Pass.setClearColor" to { Pass("p").setClearColor(floatArrayOf(Float.NaN, 0f, 0f)) },
            "Scene.ambient" to { Scene().ambient(listOf(Float.POSITIVE_INFINITY, 0f, 0f)) },
            "Model.translate" to { Model(Mesh(), material).translate(doubleArrayOf(0.0, Double.NaN, 0.0)) },
            "Model.rotate: axis" to { Model(Mesh(), material).rotate(floatArrayOf(0f, Float.POSITIVE_INFINITY, 0f), 1f) },
            "Camera.lookAt: up" to {
                Camera.perspective(1f, 1f, 0.1f, 100f).lookAt(listOf(0.0, 0.0, 5.0), listOf(0.0, 0.0, 0.0), listOf(0.0, 1e39, 0.0))
            },
            "Material.baseColor" to { material.baseColor(listOf(1f, 1f, Float.NaN, 1f)) },
            "Material.uvTransform: offset" to { material.uvTransform(doubleArrayOf(1e39, 0.0), doubleArrayOf(1.0, 1.0), 0.0) },
            "Material.textureTransform: scale" to {
                material.textureTransform(TextureSlot.NORMAL, listOf(0f, 0f), listOf(Float.POSITIVE_INFINITY, 1f), 0f)
            },
            "Vertex: position" to { Vertex(doubleArrayOf(1e39, 0.0, 0.0)) },
            "Vertex: position" to { Vertex(listOf(Float.NaN, 0f)) },
            "Vertex.pbr: position" to { Vertex.pbr(listOf(0.0, Double.POSITIVE_INFINITY, 0.0)) },
            "Mesh.addVertex: vertex" to { Mesh().addVertex(listOf(0.0, Double.NaN, 0.0)) },
            "Mesh.addVertex: vertex" to { Mesh().addVertex(floatArrayOf(Float.POSITIVE_INFINITY, 0f)) },
            "Mesh.addVertex: vertex: position" to { Mesh().addVertex(listOf(0f, 0f, Float.NaN), listOf(0f, 0f)) },
            "Mesh.addVertices: vertex 1" to { Mesh().addVertices(listOf(listOf(0.0, 0.0), listOf(1e39, 0.0))) },
            "Mesh.fromVertices: vertex 0" to { Mesh.fromVertices(listOf(listOf(Float.NaN, 0f))) },
            "Quad: min" to { Quad(listOf(1e39, 0.0), listOf(1.0, 1.0)) },
            "Quad: max" to { Quad(listOf(0f, 0f), listOf(Float.POSITIVE_INFINITY, 1f)) },
        )
        for ((label, call) in cases) {
            val error = assertFailsWith<FragmentColorException> { call() }
            assertTrue(error.message.orEmpty().contains(label), "$label threw ${error.message}")
        }
        // A Double item prints the way the shared Rust message prints it.
        val shown = listOf(1e39 to "1e39", -1e39 to "-1e39", Double.NaN to "NaN", Double.POSITIVE_INFINITY to "inf")
        for ((value, text) in shown) {
            val error = assertFailsWith<FragmentColorException> {
                Light.point(doubleArrayOf(value, 0.0, 0.0), doubleArrayOf(1.0, 1.0, 1.0))
            }
            assertTrue(error.toString().endsWith("got $text"), "$text printed as $error")
        }
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
