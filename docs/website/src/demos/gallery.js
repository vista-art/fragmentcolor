// Live artwork gallery rendered by FragmentColor. Each artwork draws into an
// offscreen texture target; a compositor pass samples it back, crossfades
// between artworks, and adds grain, chromatic aberration, and a vignette.

import { PRELUDE } from "./prelude.js";
import { ARTWORKS } from "./artworks/index.js";
import { COMPOSITOR_SLUGS, COMPOSITOR_SOURCE } from "./compositor.js";

export { ARTWORKS };

let runtime;

export function supportsWebGPU() {
  return typeof navigator !== "undefined" && !!navigator.gpu;
}

// Boots the WASM module once per page and resolves catalog slugs against
// the site's own /shaders/ tree, so dev works offline and prod serves the
// same files fragmentcolor.org publishes.
async function loadRuntime() {
  const fc = await import("fragmentcolor");
  const wasmUrl = (await import("fragmentcolor/fragmentcolor_bg.wasm?url")).default;
  await fc.default({ module_or_path: wasmUrl });
  fc.Shader.setRegistry("/shaders/");
  return fc;
}

function clamp(v, lo, hi) {
  return Math.min(Math.max(v, lo), hi);
}

export class Gallery {
  /**
   * @param {HTMLCanvasElement} canvas
   * @param {object} [options]
   * @param {number} [options.start] index of the first artwork
   * @param {number} [options.dprCap] device pixel ratio ceiling
   * @param {number} [options.transition] crossfade length in seconds
   * @param {number} [options.autoAdvance] seconds of quiet before moving on; 0 disables
   * @param {boolean} [options.wheelZoom] scroll wheel zooms instead of scrolling the page
   * @param {boolean} [options.touchDrag] touch drags steer the artwork instead of scrolling
   * @param {(index: number) => void} [options.onChange] called when a new artwork starts fading in
   */
  constructor(canvas, options = {}) {
    this.canvas = canvas;
    this.options = {
      start: 0,
      dprCap: 1.6,
      transition: 1.7,
      autoAdvance: 0,
      wheelZoom: true,
      touchDrag: true,
      onChange: null,
      ...options,
    };
    this.input = {
      mouse: [0.5, 0.5],
      smooth: [0.5, 0.5],
      drag: [0.55, 0.32],
      dragging: false,
      lastPointer: null,
      zoom: 1,
      zoomTarget: 1,
      press: 0,
      pressTarget: 0,
      pulse: 0,
    };
    this.show = { current: this.options.start, slot: 0, transition: null };
    this.pointers = new Map();
    this.lastPinch = null;
    this.lastTime = null;
    this.lastPoke = 0;
    this.lastAdvance = 0;
    this.raf = 0;
    this.paused = false;
    this.destroyed = false;
    this.cleanups = [];
  }

  get index() {
    return this.show.transition ? this.show.transition.to : this.show.current;
  }

  pixelSize() {
    const dpr = Math.min(window.devicePixelRatio || 1, this.options.dprCap);
    return [
      Math.max(1, Math.round(this.canvas.clientWidth * dpr)),
      Math.max(1, Math.round(this.canvas.clientHeight * dpr)),
    ];
  }

  async start() {
    if (!supportsWebGPU()) throw new Error("WebGPU is not available");
    // A failed boot clears the cache so a later start() can retry.
    runtime ??= loadRuntime().catch((err) => {
      runtime = undefined;
      throw err;
    });
    const { Renderer, Shader, Pass } = await runtime;
    if (this.destroyed) return;

    const canvas = this.canvas;
    const [w, h] = this.pixelSize();
    canvas.width = w;
    canvas.height = h;

    this.renderer = new Renderer();
    this.target = await this.renderer.createTarget(canvas);
    this.texTargets = [
      await this.renderer.createTextureTarget([w, h]),
      await this.renderer.createTextureTarget([w, h]),
    ];

    this.shaders = await Promise.all(
      ARTWORKS.map((a) => Shader.fetch([...a.slugs, PRELUDE + a.source])),
    );
    this.compositor = await Shader.fetch([...COMPOSITOR_SLUGS, COMPOSITOR_SOURCE]);
    if (this.destroyed) return;

    this.passes = ARTWORKS.map((a, i) => {
      const pass = new Pass(a.id);
      pass.addShader(this.shaders[i]);
      return pass;
    });
    this.compositorPass = new Pass("composite");
    this.compositorPass.addShader(this.compositor);

    const { show, compositor, texTargets, passes } = this;
    compositor.set("tex_from", texTargets[show.slot].texture());
    compositor.set("tex_to", texTargets[show.slot].texture());
    compositor.set("u.aberration", 0.0016);
    compositor.set("u.grain", 0.035);
    compositor.set("u.fade", 0);
    compositor.set("u.resolution", [w, h]);

    // Render every pass once so all pipelines compile before the first frame.
    for (let i = 0; i < passes.length; i++) {
      passes[i].setTarget(texTargets[1 - show.slot]);
      this.setGlobals(this.shaders[i], 0, w, h);
    }
    passes[show.current].setTarget(texTargets[show.slot]);
    this.renderer.render([...passes, this.compositorPass], this.target);

    this.wireInput();
    this.poke();
    this.lastAdvance = this.lastPoke;
    this.raf = requestAnimationFrame((t) => this.frame(t));
  }

