// Infinite zoom, step 1: one image on the CPU.
//
// Runs the escape-time loop for every pixel of a 1200 by 800 image and draws it
// into a 2D canvas. Load it as a module on a page with a <canvas>. No GPU yet:
// this is the whole algorithm.

// #region: escape
// Iterations beyond which a point counts as inside the set.
const LIMIT = 500;

// How many steps of z -> z^2 + c the orbit of 0 takes to leave the disk of
// radius 2, or LIMIT if it stays inside. Comparing squared sizes with 4 skips
// a square root.
function escape(cr, ci) {
  let [zr, zi] = [0, 0];
  for (let n = 0; n < LIMIT; n++) {
    if (zr * zr + zi * zi > 4) return n;
    [zr, zi] = [zr * zr - zi * zi + cr, 2 * zr * zi + ci];
  }
  return LIMIT;
}
// #endregion: escape

// #region: image
const WIDTH = 1200;
const HEIGHT = 800;

const canvas = document.querySelector("canvas");
canvas.width = WIDTH;
canvas.height = HEIGHT;
const context = canvas.getContext("2d");
const image = context.createImageData(WIDTH, HEIGHT);

// The view: 2.6 units tall, centered a little left of 0, where the set sits.
const [center, height] = [[-0.65, 0], 2.6];
const pixel = height / HEIGHT;
for (let y = 0; y < HEIGHT; y++) {
  for (let x = 0; x < WIDTH; x++) {
    // Rows grow downward, and the imaginary axis points up.
    const cr = center[0] + (x - WIDTH / 2) * pixel;
    const ci = center[1] - (y - HEIGHT / 2) * pixel;
    image.data.set([...color(escape(cr, ci)), 255], (y * WIDTH + x) * 4);
  }
}
context.putImageData(image, 0, 0);
// #endregion: image

// #region: color
// Near black inside the set. Outside, each iteration moves the color a little along a cosine palette.
function color(count) {
  if (count === LIMIT) return [5, 5, 13];
  const t = count * 0.02 + 0.6;
  return [0, 0.1, 0.2].map((phase) => 255 * (0.5 + 0.5 * Math.cos(2 * Math.PI * (t + phase))));
}
// #endregion: color
