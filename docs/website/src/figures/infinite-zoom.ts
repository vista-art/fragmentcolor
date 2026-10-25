// Geometry for the Infinite Mandelbrot zoom figures, computed from the same formulas as the steps.

type Point = [number, number];
type Complex = [number, number];

function polyline(points: Point[]): string {
  return points.map(([x, y], i) => `${i ? "L" : "M"} ${x.toFixed(1)} ${y.toFixed(1)}`).join(" ");
}

/** Steps until the orbit of 0 leaves the disk of radius 2, or `limit`, as in step 1. */
function escape([cr, ci]: Complex, limit: number): number {
  let [zr, zi] = [0, 0];
  for (let n = 0; n < limit; n++) {
    if (zr * zr + zi * zi > 4) return n;
    [zr, zi] = [zr * zr - zi * zi + cr, 2 * zr * zi + ci];
  }
  return limit;
}

/** The orbit of 0 under z^2 + c, up to and including its first step outside the disk. */
function orbit([cr, ci]: Complex, steps: number): Complex[] {
  const points: Complex[] = [[0, 0]];
  let [zr, zi] = [0, 0];
  for (let n = 0; n < steps && zr * zr + zi * zi <= 4; n++) {
    [zr, zi] = [zr * zr - zi * zi + cr, 2 * zr * zi + ci];
    points.push([zr, zi]);
  }
  return points;
}

/** The set as one path of horizontal runs, `cell` units of the plane tall. */
function silhouette(
  [left, top]: Complex,
  [right, bottom]: Complex,
  cell: number,
  place: (c: Complex) => Point,
): string {
  const parts: string[] = [];
  const columns = Math.round((right - left) / cell);
  for (let ci = top - cell / 2; ci > bottom; ci -= cell) {
    let run: number | null = null;
    for (let column = 0; column <= columns; column++) {
      const inside = column < columns && escape([left + (column + 0.5) * cell, ci], 300) === 300;
      if (inside && run === null) run = column;
      if (!inside && run !== null) {
        const [x0, y0] = place([left + run * cell, ci + cell / 2]);
        const [x1, y1] = place([left + column * cell, ci - cell / 2]);
        parts.push(`M${x0.toFixed(1)},${y0.toFixed(1)}H${x1.toFixed(1)}V${y1.toFixed(1)}H${x0.toFixed(1)}z`);
        run = null;
      }
    }
  }
  return parts.join("");
}

// Two orbits on the plane, 120 figure units per unit. The circle of radius 2 runs past the top and bottom edges.
const ORBITS = { origin: [400, 230] as Point, scale: 120 };
const onPlane = ([re, im]: Complex): Point => [ORBITS.origin[0] + re * ORBITS.scale, ORBITS.origin[1] - im * ORBITS.scale];
// Just outside the main cardioid the orbit spirals out; just inside, it spirals in to a fixed point.
const leaving = orbit([0.42, 0.13], 20);
const staying = orbit([0.33, -0.2], 16);
const inside = leaving.slice(0, -1).map(onPlane);
const lastInside = inside[inside.length - 1];
const outside = onPlane(leaving[leaving.length - 1]);
// The step out of the disk runs off the top of the figure.
const exitAt = (14 - lastInside[1]) / (outside[1] - lastInside[1]);

export const orbits = {
  set: silhouette([-2.05, 1.2], [0.5, -1.2], 0.0125, onPlane),
  origin: ORBITS.origin,
  radius: 2 * ORBITS.scale,
  leaving: { path: polyline(inside), points: inside, steps: leaving.length - 1 },
  exit: [lastInside, [lastInside[0] + exitAt * (outside[0] - lastInside[0]), 14]] as [Point, Point],
  staying: { path: polyline(staying.map(onPlane)), points: staying.map(onPlane) },
  ticks: [-2, -1, 1, 2].map((value) => ({ value, x: onPlane([value, 0])[0] })),
  unitUp: onPlane([0, 1])[1],
  unitDown: onPlane([0, -1])[1],
  leavingC: onPlane([0.42, 0.13]),
  stayingC: onPlane([0.33, -0.2]),
};

// Step 1's view, 2.6 units tall around (-0.65, 0), drawn as a coarse pixel grid.
const PIXELS = { left: 40, top: 40, width: 480, height: 320, columns: 12, rows: 8, center: [-0.65, 0] as Complex, span: 2.6 };
const unit = PIXELS.span / PIXELS.height;
const inView = ([re, im]: Complex): Point => [
  PIXELS.left + PIXELS.width / 2 + (re - PIXELS.center[0]) / unit,
  PIXELS.top + PIXELS.height / 2 - (im - PIXELS.center[1]) / unit,
];
const cell = PIXELS.width / PIXELS.columns;
const chosen = { column: 9, row: 2 };