  setGlobals(shader, time, w, h) {
    const input = this.input;
    shader.set("u.resolution", [w, h]);
    shader.set("u.mouse", input.smooth);
    shader.set("u.drag", input.drag);
    shader.set("u.time", time);
    shader.set("u.zoom", input.zoom);
    shader.set("u.press", input.press);
    shader.set("u.pulse", input.pulse);
  }

  frame(now) {
    if (this.destroyed) return;
    this.raf = requestAnimationFrame((t) => this.frame(t));
    if (this.paused) {
      this.lastTime = null;
      return;
    }

    const time = now / 1000;
    const dt = Math.min(this.lastTime === null ? 0.016 : time - this.lastTime, 0.1);
    this.lastTime = time;

    this.resize();

    const input = this.input;
    const k = 1 - Math.exp(-dt * 7);
    input.smooth[0] += (input.mouse[0] - input.smooth[0]) * k;
    input.smooth[1] += (input.mouse[1] - input.smooth[1]) * k;
    input.zoom += (input.zoomTarget - input.zoom) * (1 - Math.exp(-dt * 8));
    input.press += (input.pressTarget - input.press) * (1 - Math.exp(-dt * 6));
    input.pulse *= Math.exp(-dt * 2.1);

    const { canvas, show, shaders, passes, compositor } = this;
    const [w, h] = [canvas.width, canvas.height];
    let fade = 0;
    if (show.transition) {
      fade = Math.min((time - show.transition.start) / this.options.transition, 1);
      if (fade >= 1) {
        this.finishTransition();
        fade = 0;
      }
    }

    const active = show.transition
      ? [show.transition.from, show.transition.to]
      : [show.current];
    for (const i of active) this.setGlobals(shaders[i], time, w, h);

    compositor.set("u.resolution", [w, h]);
    compositor.set("u.time", time);
    compositor.set("u.fade", fade);

    const list = show.transition
      ? [passes[show.transition.from], passes[show.transition.to], this.compositorPass]
      : [passes[show.current], this.compositorPass];
    try {
      this.renderer.render(list, this.target);
    } catch (err) {
      console.error("render failed:", err);
    }

    const quiet = this.options.autoAdvance;
    if (
      quiet > 0 &&
      time - this.lastPoke > quiet &&
      time - this.lastAdvance > quiet &&
      !show.transition
    ) {
      this.lastAdvance = time;
      this.goTo(show.current + 1);
    }
  }

  resize() {
    const [w, h] = this.pixelSize();
    const canvas = this.canvas;
    if (w === canvas.width && h === canvas.height) return;
    canvas.width = w;
    canvas.height = h;
    this.target.resize([w, h]);
    for (const t of this.texTargets) t.resize([w, h]);
  }

  goTo(index) {
    const show = this.show;
    const next = (index + ARTWORKS.length) % ARTWORKS.length;
    if (show.transition) this.finishTransition();
    if (next === show.current) return;

    const freeSlot = 1 - show.slot;
    this.passes[next].setTarget(this.texTargets[freeSlot]);
    this.compositor.set("tex_from", this.texTargets[show.slot].texture());
    this.compositor.set("tex_to", this.texTargets[freeSlot].texture());
    show.transition = { from: show.current, to: next, start: performance.now() / 1000 };
    this.lastAdvance = show.transition.start;
    this.options.onChange?.(next);
  }

