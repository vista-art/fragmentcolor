// Geometry for the Hello, million triangles! figures, computed from the same numbers as the shaders.

type Point = [number, number];

/** Step 1's corners in normalized device coordinates, in `vs_main`'s order. */
const CORNERS: Point[] = [
  [-0.7, -0.404],
  [0.7, -0.404],
  [0.0, 0.808],
];

/** Step 4's vertex colors, one per corner. */
const COLORS = [
  [0.95, 0.3, 0.42],
  [0.3, 0.85, 0.55],
  [0.3, 0.55, 0.95],
];

function polyline(points: Point[], closed = false): string {
  const path = points.map(([x, y], i) => `${i ? "L" : "M"} ${x.toFixed(1)} ${y.toFixed(1)}`).join(" ");
  return closed ? `${path} Z` : path;
}

// Zero-length strokes with round caps, which draw as dots.
function dots(points: Point[]): string {
  return points.map(([x, y]) => `M ${x.toFixed(1)} ${y.toFixed(1)} h 0.01`).join(" ");
}

/** Step 7's PCG hash, in 32-bit unsigned arithmetic. */
export function hash(value: number): number {
  const state = (Math.imul(value, 747796405) + 2891336453) >>> 0;
  const word = Math.imul((state >>> ((state >>> 28) + 4)) ^ state, 277803737) >>> 0;
  return ((word >>> 22) ^ word) >>> 0;
}

/** Step 7's `random`: a number from 0 to 1 for each seed. */
export function random(seed: number): number {
  return hash(seed) / 4294967295;
}

// The rasterizer covers a pixel when its center lies on the inner side of all three edges.
function covers([a, b, c]: Point[], p: Point): boolean {
  const edge = (from: Point, to: Point) => (to[0] - from[0]) * (p[1] - from[1]) - (to[1] - from[1]) * (p[0] - from[0]);
  const sides = [edge(a, b), edge(b, c), edge(c, a)];
  return sides.every((side) => side >= 0) || sides.every((side) => side <= 0);
}

// Step 1's triangle on a canvas 18 pixels wide and 11 tall, each pixel 28 units across.
const RASTER = { columns: 18, rows: 11, cell: 28, left: 16, top: 16 };
const rasterPixel = ([x, y]: Point): Point => [((x + 1) / 2) * RASTER.columns, ((1 - y) / 2) * RASTER.rows];
const rasterFigure = ([column, row]: Point): Point => [RASTER.left + column * RASTER.cell, RASTER.top + row * RASTER.cell];
const rasterCorners = CORNERS.map(rasterPixel);
const centers = Array.from({ length: RASTER.columns * RASTER.rows }, (_, i): Point => [
  (i % RASTER.columns) + 0.5,
  Math.floor(i / RASTER.columns) + 0.5,
]);
const covered = centers.filter((center) => covers(rasterCorners, center));
// The callout points at the covered pixel nearest the middle of the right edge.
const rightEdge: Point = [(rasterCorners[1][0] + rasterCorners[2][0]) / 2, (rasterCorners[1][1] + rasterCorners[2][1]) / 2];
const nearest = covered.reduce((best, center) =>
  Math.hypot(center[0] - rightEdge[0], center[1] - rightEdge[1]) < Math.hypot(best[0] - rightEdge[0], best[1] - rightEdge[1])
    ? center
    : best,
);

export const raster = {
  outline: polyline(rasterCorners.map(rasterFigure), true),
  corners: rasterCorners.map(rasterFigure),
  covered: covered
    .map(([column, row]) => {
      const [x, y] = rasterFigure([column - 0.5, row - 0.5]);
      return `M ${x} ${y} h ${RASTER.cell} v ${RASTER.cell} h ${-RASTER.cell} Z`;
    })
    .join(" "),
  inside: dots(covered.map(rasterFigure)),
  outside: dots(centers.filter((center) => !covered.includes(center)).map(rasterFigure)),
  runs: covered.length,
  sample: rasterFigure(nearest),
};

// Normalized device coordinates on a square 300 units wide.
const square = ([x, y]: Point): Point => [360 + x * 150, 200 - y * 150];

