//! Infinite zoom, step 1: one image on the CPU.
//!
//! Runs the escape-time loop for every pixel of a 1200 by 800 image, in f64,
//! and writes `mandelbrot.png`. No GPU yet: this is the whole algorithm.

use std::error::Error;
use std::f64::consts::TAU;

// #region: escape
/// Iterations beyond which a point counts as inside the set.
const LIMIT: u32 = 500;

/// How many steps of z -> z^2 + c the orbit of 0 takes to leave the disk of
/// radius 2, or LIMIT if it stays.
fn escape(cr: f64, ci: f64) -> u32 {
    let (mut zr, mut zi) = (0.0, 0.0);
    for n in 0..LIMIT {
        if zr * zr + zi * zi > 4.0 {
            return n;
        }
        let next = zr * zr - zi * zi + cr;
        zi = 2.0 * zr * zi + ci;
        zr = next;
    }
    LIMIT
}
// #endregion: escape

// #region: image
const WIDTH: u32 = 1200;
const HEIGHT: u32 = 800;

fn main() -> Result<(), Box<dyn Error>> {
    // The view: 2.6 units tall, centered a little left of 0, where the set sits.
    let (center, height) = ([-0.65, 0.0], 2.6);
    let pixel = height / f64::from(HEIGHT);
    let mut rgb = Vec::with_capacity((WIDTH * HEIGHT * 3) as usize);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            // Rows grow downward, and the imaginary axis points up.
            let cr = center[0] + (f64::from(x) - f64::from(WIDTH) / 2.0) * pixel;
            let ci = center[1] - (f64::from(y) - f64::from(HEIGHT) / 2.0) * pixel;
            rgb.extend(color(escape(cr, ci)));
        }
    }
    image::save_buffer(
        "mandelbrot.png",
        &rgb,
        WIDTH,
        HEIGHT,
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(())
}
// #endregion: image

// #region: color
/// Near black inside the set. Outside, a cosine palette walks a little per iteration.
fn color(count: u32) -> [u8; 3] {
    if count == LIMIT {
        return [5, 5, 13];
    }
    let t = f64::from(count) * 0.02 + 0.6;
    [0.0, 0.1, 0.2].map(|phase| (255.0 * (0.5 + 0.5 * (TAU * (t + phase)).cos())) as u8)
}
// #endregion: color
