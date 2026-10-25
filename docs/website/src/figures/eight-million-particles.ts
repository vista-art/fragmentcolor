// Geometry for the Eight million particles figures, computed from the same formulas as the shaders.

import { random } from "./01-hello-triangle";

type Point = [number, number];

/** The program's own sizes: particles, and the grid's width and height in cells. */
const COUNT = 8_000_000;
const WIDTH = 1280;
const HEIGHT = 720;

function polyline(points: Point[]): string {
  return points.map(([x, y], i) => `${i ? "L" : "M"} ${x.toFixed(1)} ${y.toFixed(1)}`).join(" ");
}

// The three passes in a row, and the two storage buffers under them.
const PASS = { top: 40, width: 160, height: 60 };
const passes = [
  { name: "clear", stage: "compute", left: 30 },
  { name: "simulate", stage: "compute", left: 280 },
  { name: "draw", stage: "render", left: 530 },
].map((pass) => ({ ...pass, at: [pass.left, PASS.top] as Point, middle: pass.left + PASS.width / 2 }));
const [clearPass, simulatePass, drawPass] = passes;
const particles = { at: [160, 175] as Point, size: [220, 58] as Point };
const counts = { at: [30, 268] as Point, size: [660, 58] as Point };

export const frame = {
  passes,
  size: [PASS.width, PASS.height] as Point,
  bottom: PASS.top + PASS.height,
  links: passes.slice(1).map((pass, i): [Point, Point] => [
    [passes[i].left + PASS.width + 4, PASS.top + PASS.height / 2],
    [pass.left - 4, PASS.top + PASS.height / 2],
  ]),
  particles: { ...particles, note: `${COUNT.toLocaleString("en-US")} × 16 bytes` },
  counts: { ...counts, note: `${WIDTH} × ${HEIGHT} cells` },
  // Where each pass reaches a buffer: `simulate` meets `particles` where the two overlap.
  wires: {
    store: clearPass.middle,
    both: (simulatePass.left + particles.at[0] + particles.size[0]) / 2,
    add: simulatePass.left + PASS.width - 15,
    read: drawPass.middle,
  },
};

// A grid 8 cells wide and 4 tall, each cell 60 units, with row 0 at the bottom as in `simulate`.
const GRID = { columns: 8, rows: 4, cell: 60, left: 150, bottom: 310 };
const cellCorner = (column: number, row: number): Point => [
  GRID.left + column * GRID.cell,
  GRID.bottom - (row + 1) * GRID.cell,
];
// Particles in a loose cluster, placed with the shaders' own `random`. The figure keeps
// those that sit clear of the cell borders and of the count in each cell's top left corner.
const fits = (value: number) => value % 1 > 0.12 && value % 1 < 0.88;
const spots = Array.from({ length: 64 }, (_, i): Point => [
  4.9 + (random(4 * i + 500) + random(4 * i + 501) - 1) * 4.2,
  2.1 + (random(4 * i + 502) + random(4 * i + 503) - 1) * 3.0,
]).filter(
  ([x, y]) =>
    x >= 0 && x < GRID.columns && y >= 0 && y < GRID.rows && fits(x) && fits(y) && !(x % 1 < 0.35 && y % 1 > 0.6),
);
// `vec2<u32>(uv * vec2<f32>(GRID))` rounds each coordinate down to a cell.
const tally = new Map<string, number>();
for (const [x, y] of spots) {
  const key = `${Math.floor(x)},${Math.floor(y)}`;
  tally.set(key, (tally.get(key) ?? 0) + 1);
}
const HIGHLIGHT = { column: 5, row: 2 };

export const grid = {
  cells: Array.from({ length: GRID.columns * GRID.rows }, (_, i) => cellCorner(i % GRID.columns, Math.floor(i / GRID.columns))),
  cell: GRID.cell,
  columns: Array.from({ length: GRID.columns }, (_, column) => GRID.left + (column + 0.5) * GRID.cell),
  rows: Array.from({ length: GRID.rows }, (_, row) => GRID.bottom - (row + 0.5) * GRID.cell),
  left: GRID.left,
  bottom: GRID.bottom,
  dots: spots.map(([x, y]): Point => [GRID.left + x * GRID.cell, GRID.bottom - y * GRID.cell]),
  counts: [...tally].map(([key, count]) => {
    const [column, row] = key.split(",").map(Number);
    const [x, y] = cellCorner(column, row);
    return { at: [x + 6, y + 13] as Point, count };
  }),
  highlight: {
    at: cellCorner(HIGHLIGHT.column, HIGHLIGHT.row),
    // The arrow comes down the cell's right side, clear of the dots in the cell above.
    lane: GRID.left + (HIGHLIGHT.column + 0.75) * GRID.cell,
    sum: `${HIGHLIGHT.row} * ${GRID.columns} + ${HIGHLIGHT.column}`,
    element: HIGHLIGHT.row * GRID.columns + HIGHLIGHT.column,
    count: tally.get(`${HIGHLIGHT.column},${HIGHLIGHT.row}`) ?? 0,
  },
};

