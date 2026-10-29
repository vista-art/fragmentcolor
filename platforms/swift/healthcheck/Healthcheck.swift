//
//  Healthcheck.swift
//  FragmentColor
//
//  Mirrors platforms/python/healthcheck.py. Exercises the uniffi surface
//  against a real Metal device on an iOS simulator, where
//  `./fc healthcheck ios` runs it with the generated example tests.
//
//  DOC: This file is the source of truth for Swift code snippets shown on
//  fragmentcolor.org. Examples between `// DOC: <Object>.<method> (begin)`
//  and `// DOC: (end)` markers get extracted at build time and spliced
//  into the generated MDX pages.
//

import Foundation
import FragmentColor
import XCTest

enum HealthcheckError: Error {
    case assertion(String)
}

func assertEqual<T: Equatable>(_ actual: T, _ expected: T, _ label: String) throws {
    guard actual == expected else {
        throw HealthcheckError.assertion("\(label): expected \(expected), got \(actual)")
    }
}

final class Healthcheck: ExampleCase {
    func testHeadlessRender() async throws {
        // DOC: Renderer.new (begin)
        let renderer = Renderer()
        // DOC: (end)

        // DOC: Renderer.create_texture_target (begin)
        let target = try await renderer.createTextureTarget([32, 64])
        // DOC: (end)

        // DOC: Shader.new (begin)
        let shader = try Shader(source: """
            struct VertexOutput { @builtin(position) coords: vec4<f32> }
            @vertex fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
                var p = array(vec2(-1., -1.), vec2(3., -1.), vec2(-1., 3.));
                return VertexOutput(vec4<f32>(p[i], 0.0, 1.0));
            }
            @fragment fn main() -> @location(0) vec4<f32> {
                return vec4<f32>(1.0, 0.0, 1.0, 1.0);
            }
        """)
        // DOC: (end)

        // DOC: Renderer.render (begin)
        try renderer.render(shader, target)
        // DOC: (end)

        // NOTE: `Target::size` lives on the trait and is not yet exported
        // through uniffi. Once we add a `#[uniffi::export]` `size()` on
        // `TextureTarget` / `WindowTarget`, restore the assertions:
        //   try assertEqual(target.size().width, 32, "TextureTarget.width")
        //   try assertEqual(target.size().height, 64, "TextureTarget.height")
        // Tracked in CHANGELOG "Unfinished work" for v0.11.x.
    }

    // A pipeline the GPU refuses throws FragmentColorError.Device, with its kind, the
    // step's label and the device's own detail. An error scope catches it, so the
    // GPU error log stays empty, and localizedDescription carries the full text.
    func testRefusedPipelineThrowsDevice() async throws {
        let renderer = Renderer()
        let target = try await renderer.createTextureTarget([8, 8])
        // The fragment stage reads @location(0), which the vertex stage never writes.
        let unfed = try Shader(source: """
            struct VOut { @builtin(position) pos: vec4<f32> };
            @vertex fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
              let p = array<vec2<f32>, 3>(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
              return VOut(vec4f(p[i], 0., 1.));
            }
            @fragment fn fs_main(@location(0) tint: vec4<f32>) -> @location(0) vec4<f32> { return tint; }
        """)
        do {
            try renderer.render(unfed, target)
            throw HealthcheckError.assertion("render of a refused pipeline should throw")
        } catch let error as FragmentColorError {
            guard case let .Device(kind, label, detail) = error else {
                throw HealthcheckError.assertion("render threw \(error), not Device")
            }
            try assertEqual(kind, .validation, "Device kind")
            try assertEqual(label, "render pipeline", "Device label")
            guard !detail.isEmpty else {
                throw HealthcheckError.assertion("Device detail is empty")
            }
            let text = error.localizedDescription
            guard text == "Device error (validation) in 'render pipeline': \(detail)" else {
                throw HealthcheckError.assertion("localizedDescription is \(text)")
            }
        }
    }