export const pixels = {
  view: { at: [PIXELS.left, PIXELS.top] as Point, size: [PIXELS.width, PIXELS.height] as Point },
  set: silhouette([-2.6, 1.3], [1.3, -1.3], 0.0125, inView),
  grid: [
    ...Array.from({ length: PIXELS.columns - 1 }, (_, i) => `M${PIXELS.left + (i + 1) * cell},${PIXELS.top}v${PIXELS.height}`),
    ...Array.from({ length: PIXELS.rows - 1 }, (_, i) => `M${PIXELS.left},${PIXELS.top + (i + 1) * cell}h${PIXELS.width}`),
  ].join(""),
  origin: inView([0, 0]),
  cell,
  pixel: [PIXELS.left + chosen.column * cell, PIXELS.top + chosen.row * cell] as Point,
  // Step 1 maps pixel (x, y) to c through its top left corner.
  c: [
    PIXELS.center[0] + (chosen.column * cell - PIXELS.width / 2) * unit,
    PIXELS.center[1] - (chosen.row * cell - PIXELS.height / 2) * unit,
  ] as Complex,
};

// Step 1's cosine palette over the first 100 counts, with t = 0.02 count + 0.6.
const PALETTE = { left: 60, width: 550, top: 30, height: 160, counts: 100 };
const channel = (count: number, phase: number) => 0.5 + 0.5 * Math.cos(2 * Math.PI * (count * 0.02 + 0.6 + phase));
const paletteX = (count: number) => PALETTE.left + (count / PALETTE.counts) * PALETTE.width;
const paletteY = (value: number) => PALETTE.top + (1 - value) * PALETTE.height;

export const palette = {
  left: PALETTE.left,
  right: PALETTE.left + PALETTE.width,
  levels: [0, 0.5, 1].map((value) => ({ value, y: paletteY(value) })),
  curves: [0, 0.1, 0.2].map((phase) =>
    polyline(Array.from({ length: 401 }, (_, i) => [paletteX(i / 4), paletteY(channel(i / 4, phase))] as Point)),
  ),
  strip: {
    y: PALETTE.top + PALETTE.height + 18,
    height: 34,
    width: PALETTE.width / PALETTE.counts,
    cells: Array.from({ length: PALETTE.counts }, (_, count) => ({
      x: paletteX(count),
      // Step 1 truncates each channel to a byte.
      color: `rgb(${[0, 0.1, 0.2].map((phase) => Math.trunc(255 * channel(count, phase))).join(",")})`,
    })),
  },
  ticks: [0, 25, 50, 75, 100].map((count) => ({ count, x: paletteX(count) })),
  // Where each curve ends, for its label.
  ends: [0, 0.1, 0.2].map((phase) => [paletteX(PALETTE.counts) + 10, paletteY(channel(PALETTE.counts, phase))] as Point),
};

// f32 helpers: the value a double rounds to, and the gap to the next f32 above it.
const f32 = Math.fround;
function ulp(value: number): number {
  const bits = new Float32Array([value]);
  const view = new Int32Array(bits.buffer);
  view[0] += value >= 0 ? 1 : -1;
  return Math.abs(bits[0] - f32(value));
}

const TARGET: Complex = [-0.7432918908524302, 0.1312405523087976];

/**
 * The f32 block that holds step 2's center, in a 720x420 capture of a view `height` tall.
 * Every pixel inside it rounds to the same c, and its edges sit halfway to the neighbors.
 */
function centerBlock(height: number): { at: Point; size: Point } {
  const unit = height / 420;
  const [re, im] = [f32(TARGET[0]), f32(TARGET[1])];
  const [wide, tall] = [ulp(re), ulp(im)];
  const left = 360 + (re - wide / 2 - TARGET[0]) / unit;
  const top = 210 - (im + tall / 2 - TARGET[1]) / unit;
  return { at: [left, top], size: [wide / unit, tall / unit] };
}

