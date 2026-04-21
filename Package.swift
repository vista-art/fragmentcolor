// swift-tools-version: 5.9
//
// Root Package.swift — consumer-facing entry point for Swift Package
// Manager. Users add FragmentColor to their app via:
//
//     dependencies: [
//       .package(url: "https://github.com/vista-art/fragmentcolor", from: "0.11.0"),
//     ]
//
// The xcframework is downloaded from the matching GitHub Release asset;
// `publish_swift.yml` builds it, uploads it, and `post_publish_update.yml`
// rewrites the URL + checksum below to match each release.
//
// For local development against a freshly-built xcframework, use
// `platforms/swift/Package.swift` instead (it points at `build/ios/`).

import PackageDescription

let fragmentcolorVersion = "0.11.0"
let fragmentcolorChecksum = "0000000000000000000000000000000000000000000000000000000000000000"

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
                .target(name: "fragmentcolorFFI", condition: .when(platforms: [.iOS])),
            ],
            path: "platforms/swift/Sources/FragmentColor",
            exclude: [],
            swiftSettings: []
        ),
        .binaryTarget(
            name: "fragmentcolorFFI",
            url: "https://github.com/vista-art/fragmentcolor/releases/download/v\(fragmentcolorVersion)/fragmentcolor.xcframework.zip",
            checksum: fragmentcolorChecksum
        ),
    ]
)