    // render refuses a shader the device can't run before any GPU work, with a case
    // that names the kind of refusal and carries its fields.
    func testShaderRefusalsThrowTypedCases() async throws {
        let renderer = Renderer()
        let target = try await renderer.createTextureTarget([8, 8])
        let far = try Shader(source: """
            @group(100) @binding(0) var<uniform> tint: vec4<f32>;
            @vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
              let p = array<vec2<f32>, 3>(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
              return vec4f(p[i], 0., 1.);
            }
            @fragment fn fs_main() -> @location(0) vec4<f32> { return tint; }
        """)
        do {
            try renderer.render(far, target)
            throw HealthcheckError.assertion("a group past the device should throw")
        } catch let error as FragmentColorError {
            guard case let .Limit(item, limit, value, max) = error else {
                throw HealthcheckError.assertion("render threw \(error), not Limit")
            }
            try assertEqual(limit, "max_bind_groups", "Limit limit")
            try assertEqual(value, 101, "Limit value")
            guard max < 101, item.hasPrefix("'tint'") else {
                throw HealthcheckError.assertion("Limit item \(item), max \(max)")
            }
        }
        let lone = try Shader(source: "@fragment fn main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }")
        do {
            try renderer.render(lone, target)
            throw HealthcheckError.assertion("a shader without a vertex stage should throw")
        } catch let error as FragmentColorError {
            guard case let .Entry(stage) = error else {
                throw HealthcheckError.assertion("render threw \(error), not Entry")
            }
            try assertEqual(stage, "@vertex", "Entry stage")
        }
    }

    // A size with layers reaches the check the other languages run: a texture
    // target has one layer, so a depth of 2 throws.
    func testTextureTargetRefusesLayers() async throws {
        let renderer = Renderer()
        do {
            _ = try await renderer.createTextureTarget([8, 8, 2])
            throw HealthcheckError.assertion("createTextureTarget([8, 8, 2]) should throw")
        } catch is FragmentColorError {}
    }

    // render refuses a pass that samples its own target, and a target another
    // Renderer created, before any GPU work. The foreign target stopped the app inside wgpu.
    func testPassChecksThrow() async throws {
        let renderer = Renderer()
        let target = try await renderer.createTextureTarget([8, 8])
        let feedback = try Shader(source: """
            @group(0) @binding(0) var source: texture_2d<f32>;
            @group(0) @binding(1) var soft: sampler;
            @vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
              let p = array<vec2<f32>, 3>(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
              return vec4f(p[i], 0., 1.);
            }
            @fragment fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
              return textureSample(source, soft, pos.xy / 8.0);
            }
        """)
        try feedback.set("source", target.texture())
        try assertRenderThrows("sampling the render target", "Texture conflict") {
            try renderer.render(feedback, target)
        }

        let solid = try Shader(source: """
            @vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
              let p = array<vec2<f32>, 3>(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
              return vec4f(p[i], 0., 1.);
            }
            @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4f(1., 0., 0., 1.); }
        """)
        let foreign = try await Renderer().createTextureTarget([8, 8])
        try assertRenderThrows("a target from another Renderer", "another Renderer") {
            try renderer.render(solid, foreign)
        }
        try renderer.render(solid, target)
    }

