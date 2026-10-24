// Demos the playground offers, in picker order.

import cells from "./cells.js";
import particles from "./particles.js";
import life from "./life.js";
import mandelbrot from "./mandelbrot.js";
import shapes from "./shapes.js";
import kaleidoscope from "./kaleidoscope.js";
import halftone from "./halftone.js";
import raymarch from "./raymarch.js";
import postfx from "./postfx.js";
import { ARTWORK_DEMOS } from "./artworks.js";
import { supports } from "../../runtime.js";

// A demo that needs a newer FragmentColor than the site loads stays out of the picker.
export const DEMOS = [cells, particles, life, mandelbrot, shapes, kaleidoscope, halftone, raymarch, postfx, ...ARTWORK_DEMOS].filter(
  (demo) => !demo.needs || supports(demo.needs),
);
