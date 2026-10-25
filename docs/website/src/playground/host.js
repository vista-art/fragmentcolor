// Developer playground host.
//
// Each demo is a JS program plus WGSL files. The host rebuilds the program as
// the code changes, keeps the last good build drawing when a build fails,
// generates uniform controls for the keys setup sets, and keeps the Rust,
// Python, Swift, and Kotlin tabs in step with what is running.

import { loadProgram } from "./program.js";
import { describeControls, renderControls, sliderRange } from "./controls.js";
import { LANGUAGES, generateCode, parseLibrary, replaceLibrary } from "./codegen.js";
import { HARNESS } from "./harness.js";
import { LibraryPicker } from "./picker.js";
import { loadFragmentColor, supports } from "../runtime.js";

const BUILD_DELAY = 250;
const SCRUB_DELAY = 40;
const DPR_CAP = 2;

let runtime;
let instances = 0;
let recorder = null;
// Every key the builds set on each shader, read when a later build keeps that shader.
const written = new WeakMap();
let gpuCheck;

async function loadRuntime() {
  const fc = await loadFragmentColor();
  globalThis.__fragmentcolor = fc;

  // Records which keys setup and frame set, which decides the controls.
  const set = fc.Shader.prototype.set;
  fc.Shader.prototype.set = function (key, value) {
    recorder?.(this, key);
    return set.call(this, key, value);
  };
  return fc;
}

// The engine picks WebGPU when an adapter exists and WebGL2 otherwise.
export function detectBackend() {
  gpuCheck ??= (async () => {
    try {
      return navigator.gpu && (await navigator.gpu.requestAdapter()) ? "WebGPU" : "WebGL2";
    } catch {
      return "WebGL2";
    }
  })();
  return gpuCheck;
}

// FragmentColor 0.12.5 logs shader errors and returns a default shader; later
// releases throw them and log only the GPU errors no error scope caught.
// Capturing console.error covers both, and from 0.12.6 it keeps only GPU
// errors, so a stray error elsewhere on the page can't fail a build.
const isBuildError = supports("0.12.6") ? (text) => text.includes("==== GPU error") : () => true;

async function capturing(run) {
  const errors = [];
  const original = console.error;
  console.error = (...args) => {
    const text = args.map((a) => (a instanceof Error ? a.message : String(a))).join(" ");
    if (isBuildError(text)) errors.push(text);
    original.apply(console, args);
  };
  try {
    const result = await run();
    if (errors.length) throw new Error(errors.join("\n"));
    return result;
  } catch (err) {
    // A logged shader error is the cause of whatever setup tripped on next.
    if (errors.length) throw new Error(errors.join("\n"));
    throw err;
  } finally {
    console.error = original;
  }
}

function shadersIn(state, Shader) {
  const values = state instanceof Shader ? [state] : Object.values(state ?? {}).flat();
  return values.filter((v) => v instanceof Shader);
}

// A shader that main.js keeps from an earlier build still holds what that build set.
// Keys this setup leaves alone go back to zero, which is what a new shader holds.
function clearStale(shaders, setupKeys) {
  for (const shader of shaders) {
    const keys = written.get(shader) ?? [];
    for (const key of keys) {
      if (setupKeys.get(shader)?.has(key)) continue;
      keys.delete(key);
      try {
        const zero = zeroLike(shader.get(key));
        if (zero !== null) shader.set(key, zero);
      } catch {
        // the key left the shader
      }
    }
  }
}

// Zeros in the shape and number type of `value`; null for anything but numbers.
function zeroLike(value) {
  if (typeof value === "number") return 0;
  if (ArrayBuffer.isView(value)) return new value.constructor(value.length);
  if (!Array.isArray(value)) return null;
  const zeros = value.map(zeroLike);
  return zeros.includes(null) ? null : zeros;
}

function plain(value) {
  if (typeof value === "number") return Math.round(value * 1e4) / 1e4;
  return Array.from(value, (n) => Math.round(n * 1e4) / 1e4);
}