// One particle's next two seconds under `simulate`'s rule. On a 720 by 420 canvas `fit` makes
// one unit 210 figure units along both axes, and the figure keeps the band around the middle.
const canvas = ([x, y]: Point): Point => [360 + x * 210, 160 - y * 210];
const attractorAt = (time: number): Point => [Math.cos(time * 0.37) * 0.4, Math.sin(time * 0.61) * 0.4];
const START = 4;
const STEP = 1 / 60;
const spot: Point = [0.55, -0.35];
// A fresh particle's velocity: its direction from the center, turned a quarter, times 0.6.
const distance = Math.hypot(spot[0], spot[1]);
const orbit: Point = [(-spot[1] / distance) * 0.6, (spot[0] / distance) * 0.6];
const pullAt = (position: Point, time: number): Point => {
  const attractor = attractorAt(time);
  const offset: Point = [attractor[0] - position[0], attractor[1] - position[1]];
  const weight = 1 / (offset[0] * offset[0] + offset[1] * offset[1] + 0.05);
  return [offset[0] * weight, offset[1] * weight];
};
const path: Point[] = [spot];
{
  let position = spot;
  let velocity = orbit;
  for (let frame = 0; frame < 120; frame++) {
    const pull = pullAt(position, START + frame * STEP);
    velocity = [(velocity[0] + pull[0] * STEP) * 0.995, (velocity[1] + pull[1] * STEP) * 0.995];
    position = [position[0] + velocity[0] * STEP, position[1] + velocity[1] * STEP];
    path.push(position);
  }
}
// Arrows show directions at one scale: 70 figure units per unit of velocity or pull.
const ARROW = 70;
const tip = (vector: Point): Point => [
  canvas(spot)[0] + vector[0] * ARROW,
  canvas(spot)[1] - vector[1] * ARROW,
];

export const step = {
  particle: canvas(spot),
  velocity: tip(orbit),
  pull: tip(pullAt(spot, START)),
  path: polyline(path.map(canvas)),
  // Two points a few frames apart aim the arrowhead at the end of the path.
  ending: [canvas(path[path.length - 4]), canvas(path[path.length - 1])] as [Point, Point],
  center: canvas([0, 0]),
  attractor: canvas(attractorAt(START)),
  wander: polyline(Array.from({ length: 241 }, (_, i) => canvas(attractorAt(START - 6 + i * 0.05)))),
};

// One pixel on a 1280 by 720 canvas drawn 300 units wide, measured as `draw` measures it.
// `@builtin(position)` holds the pixel's center, and the cell flips y before it rounds down.
const PANEL = { width: 300, top: 64 };
const panelHeight = (PANEL.width * HEIGHT) / WIDTH;
const pixel: Point = [Math.floor(0.3 * WIDTH) + 0.5, Math.floor(0.25 * HEIGHT) + 0.5];
const cell: Point = [Math.floor(pixel[0]), Math.floor(HEIGHT - pixel[1])];
const onPanel = (left: number): Point => [
  left + (pixel[0] / WIDTH) * PANEL.width,
  PANEL.top + (pixel[1] / HEIGHT) * panelHeight,
];

export const flip = {
  size: [PANEL.width, panelHeight] as Point,
  left: [30, PANEL.top] as Point,
  right: [390, PANEL.top] as Point,
  pixel: onPanel(30),
  cell: onPanel(390),
  pixelLabel: `(${pixel[0]}, ${pixel[1]})`,
  cellLabel: `(${cell[0]}, ${cell[1]})`,
  canvas: `${WIDTH} by ${HEIGHT}`,
  bottom: PANEL.top + panelHeight,
};
