// Infinite Mandelbrot zoom, step 1: one image on the CPU.
//
// Runs the escape-time loop for every pixel of a 1200 by 800 image and writes
// `mandelbrot.ppm`, a plain image format most viewers open. No GPU yet: this is
// the whole algorithm. Run it with `kotlinc example.kt -include-runtime -d step.jar && java -jar step.jar`.

import java.io.File
import kotlin.math.PI
import kotlin.math.cos

// #region: escape
/** Iterations beyond which a point counts as inside the set. */
const val LIMIT = 500

/**
 * How many steps of `z -> z^2 + c` the orbit of 0 takes to leave the disk of
 * radius 2, or [LIMIT] if it stays inside. Comparing squared sizes with 4
 * skips a square root.
 */
fun escape(cr: Double, ci: Double): Int {
    var zr = 0.0
    var zi = 0.0
    for (n in 0 until LIMIT) {
        if (zr * zr + zi * zi > 4) return n
        val next = zr * zr - zi * zi + cr
        zi = 2 * zr * zi + ci
        zr = next
    }
    return LIMIT
}
// #endregion: escape

// #region: image
const val WIDTH = 1200
const val HEIGHT = 800

fun main() {
    // The view: 2.6 units tall, centered a little left of 0, where the set sits.
    val center = doubleArrayOf(-0.65, 0.0)
    val pixel = 2.6 / HEIGHT
    val rgb = java.io.ByteArrayOutputStream()
    rgb.write("P6 $WIDTH $HEIGHT 255\n".toByteArray())
    for (y in 0 until HEIGHT) {
        for (x in 0 until WIDTH) {
            // Rows grow downward, and the imaginary axis points up.
            val cr = center[0] + (x - WIDTH / 2.0) * pixel
            val ci = center[1] - (y - HEIGHT / 2.0) * pixel
            rgb.write(color(escape(cr, ci)))
        }
    }
    File("mandelbrot.ppm").writeBytes(rgb.toByteArray())
}
// #endregion: image

// #region: color
/** Near black inside the set. Outside, each iteration moves the color a little along a cosine palette. */
fun color(count: Int): ByteArray {
    if (count == LIMIT) return byteArrayOf(5, 5, 13)
    val t = count * 0.02 + 0.6
    return doubleArrayOf(0.0, 0.1, 0.2).map { phase -> (255 * (0.5 + 0.5 * cos(2 * PI * (t + phase)))).toInt().toByte() }.toByteArray()
}
// #endregion: color