// `.set("key", value)` calls in main.js, with the span of each literal value.
// The generated controls and the inline sliders keep these literals current.
const SET_CALL = /\.set\(\s*(["'])([^"'\n]+)\1\s*,\s*/g;
const SET_VALUE = /^(-?(?:\d[\d_]*\.?\d*|\.\d+)(?:[eE][-+]?\d+)?|\[[^\]\n]*\])\s*\)/;

function findSetValues(text) {
  const found = [];
  for (const match of text.matchAll(SET_CALL)) {
    const from = match.index + match[0].length;
    const literal = text.slice(from, from + 200).match(SET_VALUE);
    if (literal) found.push({ key: match[2], from, to: from + literal[1].length, text: literal[1] });
  }
  return found;
}

function parseLiteral(text) {
  const numbers = text.replace(/[[\]\s_]/g, "").split(",").map(Number);
  if (numbers.some((n) => !Number.isFinite(n))) return null;
  return text.trim().startsWith("[") ? numbers : numbers[0];
}

function formatLiteral(value) {
  const one = (n) => String(Math.round(n * 1e4) / 1e4);
  return typeof value === "number" ? one(value) : `[${Array.from(value, one).join(", ")}]`;
}

function message(err) {
  return String(err?.message ?? err).trim();
}

export class Playground {
  /**
   * @param {HTMLElement} root the <fc-playground> element
   * @param {object[]} demos demo modules, in picker order
   */
  constructor(root, demos) {
    this.root = root;
    this.demos = demos;
    this.canvas = root.querySelector("canvas");
    this.ui = {
      status: root.querySelector(".fcp-status"),
      library: root.querySelector(".fcp-library-list"),
      add: root.querySelector(".fcp-add"),
      tabs: root.querySelector(".fcp-tabs"),
      editor: root.querySelector(".fcp-editor"),
      error: root.querySelector(".fcp-error"),
      controls: root.querySelector(".fcp-controls"),
      poster: root.querySelector(".fcp-poster"),
      reset: root.querySelector(".fcp-reset"),
      copy: root.querySelector(".fcp-copy"),
      fullscreen: root.querySelector(".fcp-fullscreen"),
    };
    this.edits = new Map();
    this.overrides = new Map();
    this.pointer = null;
    // Wheel, drag and pinch since the last frame, for demos with `gestures: true`.
    this.gesture = { zoom: 0, pan: [0, 0] };
    this.touches = new Map();
    this.program = null;
    this.pending = null;
    this.buildToken = 0;
    this.attempts = 0;
    this.timer = 0;
    this.raf = 0;
    this.offscreen = false;
    this.destroyed = false;
    this.cleanups = [];
    this.uid = `fcp-${++instances}`;
  }

  on(target, type, fn, options) {
    target.addEventListener(type, fn, options);
    this.cleanups.push(() => target.removeEventListener(type, fn, options));
  }

  async start(id) {
    const [fc, backend, editor] = await Promise.all([runtime ??= loadRuntime(), detectBackend(), loadEditor()]);
    if (this.destroyed) return;
    this.fc = fc;
    this.backend = backend;
    this.cm = editor;
    this.resize();
    this.view = new editor.EditorView({ parent: this.ui.editor });
    this.ui.editor.id = `${this.uid}-panel`;
    this.picker = new LibraryPicker(this.root.querySelector(".fcp-code"), {
      trigger: this.ui.add,
      onToggle: (slug, selected) => this.toggleSlug(slug, selected),
    });
    this.wire();
    this.select(this.pending ?? id);
    this.raf = requestAnimationFrame((t) => this.tick(t));
  }

  // Swaps in another demo. Edits made to the current one stay in memory until reset.
  // A demo picked while the runtime loads opens when start is done.
  select(id) {
    const demo = this.demos.find((d) => d.id === id) ?? this.demos[0];
    if (!this.view) {
      this.pending = demo.id;
      return;
    }
    clearTimeout(this.timer);
    this.buildToken++;
    this.attempts = 0;
    if (this.demo) this.edits.set(this.demo.id, this.snapshot());
    this.demo = demo;
    this.overrides.clear();
    this.program = null;
    this.controls = [];
    renderControls(this.ui.controls, [], () => {});
    this.root.dataset.active = demo.id;
    // A demo that takes gestures keeps touches on the canvas from scrolling the page.
    this.canvas.style.touchAction = demo.gestures ? "none" : "";
    this.loadTabs(this.edits.get(demo.id) ?? this.original(demo));
    this.showError(null);

    const blocked = demo.requires === "webgpu" && this.backend !== "WebGPU";
    this.blocked = blocked;
    this.ui.poster.hidden = !blocked;
    if (blocked) {
      const img = this.ui.poster.querySelector("img");
      img.hidden = !demo.poster;
      if (demo.poster) img.src = demo.poster;
      this.setStatus(`${this.backend} · this demo needs WebGPU`);
      return;
    }
    this.setStatus(`${this.backend} · building`);
    this.build();
  }

  original(demo) {
    const js = demo.js ?? generateCode(this.model(demo, demo.library, Object.entries(demo.values ?? {}))).js;
    return { files: { ...demo.files }, js };
  }

  snapshot() {
    const files = {};
    for (const tab of this.tabs ?? []) {
      if (tab.kind === "wgsl") files[tab.id] = this.text(tab.id);
    }
    return { files, js: this.text("js") };
  }

  model(demo, library, values) {
    return {
      kind: demo.kind,
      library,
      files: Object.keys(demo.files),
      values,
      frame: demo.frame ?? {},
      demo,
    };
  }

  // Editor tabs: the WGSL files and main.js are editable. index.js, the page code
  // that runs main.js, is read-only, and the other languages are generated.
  loadTabs({ files, js }) {
    // A demo opens on the last language tab the reader picked, main.js at first, never on a shader file.
    const language = this.language ?? "js";
    const tabs = [];
    for (const [name, text] of Object.entries(files)) {
      tabs.push({ id: name, label: name, kind: "wgsl", state: this.editorState(text, "wgsl", true) });
    }
    tabs.push({ id: "js", label: "main.js", kind: "js", state: this.editorState(js, "js", true) });
    // Reading index.js counts as reading JavaScript, so the next demo opens on main.js.
    const harness = this.editorState(HARNESS, "js", false);
    tabs.push({ id: "index.js", label: "index.js", kind: "js", language: "js", readonly: true, state: harness });
    for (const lang of LANGUAGES.filter((l) => l.id !== "js")) {
      const { id, label, file } = lang;
      tabs.push({ id, label, file, kind: id, generated: true, readonly: true, state: null });
    }
    this.tabs = tabs;
    this.codeStale = true;
    this.ui.tabs.replaceChildren(
      ...tabs.map((tab) => {
        const button = document.createElement("button");
        button.type = "button";
        button.role = "tab";
        button.id = `${this.uid}-tab-${tab.id.replace(/\W/g, "-")}`;
        button.setAttribute("aria-controls", this.ui.editor.id);
        button.dataset.tab = tab.id;
        button.textContent = tab.label;
        if (tab.generated) {
          button.classList.add("generated");
          button.title = `${tab.file}, generated from the running demo. Edit the shader or main.js to change it.`;
        } else if (tab.readonly) {
          button.classList.add("readonly");
          button.title = `${tab.label} calls setup and frame from main.js the way this page does. It is read-only.`;
        }
        button.addEventListener("click", () => this.showTab(tab.id));
        return button;
      }),
    );
    this.activeTab = null;
    this.showTab(language);
    this.renderLibrary();
  }

  editorState(text, kind, editable) {
    const { EditorState, EditorView, extensions, languages } = this.cm;
    return EditorState.create({
      doc: text,
      extensions: [
        extensions,
        this.cm.theme.of(),
        languages(kind),
        EditorState.readOnly.of(!editable),
        EditorView.editable.of(editable),
        editable ? this.cm.inline(kind, this) : [],
        EditorView.updateListener.of((update) => {
          if (!update.docChanged || !editable) return;
          const tab = this.tabs.find((t) => t.id === this.activeTab);
          if (tab) tab.state = update.state;
          const changes = update.transactions.filter((t) => t.docChanged);
          if (changes.every((t) => t.annotation(this.cm.synced))) return;
          if (changes.every((t) => t.annotation(this.cm.scrubbed))) {
            if (kind === "js" && this.applyScrub(update)) return;
            this.scheduleBuild(SCRUB_DELAY);
            return;
          }
          this.scheduleBuild();
        }),
      ],
    });
  }

  // Each tab keeps its own editor state, so a site theme switch reaches every one.
  applyTheme() {
    for (const tab of this.tabs ?? []) {
      if (tab.id === this.activeTab) {
        this.view.dispatch({ effects: this.cm.theme.reconfigure() });
        tab.state = this.view.state;
      } else if (tab.state) {
        tab.state = tab.state.update({ effects: this.cm.theme.reconfigure() }).state;
      }
    }
  }

  // An inline slider on a `.set` value applies it at once, with no rebuild.
  applyScrub(update) {
    const values = findSetValues(update.state.doc.toString());
    const touched = [];
    let inside = true;
    update.changes.iterChangedRanges((fromA, toA, from, to) => {
      const value = values.find((v) => v.from <= from && to <= v.to);
      if (value) touched.push(value);
      else inside = false;
    });
    if (!inside || !touched.length || !this.program) return false;
    try {
      for (const { key, text } of touched) {
        const parsed = parseLiteral(text);
        const shader = this.program.shaders.find((sh) => sh.listKeys().includes(key));
        if (parsed === null || !shader) return false;
        shader.set(key, parsed);
        this.controls.find((c) => c.key === key && c.shader === shader)?.update?.(parsed);
      }
    } catch {
      return false;
    }
    this.invalidateCode();
    return true;
  }

  // Writes a control's value into its `.set` literal in main.js, if there is one.
  writeValue(key, value) {
    const tab = this.tabs.find((t) => t.id === "js");
    if (!tab) return false;
    const state = tab.id === this.activeTab ? this.view.state : tab.state;
    const target = findSetValues(state.doc.toString()).find((v) => v.key === key);
    if (!target) return false;
    const spec = {
      changes: { from: target.from, to: target.to, insert: formatLiteral(value) },
      annotations: this.cm.synced.of(true),
    };
    if (tab.id === this.activeTab) {
      this.view.dispatch(spec);
      tab.state = this.view.state;
    } else {
      tab.state = tab.state.update(spec).state;
    }
    return true;
  }

  // Ranges and color hints for the inline controls on `.set` values.
  setValueAt(state, from, to) {
    const value = findSetValues(state.doc.toString()).find((v) => v.from <= from && to <= v.to);
    if (!value) return null;
    const control = this.controls.find((c) => c.key === value.key);
    return { key: value.key, control, from: value.from, text: value.text };
  }

  // An inline slider on a `.set` value takes the range its control shows. A value
  // typed past that range grows it, and the control resizes along with the slider.
  rangeAt(state, from, to) {
    const found = this.setValueAt(state, from, to);
    const control = found?.control;
    if (!control || control.color) return null;
    const index = found.text.slice(0, from - found.from).split(",").length - 1;
    const value = Number(state.sliceDoc(from, to).replace(/_/g, ""));
    const shown = control.ranges[index];
    if (!shown || !Number.isFinite(value)) return null;
    if (value >= shown.min && value <= shown.max) return shown;
    const range = sliderRange(control, value);
    control.setRange?.(index, range);
    return range;
  }

  showTab(id) {
    const current = this.tabs.find((t) => t.id === this.activeTab);
    if (current) {
      current.scroll = this.view.scrollSnapshot();
      if (!current.generated) current.state = this.view.state;
    }
    const tab = this.tabs.find((t) => t.id === id) ?? this.tabs[0];
    this.activeTab = tab.id;
    if (tab.kind !== "wgsl") this.language = tab.language ?? tab.id;
    if (tab.generated && (this.codeStale || !tab.state)) this.refreshCode();
    this.view.setState(tab.state);
    try {
      if (tab.scroll) this.view.dispatch({ effects: tab.scroll });
      else this.view.scrollDOM.scrollTop = 0;
    } catch {
      // a regenerated tab can be shorter than the snapshot
      this.view.scrollDOM.scrollTop = 0;
    }
    for (const button of this.ui.tabs.querySelectorAll("button")) {
      const active = button.dataset.tab === tab.id;
      button.setAttribute("aria-selected", String(active));
      button.tabIndex = active ? 0 : -1;
      if (active) this.ui.editor.setAttribute("aria-labelledby", button.id);
    }
    this.root.dataset.tab = tab.readonly ? "readonly" : "source";
    this.showInstall(tab.kind);
  }

  // The landing install line follows the language tab; WGSL tabs keep the last one.
  showInstall(lang) {
    const lines = this.root.querySelectorAll(".fcp-install-line");
    if (![...lines].some((line) => line.dataset.lang === lang)) return;
    for (const line of lines) line.hidden = line.dataset.lang !== lang;
    for (const button of this.root.querySelectorAll(".fcp-install-langs button")) {
      const active = button.dataset.lang === lang;
      button.setAttribute("aria-selected", String(active));
      button.tabIndex = active ? 0 : -1;
    }
    const select = this.root.querySelector(".fcp-install-select");
    if (select) select.value = lang;
  }

  text(id) {
    const tab = this.tabs.find((t) => t.id === id);
    if (!tab) return "";
    return (tab.id === this.activeTab ? this.view.state : tab.state).doc.toString();
  }

  setText(id, text) {
    const tab = this.tabs.find((t) => t.id === id);
    if (!tab) return;
    if (tab.id === this.activeTab) {
      this.view.dispatch({ changes: { from: 0, to: this.view.state.doc.length, insert: text } });
      tab.state = this.view.state;
    } else {
      tab.state = tab.state.update({ changes: { from: 0, to: tab.state.doc.length, insert: text } }).state;
    }
  }

  scheduleBuild(delay = BUILD_DELAY) {
    clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.renderLibrary();
      if (this.blocked) this.invalidateCode();
      else this.build();
    }, delay);
  }

  // Builds run one at a time, since a build swaps console.error and the set
  // recorder. Edits made during a build queue one more build of the latest code.
  build() {
    if (this.building) {
      this.queued = true;
      return this.building;
    }
    this.building = this.buildOnce().finally(() => {
      this.building = null;
      if (!this.queued || this.destroyed) return;
      this.queued = false;
      if (!this.blocked) this.build();
    });
    return this.building;
  }

  // Builds the current code. On failure the previous program keeps drawing.
  async buildOnce() {
    const token = ++this.buildToken;
    const demo = this.demo;
    const files = this.snapshot().files;
    const started = performance.now();
    const setupKeys = new Map();
    const frameKeys = new Map();
    const note = (map) => (shader, key) => {
      for (const keys of [map, written]) {
        if (!keys.has(shader)) keys.set(shader, new Set());
        keys.get(shader).add(key);
      }
    };

    let module;
    let state;
    try {
      await this.ensureTarget();
      module = await loadProgram(this.text("js"), files);
      recorder = note(setupKeys);
      state = await capturing(() => module.setup(this.renderer, this.target));
    } catch (err) {
      this.retryOrFail(err, token, demo);
      return;
    } finally {
      recorder = null;
    }
    if (token !== this.buildToken || this.destroyed || demo !== this.demo) return;

    const shaders = shadersIn(state, this.fc.Shader);
    clearStale(shaders, setupKeys);
    // A control's value outlives rebuilds for as long as setup still sets its key.
    for (const [key, value] of this.overrides) {
      const owners = shaders.filter((shader) => setupKeys.get(shader)?.has(key));
      if (!owners.length) this.overrides.delete(key);
      for (const shader of owners) {
        try {
          shader.set(key, value);
        } catch {
          this.overrides.delete(key);
        }
      }
    }

    const previous = this.program && !this.program.failed ? { ...this.program, previous: null } : null;
    const program = { module, state, shaders, setupKeys, previous, epoch: previous?.epoch ?? performance.now() };
    try {
      recorder = note(frameKeys);
      await capturing(() => this.draw(program, performance.now()));
    } catch (err) {
      this.retryOrFail(err, token, demo);
      return;
    } finally {
      recorder = null;
    }

    this.program = program;
    this.attempts = 0;
    this.canvas.style.visibility = "";
    this.showError(null);
    this.setStatus(`${this.backend} · built in ${Math.round(performance.now() - started)} ms`);
    this.controls = describeControls(
      shaders.map((shader) => ({
        shader,
        keys: setupKeys.get(shader) ?? new Set(),
        hidden: frameKeys.get(shader) ?? new Set(),
      })),
      Object.values(files),
    );
    renderControls(this.ui.controls, this.controls, (control, value) => this.adjust(control, value));
    // The inline sliders on main.js read their ranges from the new controls.
    if (this.activeTab === "js") this.view.dispatch({ effects: this.cm.refreshInline.of(null) });
    this.invalidateCode();
  }

  // The renderer and canvas target start with the first build, so a page whose
  // demos this browser cannot run never claims a GPU context.
  ensureTarget() {
    this.targetReady ??= (async () => {
      this.renderer = new this.fc.Renderer();
      this.target = await this.renderer.createTarget(this.canvas);
    })().catch((err) => {
      this.targetReady = null;
      this.root.querySelector(".fcp-fallback").hidden = false;
      throw err;
    });
    return this.targetReady;
  }

  adjust(control, value) {
    try {
      control.shader.set(control.key, value);
      if (!this.writeValue(control.key, value)) this.overrides.set(control.key, value);
      this.invalidateCode();
    } catch (err) {
      this.showError(err);
    }
  }

  // The generated tabs follow the library, the shader files, and the control values.
  currentModel() {
    const library = parseLibrary(this.text("js")) ?? this.demo.library;
    const values = [];
    const seen = new Set();
    const program = this.program;
    if (program) {
      for (const shader of program.shaders) {
        for (const key of program.setupKeys.get(shader) ?? []) {
          const control = this.controls.find((c) => c.key === key && c.shader === shader);
          if (seen.has(key) || !control) continue;
          seen.add(key);
          try {
            values.push([key, plain(shader.get(key)), control.scalar]);
          } catch {
            // the key left the shader
          }
        }
      }
    } else {
      values.push(...Object.entries(this.demo.values ?? {}));
    }
    return this.model(this.demo, library, values);
  }

  invalidateCode() {
    this.codeStale = true;
    clearTimeout(this.codeTimer);
    const active = this.tabs?.find((t) => t.id === this.activeTab);
    if (active?.generated) this.codeTimer = setTimeout(() => this.refreshCode(true), 120);
  }

  refreshCode(apply = false) {
    let code;
    try {
      code = generateCode(this.currentModel());
    } catch (err) {
      console.warn("[playground] codegen failed", err);
      return;
    }
    this.codeStale = false;
    for (const tab of this.tabs.filter((t) => t.generated)) {
      tab.state = this.editorState(code[tab.id] ?? "", tab.kind, false);
      if (apply && tab.id === this.activeTab) {
        const scroll = this.view.scrollDOM.scrollTop;
        this.view.setState(tab.state);
        this.view.requestMeasure({ read: () => null, write: () => (this.view.scrollDOM.scrollTop = scroll) });
      }
    }
  }

  renderLibrary() {
    const slugs = parseLibrary(this.text("js"));
    this.ui.add.disabled = slugs === null;
    this.ui.add.title = slugs === null ? "Add a `const LIBRARY = [...];` line to main.js to use the picker." : "";
    this.ui.library.replaceChildren(
      ...(slugs ?? []).map((slug) => {
        const chip = document.createElement("span");
        chip.className = "fcp-chip";
        const name = document.createElement("a");
        const [category, fn] = slug.split("/");
        name.href = `/shaders/${category}/#${fn}`;
        name.textContent = slug;
        name.target = "_blank";
        const remove = document.createElement("button");
        remove.type = "button";
        remove.setAttribute("aria-label", `Remove ${slug}`);
        remove.textContent = "×";
        remove.addEventListener("click", () => this.toggleSlug(slug, false));
        chip.append(name, remove);
        return chip;
      }),
    );
  }

  toggleSlug(slug, selected) {
    const js = this.text("js");
    const slugs = parseLibrary(js);
    if (slugs === null) return;
    const next = selected ? [...new Set([...slugs, slug])] : slugs.filter((s) => s !== slug);
    this.setText("js", replaceLibrary(js, next));
    this.renderLibrary();
    this.scheduleBuild();
  }

  // A first build also claims the GPU and fetches the library cold, which can
  // fail once on a slow phone. With nothing drawn yet, it tries twice more.
  retryOrFail(err, token, demo) {
    if (token !== this.buildToken || demo !== this.demo) return;
    if (!this.program && this.attempts < 2) {
      this.attempts++;
      this.setStatus(`${this.backend} · retrying`);
      clearTimeout(this.timer);
      this.timer = setTimeout(() => this.build(), 300 * this.attempts);
      return;
    }
    this.fail(err);
  }

  // A failed build leaves the last good one drawing. With none, the canvas goes blank.
  fail(err) {
    if (this.program) return this.showError(err);
    this.showError(err, "The canvas stays empty until the code builds.");
    this.canvas.style.visibility = "hidden";
    this.setStatus(`${this.backend} · build failed`);
  }

  showError(err, note = this.program ? "Still showing the last version that built." : "") {
    const box = this.ui.error;
    box.hidden = !err;
    box.textContent = err ? [message(err), note].filter(Boolean).join("\n\n") : "";
  }

  setStatus(text) {
    if (this.ui.status) this.ui.status.textContent = text;
  }

  draw(program, now) {
    const size = [this.canvas.width, this.canvas.height];
    const pointer = this.pointer ?? [size[0] / 2, size[1] / 2];
    const gesture = this.gesture;
    this.gesture = { zoom: 0, pan: [0, 0] };
    const time = (now - program.epoch) / 1000;
    program.module.frame(program.state, this.renderer, this.target, time, size, pointer, gesture);
  }

  tick(now) {
    if (this.destroyed) return;
    this.raf = requestAnimationFrame((t) => this.tick(t));
    const program = this.program;
    if (this.offscreen || !program || program.failed) return;
    this.resize();
    try {
      this.draw(program, now);
      program.misses = 0;
    } catch (err) {
      // A frame that throws after its build hands the canvas back to the build
      // before it. With none to go back to, it takes three misses in a row.
      program.misses = (program.misses ?? 0) + 1;
      if (!program.previous && program.misses < 3) return;
      program.failed = true;
      this.program = program.previous;
      this.showError(err, this.program ? "Back to the last version that drew." : "Drawing stopped.");
    }
  }

  resize() {
    const dpr = Math.min(window.devicePixelRatio || 1, DPR_CAP);
    const w = Math.max(1, Math.round(this.canvas.clientWidth * dpr));
    const h = Math.max(1, Math.round(this.canvas.clientHeight * dpr));
    if (w === this.canvas.width && h === this.canvas.height) return;
    this.canvas.width = w;
    this.canvas.height = h;
    this.target?.resize([w, h]);
  }

  // Turns the wheel, a drag and a two-finger pinch over the canvas into the
  // gesture frame receives: zoom in doublings (positive zooms in) and pan in
  // canvas pixels. Only a demo with `gestures: true` takes them; for the rest
  // the wheel scrolls the page as usual.
  wireGestures() {
    const { canvas } = this;
    const scale = () => canvas.width / Math.max(canvas.getBoundingClientRect().width, 1);
    const spread = () => {
      const [a, b] = [...this.touches.values()];
      return Math.hypot(a[0] - b[0], a[1] - b[1]);
    };
    const middle = () => {
      const points = [...this.touches.values()];
      return points.reduce(([x, y], [px, py]) => [x + px / points.length, y + py / points.length], [0, 0]);
    };
    this.on(
      canvas,
      "wheel",
      (e) => {
        if (!this.demo?.gestures) return;
        e.preventDefault();
        const lines = e.deltaMode === 1 ? 16 : e.deltaMode === 2 ? canvas.clientHeight : 1;
        // A trackpad pinch arrives as a wheel with ctrlKey and small deltas.
        this.gesture.zoom -= (e.deltaY * lines) / (e.ctrlKey ? 100 : 400);
      },
      { passive: false },
    );
    this.on(canvas, "pointerdown", (e) => {
      if (!this.demo?.gestures || (e.pointerType === "mouse" && e.button !== 0)) return;
      canvas.setPointerCapture(e.pointerId);
      this.touches.set(e.pointerId, [e.clientX, e.clientY]);
    });
    this.on(canvas, "pointermove", (e) => {
      const last = this.touches.get(e.pointerId);
      if (!last) return;
      const before = this.touches.size === 2 ? { spread: spread(), middle: middle() } : null;
      this.touches.set(e.pointerId, [e.clientX, e.clientY]);
      const ratio = scale();
      if (before) {
        // A pinch zooms around the point between the two fingers.
        const now = middle();
        const rect = canvas.getBoundingClientRect();
        this.pointer = [(now[0] - rect.left) * ratio, (now[1] - rect.top) * ratio];
        this.gesture.zoom += Math.log2(Math.max(spread(), 1) / Math.max(before.spread, 1));
        this.gesture.pan[0] += (now[0] - before.middle[0]) * ratio;
        this.gesture.pan[1] += (now[1] - before.middle[1]) * ratio;
      } else if (this.touches.size === 1) {
        this.gesture.pan[0] += (e.clientX - last[0]) * ratio;
        this.gesture.pan[1] += (e.clientY - last[1]) * ratio;
      }
    });
    const release = (e) => this.touches.delete(e.pointerId);
    this.on(canvas, "pointerup", release);
    this.on(canvas, "pointercancel", release);
  }

  wire() {
    const { canvas, ui } = this;
    const track = (e) => {
      const rect = canvas.getBoundingClientRect();
      this.pointer = [
        ((e.clientX - rect.left) / rect.width) * canvas.width,
        ((e.clientY - rect.top) / rect.height) * canvas.height,
      ];
    };
    this.on(canvas, "pointermove", track);
    this.on(canvas, "pointerdown", track);
    this.wireGestures();
    this.cleanups.push(this.cm.watchSiteTheme(() => this.applyTheme()));

    this.on(ui.add, "click", () => this.picker.open(parseLibrary(this.text("js")) ?? []));
    this.on(ui.reset, "click", () => {
      this.edits.delete(this.demo.id);
      this.demo = null;
      this.select(this.root.dataset.active);
    });
    this.on(ui.copy, "click", async () => {
      try {
        await navigator.clipboard.writeText(this.view.state.doc.toString());
        ui.copy.classList.add("done");
        ui.copy.setAttribute("aria-label", "Copied");
        setTimeout(() => {
          ui.copy.classList.remove("done");
          ui.copy.setAttribute("aria-label", "Copy this file");
        }, 1200);
      } catch {
        // clipboard blocked; selection still works
      }
    });
    if (ui.fullscreen && document.fullscreenEnabled) {
      const frame = canvas.parentElement;
      ui.fullscreen.hidden = false;
      this.on(ui.fullscreen, "click", () => {
        if (document.fullscreenElement === frame) document.exitFullscreen();
        else frame.requestFullscreen().catch(() => {});
      });
      this.on(document, "fullscreenchange", () => {
        const label = document.fullscreenElement === frame ? "Exit fullscreen" : "Fullscreen";
        ui.fullscreen.setAttribute("aria-label", label);
        ui.fullscreen.title = label;
      });
    }
    const installSelect = this.root.querySelector(".fcp-install-select");
    if (installSelect) this.on(installSelect, "change", () => this.showInstall(installSelect.value));
    const langs = [...this.root.querySelectorAll(".fcp-install-langs button")];
    for (const button of langs) this.on(button, "click", () => this.showInstall(button.dataset.lang));
    if (langs.length) {
      this.on(langs[0].parentElement, "keydown", (e) => {
        const index = langs.findIndex((b) => b.getAttribute("aria-selected") === "true");
        const step = { ArrowRight: index + 1, ArrowLeft: index - 1, Home: 0, End: langs.length - 1 }[e.key];
        if (step === undefined) return;
        e.preventDefault();
        const next = langs[(step + langs.length) % langs.length];
        this.showInstall(next.dataset.lang);
        next.focus();
      });
    }
    for (const button of this.root.querySelectorAll(".fcp-install-copy")) {
      this.on(button, "click", () => {
        const code = button.parentElement?.querySelector("code")?.textContent ?? "";
        navigator.clipboard?.writeText(code).catch(() => {});
      });
    }
    this.on(ui.tabs, "keydown", (e) => {
      const count = this.tabs.length;
      const index = this.tabs.findIndex((t) => t.id === this.activeTab);
      const step = { ArrowRight: index + 1, ArrowLeft: index - 1, Home: 0, End: count - 1 }[e.key];
      if (step === undefined) return;
      e.preventDefault();
      const next = this.tabs[(step + count) % count];
      this.showTab(next.id);
      ui.tabs.querySelector(`[data-tab="${CSS.escape(next.id)}"]`)?.focus();
    });

    // Hidden tabs get no animation frames, so only scrolling away needs a pause.
    if (typeof IntersectionObserver !== "undefined") {
      const io = new IntersectionObserver((entries) => (this.offscreen = !entries.at(-1).isIntersecting), { threshold: 0.05 });
      io.observe(canvas);
      this.cleanups.push(() => io.disconnect());
    }
  }

  destroy() {
    this.destroyed = true;
    cancelAnimationFrame(this.raf);
    clearTimeout(this.timer);
    clearTimeout(this.codeTimer);
    for (const fn of this.cleanups) fn();
    this.cleanups = [];
    this.picker?.close();
    this.view?.destroy();
  }
}

