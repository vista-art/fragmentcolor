# Tutorials

Rules for every tutorial: its code under `docs/tutorials/<slug>/` and its page under
`docs/website/src/content/docs/tutorials/`. A tutorial's own `AGENTS.md` adds rules for its steps,
and `PLANNED_TUTORIALS.md` holds the order and the conventions for slugs, sidebar order and drafts.

## Shaders and illustrations

A tutorial teaches with two kinds of pictures, and the illustrations enrich the text between the shaders.

- A live shader (`<Demo>`) appears where the page shows code that renders, and again wherever a step changes that code.
- An illustration appears beside the text that explains a concept. It shows what the words alone can't:
  the coordinate system, how rasterization turns a triangle into pixels, what a distance field measures,
  how a buffer moves between passes.

An illustration is a diagram, or a capture of the canvas with arrows, circles, highlights and labels drawn
on top that point at what the shader is doing. Each one sits next to the paragraph it explains, follows the
site's light and dark themes, reads at phone width, and has alt text that says what it shows. A capture can
be redone from its step when the code changes, so a tool makes it, never a hand-edited screenshot.

### Figures

`docs/website/src/components/Figure.astro` holds every illustration. With `src` it shows a capture and
draws its slot over it; without `src` the slot is the whole diagram, on a panel that follows the theme.
`width` and `height` set the SVG coordinate system, 720 units wide by default, and `alt` is required.
`narrow` keeps a figure at most 320px wide and centered, for a portrait or another small picture that
would stand far too tall at the full width of the text.

```mdx
import Figure from "../../../components/Figure.astro";
import { Arrow, Dot, Label, Line, Path, Rect, Ring } from "../../../components/figure";

<Figure src="/tutorials/drawing-with-distance/circle.webp" height={420}
        alt="A circle's distance field, with its radius and its edge marked." caption="...">
  <Ring at={[360, 210]} r={126} />
  <Arrow from={[360, 210]} to={[486, 210]} label="r" code />
  <Arrow from={[600, 340]} to={[470, 285]} label="d = 0" labelAt="start" tone="mark" />
</Figure>
```

The helpers in `components/figure/` draw arrows, labels, rings, dots, lines, rectangles and paths, each
with a halo that keeps it readable. Their `tone` is `ink`, `muted`, `accent`, `mark`, `inside` or `outside`.
On a diagram the tones follow the site theme; on a capture they keep one light palette, because the
capture itself looks the same in both themes. Labels are SVG text, and they grow on phones.

### Captures

`./fc capture` renders a canvas in headless Chromium with WebGPU and writes a WebP. It reuses the
project's running dev server, or starts one and stops it after.

- `./fc capture figures <tutorial>` redoes every capture listed in `docs/tutorials/<tutorial>/figures.json`
  into `docs/website/public/tutorials/<tutorial>/`. Each entry names a step and pins what moves:
  `{ "name": "circle", "step": "02-circle", "size": [720, 420], "time": 0, "pointer": [360, 210] }`.
  The step needs a `<Demo>` on the tutorial's page, which the tool finds through the page's `slug` when
  its frontmatter sets one. `size` and `pointer` are in figure units.
- `./fc capture demo <step file> --out <file.webp>` captures one step with `--size`, `--time` and `--pointer`.
- `./fc capture thumbs [ids...]` redoes the playground thumbnails, 384 by 240.
- `./fc capture record <script>` plays a scripted interaction on the playground and writes an animated WebP.
  The README's recordings keep their scripts beside them, as `docs/media/*.record.mjs`; `tools/js/record.mjs`
  lists the steps a script can take: click, type, select text in the editor, scroll it, drag a slider, switch tabs.

Captures are twice the figure size in pixels (`--scale 2`), so the annotations line up at any width.
The `<Demo>` component reads `data-time` and `data-pointer` for this, and the tool sets them.
Run `pnpm install` in `platforms/web/healthcheck` and in `docs/website` once before the first capture.
