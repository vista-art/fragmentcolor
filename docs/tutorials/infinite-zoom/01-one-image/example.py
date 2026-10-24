"""Infinite zoom, step 1: one image on the CPU.

Runs the escape-time loop for every pixel of a 1200 by 800 image and writes
`mandelbrot.ppm`, a plain image format most viewers open. No GPU yet: this is
the whole algorithm. Run it with `python example.py`. Plain Python visits all
960,000 pixels one at a time, which takes a while.
"""

from math import cos, pi
from pathlib import Path

# #region: escape
# Iterations beyond which a point counts as inside the set.
LIMIT = 500


# How many steps of z -> z^2 + c the orbit of 0 takes to leave the disk of
# radius 2, or LIMIT if it stays inside. Comparing squared sizes with 4 skips
# a square root.
def escape(c):
    z = 0j
    for n in range(LIMIT):
        if z.real * z.real + z.imag * z.imag > 4:
            return n
        z = z * z + c
    return LIMIT
# #endregion: escape


# #region: image
WIDTH = 1200
HEIGHT = 800


def main():
    # The view: 2.6 units tall, centered a little left of 0, where the set sits.
    center, height = complex(-0.65, 0), 2.6
    pixel = height / HEIGHT
    rgb = bytearray()
    for y in range(HEIGHT):
        for x in range(WIDTH):
            # Rows grow downward, and the imaginary axis points up.
            c = center + complex((x - WIDTH / 2) * pixel, -(y - HEIGHT / 2) * pixel)
            rgb += bytes(color(escape(c)))
    Path("mandelbrot.ppm").write_bytes(b"P6 %d %d 255\n" % (WIDTH, HEIGHT) + rgb)
# #endregion: image


# #region: color
# Near black inside the set. Outside, each iteration moves the color a little along a cosine palette.
def color(count):
    if count == LIMIT:
        return [5, 5, 13]
    t = count * 0.02 + 0.6
    return [int(255 * (0.5 + 0.5 * cos(2 * pi * (t + phase)))) for phase in (0, 0.1, 0.2)]
# #endregion: color


if __name__ == "__main__":
    main()