    // One shader draws meshes with other layouts, and a second Renderer draws them too,
    // where it stopped the app inside wgpu. render refuses indices past the vertices,
    // and addMesh a vertex property named position.
    func testMeshChecks() async throws {
        let source = """
            struct VOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec4<f32> };
            @vertex fn vs_main(@location(0) position: vec2<f32>, @location(1) color: vec4<f32>) -> VOut {
              return VOut(vec4f(position, 0., 1.), color);
            }
            @fragment fn fs_main(v: VOut) -> @location(0) vec4<f32> { return v.color; }
        """
        let corners: [[Float]] = [[-1, -1], [3, -1], [-1, 3]]
        let shader = try Shader(source)
        let painted = Mesh()
        for corner in corners {
            painted.addVertex(try Vertex(corner).set("color", [1.0, 0.0, 0.0, 1.0]))
        }
        let tinted = Mesh()
        try tinted.addVertices(corners)
        try tinted.addInstance(["color": [0.0, 0.0, 1.0, 1.0]])
        try shader.addMesh(painted)
        try shader.addMesh(tinted)

        let renderer = Renderer()
        let target = try await renderer.createTextureTarget([8, 8])
        try renderer.render(shader, target)
        let second = Renderer()
        let other = try await second.createTextureTarget([8, 8])
        try second.render(shader, other)

        painted.setIndices([0, 1, 5])
        try assertRenderThrows("an index past the last vertex", "points past the last of 3 vertices") {
            try renderer.render(shader, target)
        }
        painted.setIndices([0, 1, 2])
        try renderer.render(shader, target)

        let named = Mesh()
        named.addVertex(try Vertex([0.0, 0.0]).set("position", [5.0, 5.0]).set("color", [1.0, 1.0, 1.0, 1.0]))
        do {
            try Shader(source).addMesh(named)
            throw HealthcheckError.assertion("a position property should throw")
        } catch FragmentColorError.Shader(let message) {
            guard message.contains("property named 'position'") else {
                throw HealthcheckError.assertion("a position property: unexpected message \(message)")
            }
        }

        // No device takes 65 vertex attributes, so an input at @location(64) is past every limit.
        let far = try Shader("""
            @vertex fn vs_main(@location(0) position: vec2<f32>, @location(64) extra: f32) -> @builtin(position) vec4<f32> { return vec4<f32>(position * extra, 0.0, 1.0); }
            @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
        """)
        let reaching = Mesh()
        for corner in corners {
            reaching.addVertex(try Vertex(corner).set("extra", 1.0))
        }
        try far.addMesh(reaching)
        do {
            try renderer.render(far, target)
            throw HealthcheckError.assertion("a vertex input past the attribute limit should throw")
        } catch let error as FragmentColorError {
            guard case let .Limit(_, limit, value, _) = error else {
                throw HealthcheckError.assertion("render threw \(error), not Limit")
            }
            try assertEqual(limit, "max_vertex_attributes", "Limit limit")
            try assertEqual(value, 65, "Limit value")
        }
    }

    /// Checks that `run` throws `FragmentColorError.Render` with `words` in its text.
    private func assertRenderThrows(_ label: String, _ words: String, _ run: () throws -> Void) throws {
        do {
            try run()
        } catch let error as FragmentColorError {
            guard case let .Render(message) = error, message.contains(words) else {
                throw HealthcheckError.assertion("\(label) threw \(error)")
            }
            return
        }
        throw HealthcheckError.assertion("\(label) should throw")
    }

    // A region or viewport with a side of 0, or a far edge past UInt32.max,
    // throws instead of shrinking, and a clear color is a list of floats or doubles.
    func testRegionsAndViewportsThrow() throws {
        let pass = Pass("regions")
        do {
            _ = try ScreenRegion((0, 0), (0, 4))
            throw HealthcheckError.assertion("ScreenRegion((0, 0), (0, 4)) should throw")
        } catch is FragmentColorError {}
        do {
            _ = try ScreenRegion((UInt32.max, 0), (5, 5))
            throw HealthcheckError.assertion("ScreenRegion((UInt32.max, 0), (5, 5)) should throw")
        } catch is FragmentColorError {}
        do {
            try pass.setViewport([4, 0] as [UInt32])
            throw HealthcheckError.assertion("setViewport([4, 0]) should throw")
        } catch is FragmentColorError {}
        do {
            try pass.setViewport([1, 2, 3] as [UInt32])
            throw HealthcheckError.assertion("setViewport([1, 2, 3]) should throw")
        } catch is FragmentColorError {}
        try pass.setViewport([0, 0, 4, 4] as [UInt32])
        try pass.setClearColor([0.1, 0.2, 0.3] as [Double])
        try pass.setClearColor([0.1, 0.2, 0.3, 1.0] as [Float])
    }