/** A patch of pixels around TARGET in a view 1e-6 tall on 800 pixels, grouped by the f32 c each rounds to. */
const PATCH = { columns: 132, rows: 33, size: 5, left: 30, top: 20, canvas: 800 };
function patch() {
  const pixel = 1e-6 / PATCH.canvas;
  const owner: string[][] = [];
  const shades: number[][] = [];
  for (let row = 0; row < PATCH.rows; row++) {
    owner.push([]);
    shades.push([]);
    for (let column = 0; column < PATCH.columns; column++) {
      // As in step 2: an f32 center plus an f32 offset, rounded to f32.
      const re = f32(f32(TARGET[0]) + f32((column + 0.5 - PATCH.columns / 2) * pixel));
      const im = f32(f32(TARGET[1]) - f32((row + 0.5 - PATCH.rows / 2) * pixel));
      const steps = Math.round((re - f32(TARGET[0])) / ulp(re)) + Math.round((im - f32(TARGET[1])) / ulp(im));
      owner[row].push(`${re},${im}`);
      shades[row].push(((steps % 2) + 2) % 2);
    }
  }
  // The block under the middle pixel, outlined.
  const middle = owner[Math.floor(PATCH.rows / 2)][Math.floor(PATCH.columns / 2)];
  let [x0, y0, x1, y1] = [Infinity, Infinity, -Infinity, -Infinity];
  owner.forEach((row, r) =>
    row.forEach((key, c) => {
      if (key !== middle) return;
      [x0, y0, x1, y1] = [Math.min(x0, c), Math.min(y0, r), Math.max(x1, c + 1), Math.max(y1, r + 1)];
    }),
  );
  // Each shade becomes one path of horizontal runs.
  const runs = ["", ""];
  shades.forEach((row, r) => {
    let start = 0;
    for (let c = 1; c <= row.length; c++) {
      if (c < row.length && row[c] === row[start]) continue;
      const [x, y] = [PATCH.left + start * PATCH.size, PATCH.top + r * PATCH.size];
      runs[row[start]] += `M${x},${y}h${(c - start) * PATCH.size}v${PATCH.size}h${-(c - start) * PATCH.size}z`;
      start = c;
    }
  });
  // Faint lines between the pixels.
  const grid = [
    ...Array.from({ length: PATCH.columns - 1 }, (_, i) => `M${PATCH.left + (i + 1) * PATCH.size},${PATCH.top}v${PATCH.rows * PATCH.size}`),
    ...Array.from({ length: PATCH.rows - 1 }, (_, i) => `M${PATCH.left},${PATCH.top + (i + 1) * PATCH.size}h${PATCH.columns * PATCH.size}`),
  ].join("");
  return {
    runs,
    grid,
    frame: { at: [PATCH.left, PATCH.top] as Point, size: [PATCH.columns * PATCH.size, PATCH.rows * PATCH.size] as Point },
    block: {
      at: [PATCH.left + x0 * PATCH.size, PATCH.top + y0 * PATCH.size] as Point,
      size: [(x1 - x0) * PATCH.size, (y1 - y0) * PATCH.size] as Point,
      pixels: [x1 - x0, y1 - y0] as Point,
    },
  };
}

export const blocks = { ...patch(), gaps: [ulp(f32(TARGET[0])), ulp(f32(TARGET[1]))] as Complex };

// Captures of steps 2 to 4 are 720x420 figure units, centered on TARGET.
const captureUnit = (height: number) => height / 420;

export const captures = {
  /** Where a point of the plane lands in step 2's capture at zoom 0, a view 3 units tall. */
  overview: (c: Complex): Point => [360 + (c[0] - TARGET[0]) / captureUnit(3), 210 - (c[1] - TARGET[1]) / captureUnit(3)],
  overviewUnit: 1 / captureUnit(3),
  /** The f32 block around the center at 1e-6, in step 2's capture. */
  blockAt6: centerBlock(1e-6),
  /** The same block at 1e-7, drawn on step 3's capture. */
  blockAt7: centerBlock(1e-7),
};

// A reference orbit and a nearby pixel's orbit, both from 0.
const PERTURBATION = { origin: [270, 320] as Point, scale: 300, c: [0.36, 0.1] as Complex, delta: [0.03, 0.02] as Complex };
const onPerturbation = ([re, im]: Complex): Point => [
  PERTURBATION.origin[0] + re * PERTURBATION.scale,
  PERTURBATION.origin[1] - im * PERTURBATION.scale,
];
const referenceOrbit = orbit(PERTURBATION.c, 6).map(onPerturbation);
const pixelOrbit = orbit([PERTURBATION.c[0] + PERTURBATION.delta[0], PERTURBATION.c[1] + PERTURBATION.delta[1]], 6).map(
  onPerturbation,
);

export const perturbation = {
  reference: { path: polyline(referenceOrbit), points: referenceOrbit },
  pixel: { path: polyline(pixelOrbit), points: pixelOrbit },
  differences: referenceOrbit.slice(1).map((point, i) => [point, pixelOrbit[i + 1]] as [Point, Point]),
};