export const ndc = {
  low: square([-1, -1]),
  high: square([1, 1]),
  origin: square([0, 0]),
  outline: polyline(CORNERS.map(square), true),
  corners: CORNERS.map(square),
  labels: CORNERS.map(([x, y]) => `(${x}, ${y})`),
  axis: { right: square([1.25, 0]), up: square([0, 1.1]), left: square([-1, 0]), down: square([0, -1]) },
};

// Step 4 on the 720 by 420 capture that figures.json asks for. The vertex stage divides x by the aspect ratio.
const CAPTURE = { width: 720, height: 420 };
const capture = ([x, y]: Point): Point => [
  CAPTURE.width / 2 + ((x * CAPTURE.height) / CAPTURE.width) * (CAPTURE.width / 2),
  CAPTURE.height / 2 - y * (CAPTURE.height / 2),
];
const captureCorners = CORNERS.map(capture);
const format = (values: number[]) => values.map((value) => value.toFixed(2)).join(", ");

// The rasterizer blends position and color with the same weights, one per corner.
function blendAt(weights: number[]) {
  const at = captureCorners.reduce<Point>(
    (sum, [x, y], i) => [sum[0] + x * weights[i], sum[1] + y * weights[i]],
    [0, 0],
  );
  const color = [0, 1, 2].map((channel) => COLORS.reduce((sum, rgb, i) => sum + rgb[channel] * weights[i], 0));
  return { at, color: format(color) };
}

export const gradient = {
  corners: captureCorners,
  colors: COLORS.map(format),
  middle: blendAt([1 / 3, 1 / 3, 1 / 3]),
  pinkish: blendAt([0.6, 0.2, 0.2]),
};

// Step 4's mesh beside a few of step 5's copies, on a jittered grid so they don't overlap.
// One unit of `position` is `COPY.scale` figure units, and so is the shader's wobble of 0.03.
const MESH = { center: [130, 175] as Point, scale: 120 };
const COPY = { scale: 22, left: 400, top: 75, column: 66, row: 58, columns: 5, rows: 4 };
const TINTS = ["inside", "outside", "mark", "accent"] as const;
const place = (center: Point, scale: number) => (corner: Point): Point => [
  center[0] + corner[0] * scale,
  center[1] - corner[1] * scale,
];
const copies = Array.from({ length: COPY.columns * COPY.rows }, (_, i) => {
  const center: Point = [
    COPY.left + (i % COPY.columns) * COPY.column + (random(4 * i + 101) - 0.5) * 22,
    COPY.top + Math.floor(i / COPY.columns) * COPY.row + (random(4 * i + 102) - 0.5) * 18,
  ];
  return { center, tone: TINTS[hash(4 * i + 103) % TINTS.length], phase: random(4 * i + 104) * 2 * Math.PI };
});
// The copy whose wobble the figure draws, near the middle of the field,
// and the one below and to its right, whose tint gets a label.
const FOLLOWED = 7;
const TINTED = FOLLOWED + COPY.columns + 1;
const followed = copies[FOLLOWED];
// Step 5's wobble over one turn of its first sine.
const loop = Array.from({ length: 120 }, (_, i): Point => {
  const time = (i / 119) * ((2 * Math.PI) / 1.3);
  return [
    followed.center[0] + Math.sin(time * 1.3 + followed.phase) * COPY.scale,
    followed.center[1] - Math.cos(time * 0.9 + followed.phase * 1.4) * COPY.scale,
  ];
});

export const instances = {
  mesh: polyline(CORNERS.map(place(MESH.center, MESH.scale)), true),
  meshCorners: CORNERS.map(place(MESH.center, MESH.scale)),
  meshCenter: MESH.center,
  // The copies above and to the right of the followed one stay out, which leaves room for its labels.
  copies: copies.flatMap((copy, i) =>
    i === FOLLOWED - COPY.columns || i === FOLLOWED + 1
      ? []
      : [
          {
            tone: i === FOLLOWED ? "ink" : copy.tone,
            // The followed copy sits where its loop starts.
            path: polyline(CORNERS.map(place(i === FOLLOWED ? loop[0] : copy.center, COPY.scale)), true),
          },
        ],
  ),
  followed: { center: followed.center, copy: loop[0], loop: polyline(loop) },
  tinted: copies[TINTED].center,
};

