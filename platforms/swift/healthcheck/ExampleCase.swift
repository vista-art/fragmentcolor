//
//  ExampleCase.swift
//  FragmentColor
//
//  Base class of the generated example tests. The examples read fixtures
//  such as `examples/assets/logo.png` relative to the repository root, as
//  the Rust doctests do. The simulator shares the host's file system, so
//  each test class runs from the checkout this file sits in, and a failing
//  example names its `docs/api` page.
//

import Foundation
import FragmentColor
import XCTest

class ExampleCase: XCTestCase {
    override class func setUp() {
        super.setUp()
        let root = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent() // platforms/swift/healthcheck
            .deletingLastPathComponent() // platforms/swift
            .deletingLastPathComponent() // platforms
            .deletingLastPathComponent() // repository root
        FileManager.default.changeCurrentDirectoryPath(root.path)
    }

    /// Each example starts from the default registry, as each Rust doctest runs in a process of its own.
    override func setUp() {
        super.setUp()
        Shader.setRegistry("https://fragmentcolor.org/shaders/")
    }

    /// Runs one example and fails the test with its `docs/api` page when it throws.
    func example(
        _ page: String,
        file: StaticString = #filePath,
        line: UInt = #line,
        _ body: () async throws -> Void
    ) async {
        do {
            try await body()
        } catch {
            XCTFail("\(page): \(error)", file: file, line: line)
        }
    }
}