    // A list of integer literals compiles wherever a list of floats does.
    func testIntegerLiteralsCompile() throws {
        try Pass("literals").setClearColor([0, 0, 0, 1])
        _ = try Light.point([0, 2, 0], [1, 1, 1]).setColor([1, 0, 0])
        _ = try Scene().ambient([0, 0, 0])
        _ = try Material.pbr().baseColor([1, 1, 1, 1])
        try Model(Mesh(), Material.pbr()).translate([0, 1, 0])
        _ = try Camera.perspective(1, 1, 0.1, 100).lookAt([0, 0, 5], [0, 0, 0], [0, 1, 0])
        try Mesh().addVertex([0, 0, 0])
    }

    // A color or vector item that is NaN, an infinity or past the Float range
    // throws, and the message names the call, as in every binding.
    func testFloatVectorsRefuseNonFiniteNumbers() throws {
        let material = Material.pbr()
        let cases: [(String, () throws -> Void)] = [
            ("Light.point: position", { _ = try Light.point([1e39, 0, 0] as [Double], [1, 1, 1]) }),
            ("Light.point: position", { _ = try Light.point([Float.nan, 0, 0], [1, 1, 1]) }),
            ("Light.setColor", { _ = try Light.point([0, 1, 0], [1, 1, 1]).setColor([Double.infinity, 0, 0]) }),
            ("Pass.setClearColor", { try Pass("p").setClearColor([0, -1e39, 0] as [Double]) }),
            ("Pass.setClearColor", { try Pass("p").setClearColor([Float.nan, 0, 0]) }),
            ("Scene.ambient", { _ = try Scene().ambient([Float.infinity, 0, 0]) }),
            ("Model.translate", { try Model(Mesh(), material).translate([0, Double.nan, 0]) }),
            ("Model.rotate: axis", { try Model(Mesh(), material).rotate([0, Float.infinity, 0], 1) }),
            ("Camera.lookAt: up", { _ = try Camera.perspective(1, 1, 0.1, 100).lookAt([0, 0, 5], [0, 0, 0], [0, 1e39, 0] as [Double]) }),
            ("Material.baseColor", { _ = try material.baseColor([1, 1, Float.nan, 1]) }),
            ("Material.uvTransform: offset", { _ = try material.uvTransform([1e39, 0] as [Double], [1, 1], 0) }),
            ("Material.textureTransform: scale", { _ = try material.textureTransform(.normal, [0, 0], [Float.infinity, 1], 0) }),
            ("Vertex: position", { _ = try Vertex([1e39, 0, 0] as [Double]) }),
            ("Vertex: position", { _ = try Vertex([Float.nan, 0]) }),
            ("Vertex.pbr: position", { _ = try Vertex.pbr([0, Double.infinity, 0]) }),
            ("Vertex: position", { try Mesh().addVertex([0, Double.nan, 0]) }),
            ("Quad: min", { _ = try Quad([1e39, 0] as [Double], [1, 1]) }),
            ("Quad: max", { _ = try Quad([0, 0], [Float.infinity, 1]) }),
        ]
        for (label, call) in cases {
            do {
                try call()
                throw HealthcheckError.assertion("\(label) should throw")
            } catch let error as FragmentColorError {
                guard "\(error)".contains(label) else {
                    throw HealthcheckError.assertion("\(label) threw \(error)")
                }
            }
        }
        // A [Double] item prints the way the shared Rust message prints it.
        let shown: [(Double, String)] = [(1e39, "1e39"), (-1e39, "-1e39"), (.nan, "NaN"), (.infinity, "inf")]
        for (value, text) in shown {
            do {
                _ = try Light.point([value, 0, 0], [1, 1, 1] as [Double])
                throw HealthcheckError.assertion("Light.point with \(text) should throw")
            } catch let error as FragmentColorError {
                guard "\(error)".hasSuffix("got \(text)") else {
                    throw HealthcheckError.assertion("\(text) printed as \(error)")
                }
            }
        }
    }