// CodeMirror loads with the playground, never with the page.
async function loadEditor() {
  const [{ EditorView, basicSetup }, { EditorState, Annotation }, { javascript }, { wgsl }, { highlight }, inline, theme] =
    await Promise.all([
      import("codemirror"),
      import("@codemirror/state"),
      import("@codemirror/lang-javascript"),
      import("./wgsl.js"),
      import("./highlight.js"),
      import("./inline.js"),
      import("./theme.js"),
    ]);
  // These rules come before the theme and win where both set a property.
  // The current line takes its color from the theme and paints behind the text.
  const extensions = [
    basicSetup,
    EditorView.theme({
      "&": { height: "100%", fontSize: "13px" },
      ".cm-scroller": { fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace", lineHeight: "1.55" },
      ".cm-gutters": { border: "none" },
      ".cm-activeLine": { position: "relative", backgroundColor: "transparent" },
      ".cm-activeLine::before": {
        content: '""',
        position: "absolute",
        inset: "0",
        zIndex: "-10",
        pointerEvents: "none",
      },
    }),
  ];
  const languages = (kind) => {
    if (kind === "wgsl") return wgsl();
    if (kind === "js") return javascript();
    return highlight(kind);
  };
  // Inline sliders and swatches; on main.js `.set` values they share the control's range.
  const inlineFor = (kind, host) =>
    inline.inlineControls({
      language: kind === "wgsl" ? "wgsl" : "js",
      rangeFor: (state, from, to) => (kind === "js" ? host.rangeAt(state, from, to) : null),
      isColor: (state, from, to) => {
        const found = kind === "js" ? host.setValueAt(state, from, to) : null;
        if (!found) return false;
        return found.control ? found.control.color : /colou?r|tint/i.test(found.key);
      },
    });
  // Edits from the controls into main.js are tagged so they do not rebuild.
  const synced = Annotation.define();
  return {
    EditorView,
    EditorState,
    extensions,
    theme: theme.editorTheme(),
    watchSiteTheme: theme.watchSiteTheme,
    languages,
    inline: inlineFor,
    synced,
    scrubbed: inline.scrubbed,
    refreshInline: inline.refreshInline,
  };
}
