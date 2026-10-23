//
//  ExampleCase.swift
//  FragmentColor
//
//  Base class of the healthcheck tests. The examples read fixtures such as
//  `examples/assets/logo.png` relative to the repository root, as the Rust
//  doctests do. The simulator shares the host's file system, so each test
//  class runs from the checkout this file sits in. A test fails on any GPU
//  error that wgpu reports, and a failing example names its `docs/api` page.
//

import Foundation
import FragmentColor
import XCTest

class ExampleCase: XCTestCase {
    /// The file where the library appends each GPU error that no error scope caught.
    static let gpuErrors = URL(fileURLWithPath: NSTemporaryDirectory())
        .appendingPathComponent("fc-gpu-errors.log")

    override class func setUp() {
        super.setUp()
        let root = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent() // platforms/swift/healthcheck
            .deletingLastPathComponent() // platforms/swift
            .deletingLastPathComponent() // platforms
            .deletingLastPathComponent() // repository root
        FileManager.default.changeCurrentDirectoryPath(root.path)
        setenv("FC_GPU_ERROR_LOG", gpuErrors.path, 1)
    }

    /// Each example starts from the default registry, as each Rust doctest runs in a process of its own.
    override func setUp() {
        super.setUp()
        Shader.setRegistry("https://fragmentcolor.org/shaders/")
        _ = takeGpuErrors()
    }

    /// Fails a test outside `example` that left a GPU error behind.
    override func tearDown() {
        if let errors = takeGpuErrors() {
            XCTFail("GPU error: \(errors)")
        }
        super.tearDown()
    }

    /// Runs one example and fails the test with its `docs/api` page when it throws or raises a GPU error.
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
        if let errors = takeGpuErrors() {
            XCTFail("\(page): GPU error: \(errors)", file: file, line: line)
        }
    }

    /// Empties the GPU error log and returns what it held, or nil when it held nothing.
    private func takeGpuErrors() -> String? {
        let log = Self.gpuErrors
        let text = (try? String(contentsOf: log, encoding: .utf8)) ?? ""
        try? Data().write(to: log)
        let errors = text.trimmingCharacters(in: .whitespacesAndNewlines)
        return errors.isEmpty ? nil : errors
    }
}
