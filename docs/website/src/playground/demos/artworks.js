// The artworks as playground demos. Each is one fragment shader built from
// library slugs and the shared prelude; the gestures that drive them in the
// gallery (hold, click, drag, scroll) are sliders here.

import { ARTWORKS } from "../../demos/artworks/index.js";
import { PRELUDE } from "../../demos/prelude.js";

const SUMMARIES = {
  nebula: "Fractal noise folded over itself until it ignites. The pointer is a gravity well, and press and pulse feed it.",
  swirl: "A cosine palette folded over rings and radar arms. Moving across adds arms, moving up and down winds them into a spiral, drag spins them, and press tightens the rings.",
  rosewindow: "Living stained glass. Moving the pointer across refolds the symmetry, moving it up and down twists the light, and press and pulse make it bloom.",
  plankton: "Bioluminescent cells adrift in a dark tide, pushed aside by the pointer. Press and pulse make them glow.",
  monolith: "An obsidian body carved from signed distance fields and orbited by three moons. Drag orbits the camera, zoom moves it closer, and press charges the core.",
  signal: "A late broadcast curving across old glass. The pointer tunes it, and press and pulse add interference.",
};

// Starting values that replace the shared ones for one artwork. Monolith opens at
// 60% of every slider, with its core charged and rippling.
const VALUES = {
  monolith: { "u.drag": [0.6, 0.6], "u.zoom": 1.64, "u.press": 0.6, "u.pulse": 0.84 },
};

export const ARTWORK_DEMOS = ARTWORKS.map((artwork) => ({
  id: artwork.id,
  title: artwork.title,
  summary: SUMMARIES[artwork.id] ?? artwork.caption,
  kind: "fragment",
  library: artwork.slugs,
  values: { "u.drag": [0.55, 0.32], "u.zoom": 1, "u.press": 0, "u.pulse": 0, ...VALUES[artwork.id] },
  frame: { time: "u.time", resolution: "u.resolution", pointer: "u.mouse" },
  files: { "shader.wgsl": `${artwork.source.trim()}\n\n${PRELUDE.trim()}\n` },
}));