  next() {
    this.poke();
    this.goTo(this.index + 1);
  }

  prev() {
    this.poke();
    this.goTo(this.index - 1);
  }

  finishTransition() {
    const show = this.show;
    const t = show.transition;
    if (!t) return;
    show.current = t.to;
    show.slot = 1 - show.slot;
    show.transition = null;
    this.compositor.set("tex_from", this.texTargets[show.slot].texture());
    this.compositor.set("u.fade", 0);
  }

  poke() {
    this.lastPoke = performance.now() / 1000;
  }

  pause() {
    this.paused = true;
  }

  resume() {
    if (!this.paused) return;
    this.paused = false;
    this.poke();
    this.lastAdvance = this.lastPoke;
  }

  wireInput() {
    const { canvas, input, pointers } = this;
    const on = (target, type, fn, opts) => {
      target.addEventListener(type, fn, opts);
      this.cleanups.push(() => target.removeEventListener(type, fn, opts));
    };
    const coarse = (e) => e.pointerType === "touch";

    on(canvas, "pointermove", (e) => {
      if (pointers.has(e.pointerId)) pointers.set(e.pointerId, [e.clientX, e.clientY]);

      if (pointers.size === 2) {
        const [a, b] = [...pointers.values()];
        const d = Math.hypot(a[0] - b[0], a[1] - b[1]);
        if (this.lastPinch)
          input.zoomTarget = clamp((input.zoomTarget * d) / this.lastPinch, 0.5, 2.4);
        this.lastPinch = d;
        this.poke();
        return;
      }

      if (!e.isPrimary) return;
      const rect = canvas.getBoundingClientRect();
      input.mouse[0] = clamp((e.clientX - rect.left) / rect.width, 0, 1);
      input.mouse[1] = clamp(1 - (e.clientY - rect.top) / rect.height, 0, 1);
      if (input.dragging && input.lastPointer) {
        input.drag[0] += (e.clientX - input.lastPointer[0]) * 0.006;
        input.drag[1] = clamp(
          input.drag[1] + (e.clientY - input.lastPointer[1]) * 0.006,
          -1.2,
          1.2,
        );
      }
      input.lastPointer = [e.clientX, e.clientY];
      this.poke();
    });

    on(canvas, "pointerdown", (e) => {
      if (e.button !== 0) return;
      if (coarse(e) && !this.options.touchDrag) return;
      canvas.setPointerCapture(e.pointerId);
      pointers.set(e.pointerId, [e.clientX, e.clientY]);
      this.lastPinch = null;
      if (!e.isPrimary) return;
      input.dragging = true;
      input.pressTarget = 1;
      input.pulse = Math.min(input.pulse + 1, 1.4);
      input.lastPointer = [e.clientX, e.clientY];
      this.poke();
    });

    const release = (e) => {
      pointers.delete(e.pointerId);
      this.lastPinch = null;
      if (!e.isPrimary) return;
      input.dragging = false;
      input.pressTarget = 0;
    };
    on(canvas, "pointerup", release);
    on(canvas, "pointercancel", release);
    on(canvas, "lostpointercapture", release);
    on(window, "blur", () => {
      pointers.clear();
      this.lastPinch = null;
      input.dragging = false;
      input.pressTarget = 0;
    });

    if (this.options.wheelZoom) {
      on(
        canvas,
        "wheel",
        (e) => {
          e.preventDefault();
          input.zoomTarget = clamp(input.zoomTarget * Math.exp(-e.deltaY * 0.0012), 0.5, 2.4);
          this.poke();
        },
        { passive: false },
      );
    }

    on(document, "visibilitychange", () => {
      if (document.hidden) this.pause();
      else this.resume();
    });

    if (typeof IntersectionObserver !== "undefined") {
      const io = new IntersectionObserver(
        ([entry]) => {
          if (entry.isIntersecting) this.resume();
          else this.pause();
        },
        { threshold: 0.05 },
      );
      io.observe(canvas);
      this.cleanups.push(() => io.disconnect());
    }
  }

  destroy() {
    this.destroyed = true;
    cancelAnimationFrame(this.raf);
    for (const fn of this.cleanups) fn();
    this.cleanups = [];
  }
}