// View heights from 1 to 1e-1000 on one axis, broken twice: 0 to 50 decades, 300 to 310, 995 to 1000.
const LADDER = [
  { from: 0, to: 50, top: 40, bottom: 260 },
  { from: 300, to: 310, top: 290, bottom: 320 },
  { from: 995, to: 1000, top: 350, bottom: 370 },
];
function ladderY(decades: number): number {
  const segment = LADDER.find(({ from, to }) => decades >= from && decades <= to);
  if (!segment) throw new Error(`the ladder has no room for 1e-${decades}`);
  return segment.top + ((decades - segment.from) / (segment.to - segment.from)) * (segment.bottom - segment.top);
}

export const ladder = {
  segments: LADDER.map(({ top, bottom }) => [top, bottom] as Point),
  breaks: [275, 335],
  ticks: [0, 10, 20, 30, 40, 50, 300, 1000].map((decades) => ({ decades, y: ladderY(decades) })),
  // Step 2's blocks: f32 neighbors near TARGET sit 6e-8 apart, dozens of pixels in a view 1e-6 tall.
  digits: ladderY(6),
  // The smallest normal f32, 2^-126, is one pixel of a view about 1e-35 tall on 800 pixels.
  normal: ladderY(-Math.log10(2 ** -126 * 800)),
  denormal: ladderY(-Math.log10(2 ** -149 * 800)),
  double: ladderY(-Math.log10(2 ** -1022 * 800)),
  target: ladderY(1000),
  top: ladderY(0),
};

// Step 4's reference orbit and series, as in its computeReference, kept to what the chart needs.
type Wide = [number, number, number];
const ZERO: Wide = [0, 0, 0];
const ONE: Wide = [1, 0, 0];
function normalize(re: number, im: number, exponent: number): Wide {
  const size = Math.max(Math.abs(re), Math.abs(im));
  if (size === 0) return ZERO;
  if (size > 2 ** -64 && size < 2 ** 64) return [re, im, exponent];
  const k = Math.round(Math.log2(size));
  return [re * 2 ** -k, im * 2 ** -k, exponent + k];
}
function add(a: Wide, b: Wide): Wide {
  const exponent = Math.max(a[2], b[2]);
  const [sa, sb] = [2 ** (a[2] - exponent), 2 ** (b[2] - exponent)];
  return normalize(a[0] * sa + b[0] * sb, a[1] * sa + b[1] * sb, exponent);
}
function multiply(a: Wide, b: Wide): Wide {
  return normalize(a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0], a[2] + b[2]);
}
function times(a: Wide, re: number, im: number): Wide {
  return normalize(a[0] * re - a[1] * im, a[0] * im + a[1] * re, a[2]);
}
const log2Size = (a: Wide) => Math.log2(Math.hypot(a[0], a[1])) + a[2];

function parse(text: string, bits: number): bigint {
  const [whole, fraction = ""] = text.replace("-", "").split(".");
  const value = (BigInt(whole + fraction) << BigInt(bits)) / 10n ** BigInt(fraction.length);
  return text.startsWith("-") ? -value : value;
}

function refine(bits: number): [bigint, bigint] {
  const shift = BigInt(bits);
  const one = 1n << shift;
  let re = parse("-0.7432918908524302", bits);
  let im = parse("0.1312405523087976", bits);
  for (let round = 0; round < 64; round++) {
    let [zr, zi, dr, di] = [0n, 0n, 0n, 0n];
    let [kr, ki, kdr, kdi] = [0n, 0n, 0n, 0n];
    for (let n = 0; n < 26; n++) {
      if (n === 24) [kr, ki, kdr, kdi] = [zr, zi, dr, di];
      [dr, di] = [((2n * (zr * dr - zi * di)) >> shift) + one, (2n * (zr * di + zi * dr)) >> shift];
      [zr, zi] = [((zr * zr - zi * zi) >> shift) + re, ((2n * zr * zi) >> shift) + im];
    }
    const [fr, fi, gr, gi] = [zr - kr, zi - ki, dr - kdr, di - kdi];
    const size = (gr * gr + gi * gi) >> shift;
    if (size === 0n) break;
    const sr = (((fr * gr + fi * gi) >> shift) << shift) / size;
    const si = (((fi * gr - fr * gi) >> shift) << shift) / size;
    re -= sr;
    im -= si;
    if ((sr < 0n ? -sr : sr) + (si < 0n ? -si : si) < 256n) break;
  }
  return [re, im];
}

