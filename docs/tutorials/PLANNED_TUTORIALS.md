# Planned tutorials

The tutorials teach FragmentColor and shader programming together, one idea at a
time, in the spirit of *The Book of Shaders*. Each one builds on the ones before
it, ends with a working program in every language the site shows, and, where it
fits, adds its final program to the playground as a demo. This file holds the
order and the rough ideas. A tutorial moves out of here when it ships.

## Conventions

- Prose lives in `docs/website/src/content/docs/tutorials/<slug>.mdx`, code in
  `docs/tutorials/<slug>/<step>/example.{rs,js}` (see `docs/tutorials/01-hello-triangle/AGENTS.md`).
- Slugs carry no number. The sidebar order comes from `sidebar.order` in the frontmatter,
  in steps of 10, so a tutorial can slot in between two others without moving a URL.
- A tutorial stays `draft: true` until a student read and a technical review both pass.
- `<Lock>` blocks hold words the maintainer tuned by hand. Agents never edit inside them.

## Launch set

| Order | Tutorial | Playground demo | Status |
|---|---|---|---|
| 10 | Hello, a million triangles! | | Written; student, technical and prose reviews passed |
| 20 | Drawing with distance | Shapes | Written; student, technical and prose reviews passed |
| 90 | Infinite zoom | Mandelbrot (new) | Written; reviews passed |

A fourth tutorial joins the launch only if it says something essential that the
other three leave out.

## The path

### Part 1: first pixels

- **10. Hello, a million triangles!** From a five-line triangle to a million instanced
  triangles, a post-processing pass and 8 million particles moved by a compute shader.
  A tour of the whole library in one sitting.

### Part 2: thinking in fragments

- **20. Drawing with distance.** Pixel coordinates, aspect ratio and the unit
  square; signed distance fields for a circle, a box and a star; smooth union;
  coloring a field by sign and contour; the pointer as an input. Credits
  Inigo Quilez, whose distance functions and coloring the Shapes demo uses.
- **30. Shaping and color.** Shaping functions (`smoothstep`, easing), mixing
  colors, cosine palettes (`palette_iq`), and why a little math beats a lookup table.
- **40. Noise and cells.** A hash first, then value noise, Worley cells and fractal
  Brownian motion, and finally domain warping. Ends at the Cells demo.
- **50. Symmetry and folding.** Polar coordinates, kaleidoscope folds and twirls.
  Builds the Kaleidoscope demo and explains how Rose Window bends its glass.
- **60. Halftone and dithering.** Sampling a texture, grids of dots, ordered
  dithering. The Halftone demo is the last step.

### Part 3: many things at once

- **70. A million particles.** Compute shaders, storage buffers shared by name,
  instanced drawing, and the frame loop that ties them together. Its final program
  is the Particles demo.
- **80. Life.** Several compute passes in a row, a density grid, `atomicAdd`, and
  how simple rules between species grow into colonies. Closes on the Life demo.

### Part 4: past the precision wall

- **90. Infinite zoom.** Written as `tutorials/infinite-zoom`, ending at the Mandelbrot
  demo: one CPU image, the loop in a fragment shader until `f32` rounding breaks it,
  K. I. Martin's perturbation with a `BigInt` reference orbit and rebasing, then
  rescaled iterations, Martin's series and a compute pass down to `1e-1000`.

### Part 5: light and space

- **100. Raymarching.** Distance fields in 3D, a camera ray per pixel, normals from
  the gradient, soft shadows and ambient occlusion. Finishes with the Raymarch demo.
- **110. Post effects.** Rendering to a texture, chaining passes, bloom, grain and
  chromatic aberration, building up to the Post Effects demo.
- **120. Scenes and materials.** Loading a glTF file, physically based materials,
  cameras and lights, and how the scene layer is ordinary shaders and passes.
- **130. Anatomy of an artwork.** Monolith or Nebula taken apart from the shader
  library's slugs up, as a capstone that uses everything above.
