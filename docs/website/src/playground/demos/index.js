// Demos the playground offers, in picker order.

import cells from "./cells.js";
import particles from "./particles.js";
import life from "./life.js";
import shapes from "./shapes.js";
import kaleidoscope from "./kaleidoscope.js";
import halftone from "./halftone.js";
import raymarch from "./raymarch.js";
import postfx from "./postfx.js";
import { ARTWORK_DEMOS } from "./artworks.js";

export const DEMOS = [cells, particles, life, shapes, kaleidoscope, halftone, raymarch, postfx, ...ARTWORK_DEMOS];
