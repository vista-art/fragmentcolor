// swift-tools-version: 5.9
//
// Root Package.swift — consumer-facing entry point for Swift Package
// Manager. Users add FragmentColor to their app via:
//
//     dependencies: [
//       .package(url: "https://github.com/vista-art/fragmentcolor", from: "0.11.0"),
//     ]
//
// The xcframework is downloaded from the matching GitHub Release asset.
// `prepare_release.yml` builds it, attaches it to the draft release, and
// commits the version and checksum below on the commit that the release tags,
// so every tag pins its own binary. `post_publish_update.yml` copies the pin to `main`.
//
// The binary target is named `FragmentColorFFI` because the uniffi-generated
// Swift does `#if canImport(FragmentColorFFI); import FragmentColorFFI; #endif`.
//
// For local development against a freshly-built xcframework, use
// `platforms/swift/Package.swift` instead (it points at `build/ios-macos/`,
// which `./fc build_ios` always produces — with an extra macOS arm64 slice
// when the `aarch64-apple-darwin` Rust target is installed).

import PackageDescription

let fragmentcolorVersion = "0.12.5"
let fragmentcolorChecksum = "ce50a432f7cab41057b08f9b72bfd7a118679e9951db9cda55bc51117c1cf3aa"

let package = Package(
    name: "FragmentColor",
    platforms: [
        .iOS(.v16),
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
                .target(name: "FragmentColorFFI", condition: .when(platforms: [.iOS])),
            ],
            path: "platforms/swift/Sources/FragmentColor",
            exclude: [],
            swiftSettings: []
        ),
        .binaryTarget(
            name: "FragmentColorFFI",
            url: "https://github.com/vista-art/fragmentcolor/releases/download/v\(fragmentcolorVersion)/FragmentColorFFI.xcframework.zip",
            checksum: fragmentcolorChecksum
        ),
    ]
)
