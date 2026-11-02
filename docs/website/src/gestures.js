// The wheel, a drag and a two-finger pinch over a canvas, gathered between frames
// for a demo's frame(): zoom in doublings (positive zooms in) and pan in canvas
// pixels, y down. The playground and the tutorial <Demo> both pass it to frame()
// as its seventh argument.
//
// With `scroll`, the page keeps its own scrolling, as an article needs. Only a pinch
// zooms: a trackpad pinch, which arrives as a wheel event with ctrlKey, or the wheel
// with ctrl or cmd held. One finger on a touch screen still scrolls the page, and
// two fingers pinch and pan. A mouse drag pans either way.
export class Gestures {
  #canvas;
  #scroll;
  #enabled = false;
  #touches = new Map();
  #zoom = 0;
  #pan = [0, 0];
  #pointer = null;
  #cleanups = [];

  constructor(canvas, { scroll = false } = {}) {
    this.#canvas = canvas;
    this.#scroll = scroll;
    this.#listen("wheel", (event) => this.#wheel(event), { passive: false });
    this.#listen("pointerdown", (event) => this.#press(event));
    this.#listen("pointermove", (event) => this.#move(event));
    this.#listen("pointerup", (event) => this.#touches.delete(event.pointerId));
    this.#listen("pointercancel", (event) => this.#touches.delete(event.pointerId));
    // Two fingers belong to the canvas, so the page neither scrolls nor zooms under them.
    const hold = (event) => this.#enabled && event.touches.length > 1 && event.preventDefault();
    this.#listen("touchstart", hold, { passive: false });
    this.#listen("touchmove", hold, { passive: false });
  }

  get enabled() {
    return this.#enabled;
  }

  /** Off, the canvas leaves the wheel and every touch to the page. */
  set enabled(on) {
    this.#enabled = on;
    this.#touches.clear();
    this.#canvas.style.touchAction = on ? (this.#scroll ? "pan-x pan-y" : "none") : "";
  }

  /** Zoom and pan since the last call, and the pointer a pinch moved, if one did. */
  take() {
    const taken = { zoom: this.#zoom, pan: this.#pan, pointer: this.#pointer };
    this.#zoom = 0;
    this.#pan = [0, 0];
    this.#pointer = null;
    return taken;
  }

  detach() {
    for (const cleanup of this.#cleanups) cleanup();
    this.#cleanups = [];
  }

  #listen(type, handler, options) {
    this.#canvas.addEventListener(type, handler, options);
    this.#cleanups.push(() => this.#canvas.removeEventListener(type, handler, options));
  }

  #wheel(event) {
    if (!this.#enabled) return;
    const pinch = event.ctrlKey || event.metaKey;
    if (this.#scroll && !pinch) return;
    event.preventDefault();
    const lines = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? this.#canvas.clientHeight : 1;
    // A pinch sends small deltas, so it counts four times as much as a wheel notch.
    this.#zoom -= (event.deltaY * lines) / (pinch ? 100 : 400);
  }

  #press(event) {
    if (!this.#enabled || (event.pointerType === "mouse" && event.button !== 0)) return;
    if (event.pointerType !== "touch") this.#canvas.setPointerCapture(event.pointerId);
    this.#touches.set(event.pointerId, [event.clientX, event.clientY]);
  }

  #move(event) {
    const last = this.#touches.get(event.pointerId);
    if (!last) return;
    const before = this.#touches.size === 2 ? { spread: this.#spread(), middle: this.#middle() } : null;
    this.#touches.set(event.pointerId, [event.clientX, event.clientY]);
    const ratio = this.#canvas.width / Math.max(this.#canvas.getBoundingClientRect().width, 1);
    if (before) {
      // A pinch zooms around the point between the two fingers.
      const now = this.#middle();
      const rect = this.#canvas.getBoundingClientRect();
      this.#pointer = [(now[0] - rect.left) * ratio, (now[1] - rect.top) * ratio];
      this.#zoom += Math.log2(Math.max(this.#spread(), 1) / Math.max(before.spread, 1));
      this.#pan[0] += (now[0] - before.middle[0]) * ratio;
      this.#pan[1] += (now[1] - before.middle[1]) * ratio;
    } else if (this.#touches.size === 1 && !(this.#scroll && event.pointerType === "touch")) {
      // One finger pans only where the page doesn't need it to scroll.
      this.#pan[0] += (event.clientX - last[0]) * ratio;
      this.#pan[1] += (event.clientY - last[1]) * ratio;
    }
  }

  #spread() {
    const [a, b] = [...this.#touches.values()];
    return Math.hypot(a[0] - b[0], a[1] - b[1]);
  }

  #middle() {
    const points = [...this.#touches.values()];
    return points.reduce(([x, y], [px, py]) => [x + px / points.length, y + py / points.length], [0, 0]);
  }
}
