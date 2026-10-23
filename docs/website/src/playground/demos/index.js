// Demos the playground offers, in picker order.

import cells from "./cells.js";
import shapes from "./shapes.js";
import kaleidoscope from "./kaleidoscope.js";
import halftone from "./halftone.js";
import raymarch from "./raymarch.js";
import postfx from "./postfx.js";
import particles from "./particles.js";
import { ARTWORK_DEMOS } from "./artworks.js";

export const DEMOS = [cells, shapes, kaleidoscope, halftone, raymarch, postfx, particles, ...ARTWORK_DEMOS];
