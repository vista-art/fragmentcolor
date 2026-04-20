//
//  Healthcheck.swift
//  FragmentColor
//
//  Mirrors platforms/python/healthcheck.py — exercises the full uniffi
//  surface against a real Metal device on an iOS simulator. Run via
//  xcodebuild in `./healthcheck ios`; assertions drive the process exit
//  code so the top-level runner can report pass/fail.
//
//  DOC: This file is the source of truth for Swift code snippets shown on
//  fragmentcolor.org. Examples between `// DOC: <Object>.<method> (begin)`
//  and `// DOC: (end)` markers get extracted at build time and spliced
//  into the generated MDX pages.
//

import FragmentColor

enum HealthcheckError: Error {
    case assertion(String)
}

func assertEqual<T: Equatable>(_ actual: T, _ expected: T, _ label: String) throws {
    guard actual == expected else {
        throw HealthcheckError.assertion("\(label): expected \(expected), got \(actual)")
    }
}

@main
struct Healthcheck {
    static func main() async {
        do {
            try await run()
            print("FragmentColor iOS healthcheck: PASS")
            exit(0)
        } catch {
            print("FragmentColor iOS healthcheck: FAIL — \(error)")
            exit(1)
        }
    }

    static func run() async throws {
        // DOC: Renderer.new (begin)
        let renderer = Renderer()
        // DOC: (end)

        // DOC: Renderer.create_texture_target (begin)
        let target = try await renderer.createTextureTarget(width: 32, height: 64)
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

        try assertEqual(target.size().width, 32, "TextureTarget.width")
        try assertEqual(target.size().height, 64, "TextureTarget.height")
    }
}
