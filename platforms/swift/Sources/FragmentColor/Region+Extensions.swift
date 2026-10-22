//
//  Region+Extensions.swift
//  FragmentColor
//
//  Convenience layer on top of the uniffi-generated `TextureRegion`
//  record. The Rust-side `TextureRegion` exposes `from([x, y, w, h])`,
//  `from([x, y, z, w, h, d])`, `with_stride(_)`, and `with_rows(_)` as
//  builder-style methods; the uniffi record is plain fields, so these
//  extensions reproduce the same call shape on Swift.
//

import Foundation

@available(*, deprecated, renamed: "TextureRegion")
public typealias TextureRegionMobile = TextureRegion

extension TextureRegion {
    // MARK: - From rectangle / box

    /// `[w, h]`, `[x, y, w, h]` or `[x, y, z, w, h, d]`, as Rust's `TextureRegion::from` reads them.
    /// Any other length stops the program.
    public static func from(_ rect: [UInt32]) -> TextureRegion {
        guard let region = textureRegion(rect) else {
            preconditionFailure("TextureRegion.from: expected 2, 4 or 6 numbers, got \(rect.count)")
        }
        return region
    }

    /// `[Int]` form for convenient literal use in examples.
    /// A negative number, or a length other than 2, 4 or 6, stops the program.
    public static func from(_ rect: [Int]) -> TextureRegion {
        guard let region = textureRegion(rect) else {
            preconditionFailure("TextureRegion.from: expected 2, 4 or 6 non-negative numbers, got \(rect)")
        }
        return region
    }

    /// The pixels a `ScreenRegion` covers, at depth 0.
    public static func from(_ region: ScreenRegion) -> TextureRegion {
        TextureRegion(
            originX: region.minX, originY: region.minY, originZ: 0,
            sizeWidth: max(region.minX, region.maxX) - min(region.minX, region.maxX),
            sizeHeight: max(region.minY, region.maxY) - min(region.minY, region.maxY),
            sizeDepth: 1,
            bytesPerRow: nil, rowsPerImage: nil
        )
    }

    // MARK: - Builder-style setters

    /// Set `bytes_per_row` for source data with row padding.
    ///
    /// Returns a new value, since `TextureRegion` is a value type.
    /// The stride must be at least the region width times the bytes per pixel.
    /// When unset, rows are tightly packed unless the data is exactly the size
    /// of rows padded to 256 bytes, in which case they read at the padded stride.
    public func withStride(_ bytesPerRow: UInt32) -> TextureRegion {
        return TextureRegion(
            originX: originX, originY: originY, originZ: originZ,
            sizeWidth: sizeWidth, sizeHeight: sizeHeight, sizeDepth: sizeDepth,
            bytesPerRow: bytesPerRow, rowsPerImage: rowsPerImage
        )
    }
    public func withStride(_ bytesPerRow: Int) -> TextureRegion {
        return withStride(UInt32(bytesPerRow))
    }

    /// Set `rows_per_image`. Defaults to `sizeHeight` when unset; override
    /// for layered or 3D uploads with non-default packing.
    public func withRows(_ rowsPerImage: UInt32) -> TextureRegion {
        return TextureRegion(
            originX: originX, originY: originY, originZ: originZ,
            sizeWidth: sizeWidth, sizeHeight: sizeHeight, sizeDepth: sizeDepth,
            bytesPerRow: bytesPerRow, rowsPerImage: rowsPerImage
        )
    }
    public func withRows(_ rowsPerImage: Int) -> TextureRegion {
        return withRows(UInt32(rowsPerImage))
    }
}

/// The region `[w, h]`, `[x, y, w, h]` or `[x, y, z, w, h, d]` describes,
/// or `nil` for any other length or a value `UInt32` can't hold.
func textureRegion<T: BinaryInteger>(_ rect: [T]) -> TextureRegion? {
    let numbers = rect.compactMap { UInt32(exactly: $0) }
    guard numbers.count == rect.count else {
        return nil
    }
    let (origin, size): ([UInt32], [UInt32])
    switch numbers.count {
    case 2: (origin, size) = ([0, 0, 0], [numbers[0], numbers[1], 1])
    case 4: (origin, size) = ([numbers[0], numbers[1], 0], [numbers[2], numbers[3], 1])
    case 6: (origin, size) = (Array(numbers[0..<3]), Array(numbers[3..<6]))
    default: return nil
    }
    return TextureRegion(
        originX: origin[0], originY: origin[1], originZ: origin[2],
        sizeWidth: size[0], sizeHeight: size[1], sizeDepth: size[2],
        bytesPerRow: nil, rowsPerImage: nil
    )
}
