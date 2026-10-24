// swift-tools-version: 5.9
//
// FragmentColor Swift Package — local dev variant.
//
// The `FragmentColor` target is a thin Swift layer on top of the
// uniffi-generated sources in `Sources/FragmentColor/generated/` plus the
// pre-built `FragmentColorFFI.xcframework` at `../../build/ios-macos/`. Run
// `./fc build_ios` at the repo root to build it. Besides the iOS slices, it
// holds a macOS arm64 slice for `swift build` and `swift run` on a Mac when the
// `aarch64-apple-darwin` Rust target is installed.
//
// The binary target is deliberately named `FragmentColorFFI` so the generated
// Swift's `#if canImport(FragmentColorFFI); import FragmentColorFFI; #endif`
// resolves against it.
//
// For Swift Package Manager consumers, use the root `Package.swift` which
// pulls the xcframework from a GitHub Release asset instead of a local path.

import PackageDescription

let package = Package(
    name: "FragmentColor",
    platforms: [
        .iOS(.v16),
        .macOS(.v12),
    ],
    products: [
        .library(
            name: "FragmentColor",
            targets: ["FragmentColor"]
        ),
    ],
    dependencies: [],
    targets: [
        .target(
            name: "FragmentColor",
            dependencies: [
                .target(name: "FragmentColorFFI"),
            ],
            path: "Sources/FragmentColor",
            exclude: [],
            swiftSettings: []
        ),
        .binaryTarget(
            name: "FragmentColorFFI",
            path: "../../build/ios-macos/FragmentColorFFI.xcframework"
        ),
    ]
)
