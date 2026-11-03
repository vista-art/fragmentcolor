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
    /// Throws `FragmentColorError` for any other length, as Kotlin throws `FragmentColorException`.
    public static func from(_ rect: [UInt32]) throws -> TextureRegion {
        guard let region = textureRegion(rect) else {
            throw FragmentColorError.Render("TextureRegion.from: expected 2, 4 or 6 numbers, got \(rect.count)")
        }
        return region
    }

    /// The pixels a `ScreenRegion` covers, at depth 0.
    /// Throws `FragmentColorError` for a region whose max isn't past its min, as Kotlin does.
    public static func from(_ region: ScreenRegion) throws -> TextureRegion {
        guard region.maxX > region.minX, region.maxY > region.minY else {
            throw FragmentColorError.Render(
                "TextureRegion.from: a region needs each max past its min, got x from \(region.minX) to \(region.maxX) and y from \(region.minY) to \(region.maxY)"
            )
        }
        return TextureRegion(
            originX: region.minX, originY: region.minY, originZ: 0,
            sizeWidth: region.maxX - region.minX,
            sizeHeight: region.maxY - region.minY,
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

/// The region `[w, h]`, `[x, y, w, h]` or `[x, y, z, w, h, d]` describes,
/// or `nil` for any other length.
func textureRegion(_ rect: [UInt32]) -> TextureRegion? {
    let (origin, size): ([UInt32], [UInt32])
    switch rect.count {
    case 2: (origin, size) = ([0, 0, 0], [rect[0], rect[1], 1])
    case 4: (origin, size) = ([rect[0], rect[1], 0], [rect[2], rect[3], 1])
    case 6: (origin, size) = (Array(rect[0..<3]), Array(rect[3..<6]))
    default: return nil
    }
    return TextureRegion(
        originX: origin[0], originY: origin[1], originZ: origin[2],
        sizeWidth: size[0], sizeHeight: size[1], sizeDepth: size[2],
        bytesPerRow: nil, rowsPerImage: nil
    )
}
