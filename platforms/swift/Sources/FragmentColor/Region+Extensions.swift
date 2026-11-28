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

extension TextureRegion {
    // MARK: - From rectangle / box

    /// `[w, h]`, `[x, y, w, h]` or `[x, y, z, w, h, d]`, as Rust's `TextureRegion::from` reads them.
    /// Throws `FragmentColorError` for any other length, as Kotlin throws `FragmentColorException`.
    public static func from(_ rect: [UInt32]) throws -> TextureRegion {
        try readTextureRegion(rect, "TextureRegion.from")
    }

    /// The pixels a `ScreenRegion` covers, at depth 0.
    /// Throws `FragmentColorError` for a region whose max lies before its min, or with a side of 0, as Kotlin does.
    public static func from(_ region: ScreenRegion) throws -> TextureRegion {
        try readTextureRegion(region, "TextureRegion.from")
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

    /// Set `rows_per_image`. Defaults to `sizeHeight` when unset; override
    /// for layered or 3D uploads with non-default packing.
    public func withRows(_ rowsPerImage: UInt32) -> TextureRegion {
        return TextureRegion(
            originX: originX, originY: originY, originZ: originZ,
            sizeWidth: sizeWidth, sizeHeight: sizeHeight, sizeDepth: sizeDepth,
            bytesPerRow: bytesPerRow, rowsPerImage: rowsPerImage
        )
    }
}

/// The region `[w, h]`, `[x, y, w, h]` or `[x, y, z, w, h, d]` describes.
/// Any other length throws an error that names `call`, the call that reads it.
func readTextureRegion(_ rect: [UInt32], _ call: String) throws -> TextureRegion {
    let (origin, size): ([UInt32], [UInt32])
    switch rect.count {
    case 2: (origin, size) = ([0, 0, 0], [rect[0], rect[1], 1])
    case 4: (origin, size) = ([rect[0], rect[1], 0], [rect[2], rect[3], 1])
    case 6: (origin, size) = (Array(rect[0..<3]), Array(rect[3..<6]))
    default:
        throw FragmentColorError.InvalidInput(message: "\(call): a region takes 2, 4 or 6 numbers, got \(rect.count)")
    }
    return TextureRegion(
        originX: origin[0], originY: origin[1], originZ: origin[2],
        sizeWidth: size[0], sizeHeight: size[1], sizeDepth: size[2],
        bytesPerRow: nil, rowsPerImage: nil
    )
}

/// The pixels a `ScreenRegion` covers, at depth 0, for `call`.
/// A max before its min throws, and then a side of 0, each with an error that names `call`,
/// in the order Rust's `TextureRegion::try_from` checks them, so every binding reports the same one.
func readTextureRegion(_ region: ScreenRegion, _ call: String) throws -> TextureRegion {
    guard region.maxX >= region.minX, region.maxY >= region.minY else {
        throw FragmentColorError.InvalidInput(
            message: "\(call): a region needs each max past its min, got x from \(region.minX) to \(region.maxX) and y from \(region.minY) to \(region.maxY)"
        )
    }
    let (width, height) = (region.maxX - region.minX, region.maxY - region.minY)
    guard width > 0, height > 0 else {
        throw FragmentColorError.InvalidInput(
            message: "\(call): a region needs a width and a height of at least 1, got \(width) by \(height)"
        )
    }
    return TextureRegion(
        originX: region.minX, originY: region.minY, originZ: 0,
        sizeWidth: width, sizeHeight: height, sizeDepth: 1,
        bytesPerRow: nil, rowsPerImage: nil
    )
}
