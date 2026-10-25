// Geometry for the Drawing with distance figures, computed from the same formulas as the shaders.

type Point = [number, number];

/** The pixel a point `p` lands on in a 720x420 capture, through the tutorial's `center`. */
export function pixel(p: Point): Point {
  return [360 + p[0] * 420, 210 - p[1] * 420];
}

/** Where `p` meets a circle of radius `r` at the origin, on the way from the origin to `p`. */
export function circleEdge(p: Point, r: number): Point {
  const length = Math.hypot(p[0], p[1]);
  return pixel([(p[0] / length) * r, (p[1] / length) * r]);
}

function polyline(points: Point[], closed = false): string {
  const path = points.map(([x, y], i) => `${i ? "L" : "M"} ${x.toFixed(1)} ${y.toFixed(1)}`).join(" ");
  return closed ? `${path} Z` : path;
}

// One distance along a line through two shapes: `a` rises, `b` falls, and they cross at 0.1.
const PLOT = { left: 70, width: 580, zero: 160, unit: 110, k: 0.8 };
const a = (t: number) => 1.8 * t - 0.8;
const b = (t: number) => 1.0 - 1.8 * t;
const smoothUnion = (t: number) => {
  const h = Math.min(Math.max(0.5 + (0.5 * (b(t) - a(t))) / PLOT.k, 0), 1);
  return b(t) * (1 - h) + a(t) * h - PLOT.k * h * (1 - h);
};
const plotX = (t: number) => PLOT.left + PLOT.width * t;
const plotY = (value: number) => PLOT.zero - PLOT.unit * value;
const curve = (f: (t: number) => number, from = 0, to = 1) =>
  polyline(Array.from({ length: 121 }, (_, i) => from + ((to - from) * i) / 120).map((t) => [plotX(t), plotY(f(t))]));
// |a - b| < k holds within 0.8 / 3.6 of the crossing at t = 0.5.
const band = PLOT.k / 3.6;

export const blend = {
  a: curve(a),
  b: curve(b),
  min: curve((t) => Math.min(a(t), b(t))),
  // Outside the band the smooth union equals the minimum, so only its part inside is drawn.
  smooth: curve(smoothUnion, 0.5 - band, 0.5 + band),
  zero: PLOT.zero,
  left: PLOT.left,
  right: PLOT.left + PLOT.width,
  crease: [plotX(0.5), plotY(a(0.5))] as Point,
  dip: [plotX(0.5), plotY(smoothUnion(0.5))] as Point,
  band: [plotX(0.5 - band), plotX(0.5 + band)] as Point,
  aEnd: [plotX(1), plotY(a(1))] as Point,
  bEnd: [plotX(0), plotY(b(0))] as Point,
};

// A five-pointed star, centered on `center`, turned clockwise on screen by `turn` radians.
function star(center: Point, outer: number, inner: number, turn: number): string {
  const points: Point[] = Array.from({ length: 10 }, (_, i) => {
    const radius = i % 2 ? inner : outer;
    const angle = Math.PI / 2 + (i * Math.PI) / 5 - turn;
    return [center[0] + radius * Math.cos(angle), center[1] - radius * Math.sin(angle)];
  });
  return polyline(points, true);
}

// rotate2 turns the point each pixel measures counterclockwise by `angle`, so the star turns clockwise.
const ROTATE = { center: [360, 180] as Point, outer: 115, inner: 46, angle: 0.55, arc: 132 };
const onCircle = (radius: number, angle: number): Point => [
  ROTATE.center[0] + radius * Math.cos(angle),
  ROTATE.center[1] - radius * Math.sin(angle),
];
const tipAngle = Math.PI / 2;

export const rotation = {
  upright: star(ROTATE.center, ROTATE.outer, ROTATE.inner, 0),
  turned: star(ROTATE.center, ROTATE.outer, ROTATE.inner, ROTATE.angle),
  p: onCircle(ROTATE.outer, tipAngle - ROTATE.angle),
  q: onCircle(ROTATE.outer, tipAngle),
  center: ROTATE.center,
  // The arc runs counterclockwise from p to q, just outside the tips.
  arc: (() => {
    const [startX, startY] = onCircle(ROTATE.arc, tipAngle - ROTATE.angle);
    const [endX, endY] = onCircle(ROTATE.arc, tipAngle - 0.06);
    return `M ${startX.toFixed(1)} ${startY.toFixed(1)} A ${ROTATE.arc} ${ROTATE.arc} 0 0 0 ${endX.toFixed(1)} ${endY.toFixed(1)}`;
  })(),
  arrowFrom: onCircle(ROTATE.arc, tipAngle - 0.08),
  arrowTo: onCircle(ROTATE.arc, tipAngle + 0.02),
  arcLabel: onCircle(ROTATE.arc + 22, tipAngle - ROTATE.angle / 2),
};
