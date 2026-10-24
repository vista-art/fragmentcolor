// Infinite zoom, step 1: one image on the CPU.
//
// Runs the escape-time loop for every pixel of a 1200 by 800 image and writes
// `mandelbrot.ppm`, a plain image format most viewers open. No GPU yet: this is
// the whole algorithm. Run it with `swift -O example.swift`.

import Foundation

// #region: escape
/// Iterations beyond which a point counts as inside the set.
let limit = 500

/// How many steps of `z -> z^2 + c` the orbit of 0 takes to leave the disk of
/// radius 2, or `limit` if it stays inside. Comparing squared sizes with 4
/// skips a square root.
func escape(_ cr: Double, _ ci: Double) -> Int {
    var (zr, zi) = (0.0, 0.0)
    for n in 0..<limit {
        if zr * zr + zi * zi > 4 {
            return n
        }
        (zr, zi) = (zr * zr - zi * zi + cr, 2 * zr * zi + ci)
    }
    return limit
}
// #endregion: escape

// #region: color
/// Near black inside the set. Outside, each iteration moves the color a little along a cosine palette.
func color(_ count: Int) -> [UInt8] {
    if count == limit {
        return [5, 5, 13]
    }
    let t = Double(count) * 0.02 + 0.6
    return [0, 0.1, 0.2].map { phase in UInt8(255 * (0.5 + 0.5 * cos(2 * Double.pi * (t + phase)))) }
}
// #endregion: color

// #region: image
let width = 1200
let height = 800

// The view: 2.6 units tall, centered a little left of 0, where the set sits.
let (center, viewHeight) = ((-0.65, 0.0), 2.6)
let pixel = viewHeight / Double(height)
var rgb = Data("P6 \(width) \(height) 255\n".utf8)
for y in 0..<height {
    for x in 0..<width {
        // Rows grow downward, and the imaginary axis points up.
        let cr = center.0 + (Double(x) - Double(width) / 2) * pixel
        let ci = center.1 - (Double(y) - Double(height) / 2) * pixel
        rgb.append(contentsOf: color(escape(cr, ci)))
    }
}
try rgb.write(to: URL(fileURLWithPath: "mandelbrot.ppm"))
// #endregion: image
