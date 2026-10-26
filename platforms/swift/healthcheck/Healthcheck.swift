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