    // A region of 3 numbers throws, as in Kotlin, where it stopped the app.
    func testTextureRegionRefusesBadLengths() throws {
        do {
            _ = try TextureRegion.from([1, 2, 3])
            throw HealthcheckError.assertion("TextureRegion.from([1, 2, 3]) should throw")
        } catch is FragmentColorError {}
        let region = try TextureRegion.from([1, 2, 3, 4])
        try assertEqual(region.sizeWidth, 3, "TextureRegion.from width")
    }

    // Shader.set shapes each Swift value to the uniform's declared WGSL type.
    func testShaderSetValues() throws {
        let shader = try Shader(source: """
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
        """)

        try shader.set("count", 3)
        try assertEqual(shader.get("count"), .uInt(3), "u32 from an Int literal")
        try shader.set("cell", [2, -1])
        try assertEqual(shader.get("cell"), .iVec2([2, -1]), "vec2<i32> from [Int]")
        try shader.set("rot", [1.0, 2.0, 3.0, 4.0])
        try assertEqual(shader.get("rot"), .mat2([1, 2, 3, 4]), "mat2x2 from four numbers")
        try shader.set("offset", [Int32(5), Int32(6)])
        try assertEqual(shader.get("offset"), .vec2([5, 6]), "vec2<f32> from [Int32]")

        for (label, value) in [("three numbers", [1.0, 2.0, 3.0]), ("an empty list", [])] {
            do {
                try shader.set("offset", value)
                throw HealthcheckError.assertion("set(\"offset\", \(label)) should throw")
            } catch is FragmentColorError {}
        }
        try assertEqual(shader.get("offset"), .vec2([5, 6]), "vec2 kept after rejected sets")

        let blob = Data((0..<32).map { UInt8($0) })
        try shader.set("particles", blob)
        try assertEqual(shader.get("particles"), .bytes(blob), "storage root from Data")
    }

    // Vertex and Instance numbers take the type the shader declares when the mesh attaches.
    // Attaching reads no GPU, so this runs on any simulator.
    func testVertexNumbersTakeShaderTypes() throws {
        let source = """
            @vertex fn vs_main(
              @location(0) position: vec2<f32>,
              @location(1) id: u32,
              @location(2) cell: vec2<i32>,
            ) -> @builtin(position) vec4<f32> {
              return vec4f(position, f32(id) + f32(cell.x), 1.);
            }
            @fragment fn main() -> @location(0) vec4<f32> { return vec4f(1., 0., 0., 1.); }
        """

        let vertices = Mesh()
        vertices.addVertex(try Vertex([0.0, 0.0]).set("id", 7).set("cell", [-1, 2]))
        try Shader(source).addMesh(vertices)

        let instanced = Mesh()
        instanced.addVertex(try Vertex([0.0, 0.0]).set("cell", [-1, 2]))
        try instanced.addInstance(["id": [7]])
        try Shader(source).addMesh(instanced)

        let negative = Mesh()
        negative.addVertex(try Vertex([0.0, 0.0]).set("id", 7).set("cell", [0, 0]))
        negative.addVertex(try Vertex([1.0, 0.0]).set("id", -1).set("cell", [0, 0]))
        do {
            try Shader(source).addMesh(negative)
            throw HealthcheckError.assertion("attaching id = -1 to a u32 input should throw")
        } catch FragmentColorError.Shader(let message) {
            guard message.contains("expected u32, got -1 at vertex 1") else {
                throw HealthcheckError.assertion("-1 for a u32: unexpected message \(message)")
            }
        }

        for (label, values) in [("five numbers", [1.0, 2.0, 3.0, 4.0, 5.0]), ("an empty list", [])] {
            do {
                _ = try Instance().set("offset", values)
                throw HealthcheckError.assertion("Instance.set with \(label) should throw")
            } catch is FragmentColorError {}
        }
    }
}
