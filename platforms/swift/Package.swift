// swift-tools-version: 5.9
//
// FragmentColor Swift Package.
//
// The `FragmentColor` target is a thin layer on top of the uniffi-generated
// Swift sources that live in `Sources/FragmentColor/generated/` and a
// pre-built xcframework at `../../build/ios/fragmentcolor.xcframework`.
//
// Regenerate both with `./build_ios` at the repo root before building.

import PackageDescription

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
                .target(name: "fragmentcolor", condition: .when(platforms: [.iOS])),
            ],
            path: "Sources/FragmentColor",
            exclude: [],
            swiftSettings: []
        ),
        .binaryTarget(
            name: "fragmentcolor",
            path: "../../build/ios/fragmentcolor.xcframework"
        ),
    ]
)