/** log2 of the widest radius the series covers at each iteration, for a view down to 10^-depth. */
function seriesReach(depth: number): number[] {
  const bits = Math.ceil(depth * Math.log2(10)) + 80;
  const shift = BigInt(bits);
  const [cr, ci] = refine(bits);
  const reach: number[] = [Infinity];
  let [zr, zi] = [0n, 0n];
  let [a, b, c, d] = [ZERO, ZERO, ZERO, ZERO];
  for (let n = 0; n < 1 << 16; n++) {
    const re = Number(zr >> BigInt(bits - 60)) / 2 ** 60;
    const im = Number(zi >> BigInt(bits - 60)) / 2 ** 60;
    if (n > 0) reach.push(Math.min(reach[n - 1], (Math.log2(1e-7) + log2Size(a) - log2Size(d)) / 3));
    if (re * re + im * im > 4 || log2Size(a) > bits - 40) break;
    d = add(times(d, 2 * re, 2 * im), add(times(multiply(a, c), 2, 0), multiply(b, b)));
    c = add(times(c, 2 * re, 2 * im), times(multiply(a, b), 2, 0));
    b = add(times(b, 2 * re, 2 * im), multiply(a, a));
    a = add(times(a, 2 * re, 2 * im), ONE);
    [zr, zi] = [((zr * zr - zi * zi) >> shift) + cr, ((2n * zr * zi) >> shift) + ci];
  }
  return reach;
}

/** The last iteration whose radius covers the view, as step 4's skipFor finds it. */
function skipFor(reach: number[], radius: number): number {
  let [low, high] = [0, Math.max(reach.length - 2, 0)];
  while (low < high) {
    const middle = (low + high + 1) >> 1;
    if (reach[middle] > radius) low = middle;
    else high = middle - 1;
  }
  return low;
}

const SERIES = { left: 120, top: 30, width: 560, height: 280, floor: -1100, image: [1280, 800] };
const reach = seriesReach(1000);
const length = reach.length - 1;
const seriesX = (n: number) => SERIES.left + (n / length) * SERIES.width;
const seriesY = (decades: number) => SERIES.top + (decades / SERIES.floor) * SERIES.height;
const views = [100, 500, 1000].map((depth) => {
  // As in step 4's frame: the radius that covers every pixel of a view 10^-depth tall.
  const zoom = depth * Math.log2(10) + Math.log2(3);
  const pixel = Math.log2(3 / SERIES.image[1]) - zoom;
  const radius = pixel + Math.log2(Math.hypot(SERIES.image[0], SERIES.image[1]) / 2);
  const skip = skipFor(reach, radius);
  return { depth, skip, at: [seriesX(skip), seriesY(radius * Math.log10(2))] as Point };
});

export const series = {
  length,
  frame: { left: SERIES.left, top: SERIES.top, right: SERIES.left + SERIES.width, bottom: SERIES.top + SERIES.height },
  curve: polyline(
    reach
      .map((value, n) => [n, value] as [number, number])
      .filter(([n, value]) => n > 0 && n % 32 === 0 && Number.isFinite(value))
      .map(([n, value]) => [seriesX(n), seriesY(Math.max(value * Math.log10(2), SERIES.floor))] as Point),
  ),
  levels: [0, -250, -500, -750, -1000].map((decades) => ({ decades, y: seriesY(decades) })),
  iterations: [0, 10000, 20000, 30000].map((n) => ({ n, x: seriesX(n) })),
  views,
};

// Workgroups of 8 by 8 invocations over part of an image, inside the compute pass box.
const TILES = { left: 40, top: 92, columns: 5, rows: 3, tile: 32 };
export const passes = {
  invocations: Array.from({ length: TILES.columns * 4 + 1 }, (_, i) => `M${TILES.left + i * 8},${TILES.top}v${TILES.rows * TILES.tile}`)
    .concat(Array.from({ length: TILES.rows * 4 + 1 }, (_, i) => `M${TILES.left},${TILES.top + i * 8}h${TILES.columns * TILES.tile}`))
    .join(""),
  workgroups: Array.from({ length: TILES.columns + 1 }, (_, i) => `M${TILES.left + i * TILES.tile},${TILES.top}v${TILES.rows * TILES.tile}`)
    .concat(Array.from({ length: TILES.rows + 1 }, (_, i) => `M${TILES.left},${TILES.top + i * TILES.tile}h${TILES.columns * TILES.tile}`))
    .join(""),
  // The counts buffer, one cell per pixel, drawn as 5 by 4 cells.
  counts: Array.from({ length: 4 }, (_, i) => `M${288 + (i + 1) * 30},70v110`)
    .concat(Array.from({ length: 3 }, (_, i) => `M288,${70 + (i + 1) * 27.5}h150`))
    .join(""),
};
