//
//  Region+Extensions.swift
//  FragmentColor
//
//  Convenience layer on top of the uniffi-generated `TextureRegionMobile`
//  record. The Rust-side `TextureRegion` exposes `from([x, y, w, h])`,
//  `from([x, y, z, w, h, d])`, `with_stride(_)`, and `with_rows(_)` as
//  builder-style methods; the uniffi record is plain fields, so these
//  extensions reproduce the same call shape on Swift.
//

import Foundation

extension TextureRegionMobile {
    // MARK: - From rectangle / box

    /// `[w, h]` — full size, origin (0, 0, 0).
    public static func from(_ rect: [UInt32]) -> TextureRegionMobile {
        switch rect.count {
        case 2:
            return TextureRegionMobile(
                originX: 0, originY: 0, originZ: 0,
                sizeWidth: rect[0], sizeHeight: rect[1], sizeDepth: 1,
                bytesPerRow: nil, rowsPerImage: nil
            )
        case 4:
            return TextureRegionMobile(
                originX: rect[0], originY: rect[1], originZ: 0,
                sizeWidth: rect[2], sizeHeight: rect[3], sizeDepth: 1,
                bytesPerRow: nil, rowsPerImage: nil
            )
        case 6:
            return TextureRegionMobile(
                originX: rect[0], originY: rect[1], originZ: rect[2],
                sizeWidth: rect[3], sizeHeight: rect[4], sizeDepth: rect[5],
                bytesPerRow: nil, rowsPerImage: nil
            )
        default:
            // Fall back to "whole texture" (all zeros) — same semantics as
            // `TextureRegion::default()`. Better than a partial / wrong region.
            return TextureRegionMobile(
                originX: 0, originY: 0, originZ: 0,
                sizeWidth: 0, sizeHeight: 0, sizeDepth: 0,
                bytesPerRow: nil, rowsPerImage: nil
            )
        }
    }

    /// `[Int]` form for convenient literal use in examples.
    public static func from(_ rect: [Int]) -> TextureRegionMobile {
        return TextureRegionMobile.from(rect.map { UInt32($0) })
    }

    // MARK: - Builder-style setters

    /// Set `bytes_per_row` (must be a multiple of 256 when passed to
    /// `Texture.writeRegion`). Returns a new value — `TextureRegionMobile`
    /// is a value type.
    public func withStride(_ bytesPerRow: UInt32) -> TextureRegionMobile {
        return TextureRegionMobile(
            originX: originX, originY: originY, originZ: originZ,
            sizeWidth: sizeWidth, sizeHeight: sizeHeight, sizeDepth: sizeDepth,
            bytesPerRow: bytesPerRow, rowsPerImage: rowsPerImage
        )
    }
    public func withStride(_ bytesPerRow: Int) -> TextureRegionMobile {
        return withStride(UInt32(bytesPerRow))
    }

    /// Set `rows_per_image`. Defaults to `sizeHeight` when unset; override
    /// for layered or 3D uploads with non-default packing.
    public func withRows(_ rowsPerImage: UInt32) -> TextureRegionMobile {
        return TextureRegionMobile(
            originX: originX, originY: originY, originZ: originZ,
            sizeWidth: sizeWidth, sizeHeight: sizeHeight, sizeDepth: sizeDepth,
            bytesPerRow: bytesPerRow, rowsPerImage: rowsPerImage
        )
    }
    public func withRows(_ rowsPerImage: Int) -> TextureRegionMobile {
        return withRows(UInt32(rowsPerImage))
    }
}