// Step 6's passes, left to right, each box 140 units wide.
const BOX = { width: 140, height: 76, top: 92, gap: 40, left: 20 };
const boxes = [
  { name: "particles", note: "Pass" },
  { name: "scene", note: "1024 × 1024" },
  { name: "postfx", note: "Pass" },
  { name: "target", note: "canvas" },
].map((box, i) => {
  const left = BOX.left + i * (BOX.width + BOX.gap);
  return { ...box, at: [left, BOX.top] as Point, middle: left + BOX.width / 2 };
});

export const passes = {
  boxes,
  size: [BOX.width, BOX.height] as Point,
  top: BOX.top,
  bottom: BOX.top + BOX.height,
  // Arrows run from one box's right edge to the next box's left edge.
  links: boxes.slice(1).map((box, i): [Point, Point] => [
    [boxes[i].at[0] + BOX.width + 4, BOX.top + BOX.height / 2],
    [box.at[0] - 4, BOX.top + BOX.height / 2],
  ]),
  // `require` arches over the boxes from `postfx` back to `particles`.
  require: `M ${boxes[2].middle} ${BOX.top - 4} C ${boxes[2].middle} ${BOX.top - 70}, ${boxes[0].middle} ${BOX.top - 70}, ${boxes[0].middle} ${BOX.top - 16}`,
  requireTip: [boxes[0].middle, BOX.top - 3] as Point,
  requireLabel: [(boxes[0].middle + boxes[2].middle) / 2, BOX.top - 66] as Point,
};

// Step 6's oversized triangle around the target's square, 66 units per unit of x and y,
// with the `uv` the shader gives each corner.
const cover = ([x, y]: Point): Point => [210 + (x + 1) * 66, 300 - (y + 1) * 66];
const OVERSIZED: Point[] = [
  [-1, -1],
  [3, -1],
  [-1, 3],
];
const UVS: Point[] = [
  [0, 1],
  [2, 1],
  [0, -1],
];
const targetLow = cover([-1, 1]);
const targetHigh = cover([1, -1]);

export const fullscreen = {
  triangle: polyline(OVERSIZED.map(cover), true),
  corners: OVERSIZED.map(cover),
  labels: OVERSIZED.map(([x, y]) => `(${x}, ${y})`),
  uvs: UVS.map(([u, v]) => `uv (${u}, ${v})`),
  target: { at: targetLow, size: [targetHigh[0] - targetLow[0], targetHigh[1] - targetLow[1]] as Point },
  corner: cover([1, 1]),
  // Two points inside the triangle, one past the target's right edge and one above it.
  discarded: [cover([1.9, -0.4]), cover([-0.4, 1.9])],
};

// Step 7's galaxy at time 0: where each index lands, computed as `vs_main` does.
const DISK = { center: [535, 195] as Point, scale: 175 };
function galaxy(index: number) {
  const radius = random(index * 4);
  const arm = (hash(index * 4 + 1) % 3) * 2.094;
  const spread = (random(index * 4 + 2) - 0.5) * (1.2 - radius);
  const brightness = 0.4 + 0.6 * random(index * 4 + 3);
  const angle = arm + radius * 5 + spread;
  const at: Point = [
    DISK.center[0] + Math.cos(angle) * radius * 0.95 * DISK.scale,
    DISK.center[1] - Math.sin(angle) * radius * 0.95 * DISK.scale,
  ];
  return { radius, arm, spread, brightness, at };
}
// The index the figure follows from its seeds to its spot.
const TRACED = 10;
const traced = galaxy(TRACED);
const seeds = [0, 1, 2, 3].map((offset) => TRACED * 4 + offset);

export const disk = {
  dots: dots(Array.from({ length: 1200 }, (_, index) => galaxy(index).at)),
  traced: { index: TRACED, at: traced.at },
  seeds,
  steps: [
    { code: `random(${seeds[0]})`, spoken: `random of ${seeds[0]}`, name: "radius", value: traced.radius },
    { code: `hash(${seeds[1]}) % 3`, spoken: `hash of ${seeds[1]} modulo 3`, name: "arm", value: traced.arm },
    { code: `random(${seeds[2]})`, spoken: `random of ${seeds[2]}`, name: "spread", value: traced.spread },
    { code: `random(${seeds[3]})`, spoken: `random of ${seeds[3]}`, name: "brightness", value: traced.brightness },
  ].map((step) => ({ ...step, value: step.value.toFixed(2) })),
};
