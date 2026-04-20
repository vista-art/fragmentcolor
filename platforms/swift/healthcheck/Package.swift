// swift-tools-version: 5.9
//
// Minimal SPM executable that links against the FragmentColor Swift Package
// and runs a headless render smoke test. Invoked by `./healthcheck ios`
// once `./build_ios` has produced the xcframework.

import PackageDescription

let package = Package(
    name: "FragmentColorHealthcheck",
    platforms: [.iOS(.v16)],
    products: [
        .executable(name: "fragmentcolor-healthcheck", targets: ["FragmentColorHealthcheck"]),
    ],
    dependencies: [
        .package(path: "../"),
    ],
    targets: [
        .executableTarget(
            name: "FragmentColorHealthcheck",
            dependencies: [.product(name: "FragmentColor", package: "swift")],
            path: ".",
            exclude: ["Package.swift"],
            sources: ["Healthcheck.swift"]
        ),
    ]
)
