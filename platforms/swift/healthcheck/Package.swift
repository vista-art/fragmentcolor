// swift-tools-version: 5.9
//
// XCTest package that links against the FragmentColor Swift Package. It holds
// the healthcheck and one test per generated example, and `./fc healthcheck ios`
// runs it on an iOS simulator once `./fc build_ios` has produced the xcframework.

import PackageDescription

let package = Package(
    name: "FragmentColorHealthcheck",
    platforms: [.iOS(.v16), .macOS(.v12)],
    dependencies: [
        .package(path: "../"),
    ],
    targets: [
        .testTarget(
            name: "FragmentColorHealthcheck",
            dependencies: [.product(name: "FragmentColor", package: "swift")],
            path: ".",
            exclude: ["Package.swift"],
            sources: ["Healthcheck.swift", "ExampleCase.swift", "GeneratedExamples.swift"]
        ),
    ]
)
