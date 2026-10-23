// Developer playground host.
//
// Each demo is a JS program plus WGSL files. The host rebuilds the program as
// the code changes, keeps the last good build drawing when a build fails,
// generates uniform controls from the shader's own keys, and keeps the Rust,
// Python, Swift, and Kotlin tabs in step with what is running.

import { loadProgram } from "./program.js";
import { describeControls, renderControls } from "./controls.js";
import { LANGUAGES, generateCode, parseLibrary, replaceLibrary } from "./codegen.js";
import { LibraryPicker } from "./picker.js";
import { loadFragmentColor } from "../runtime.js";

const BUILD_DELAY = 250;
const DPR_CAP = 2;

let runtime;
let instances = 0;
let recorder = null;
let gpuCheck;

async function loadRuntime() {
  const fc = await loadFragmentColor();
  globalThis.__fragmentcolor = fc;

  // Records which keys a program sets, so controls skip the ones frame drives.
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
// releases throw. Capturing console.error covers both.
async function capturing(run) {
  const errors = [];
  const original = console.error;
  console.error = (...args) => {
    errors.push(args.map((a) => (a instanceof Error ? a.message : String(a))).join(" "));
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

function plain(value) {
  if (typeof value === "number") return Math.round(value * 1e4) / 1e4;
  return Array.from(value, (n) => Math.round(n * 1e4) / 1e4);
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
      demos: root.querySelector(".fcp-demos"),
      summary: root.querySelector(".fcp-summary"),
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
    };
    this.edits = new Map();
    this.overrides = new Map();
    this.pointer = null;
    this.program = null;
    this.buildToken = 0;
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
    this.select(id);
    this.raf = requestAnimationFrame((t) => this.tick(t));
  }

  // Swaps in another demo. Edits made to the current one stay in memory until reset.
  select(id) {
    const demo = this.demos.find((d) => d.id === id) ?? this.demos[0];
    clearTimeout(this.timer);
    this.buildToken++;
    if (this.demo) this.edits.set(this.demo.id, this.snapshot());
    this.demo = demo;
    this.overrides.clear();
    this.program = null;
    this.controls = [];
    renderControls(this.ui.controls, [], () => {});
    for (const button of this.ui.demos?.querySelectorAll("button") ?? []) {
      button.setAttribute("aria-pressed", String(button.dataset.demo === demo.id));
    }
    if (this.ui.summary) this.ui.summary.textContent = demo.summary;
    this.root.dataset.active = demo.id;
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

  // Editor tabs: the WGSL files and main.js are editable; the other languages are generated.
  loadTabs({ files, js }) {
    const tabs = [];
    for (const [name, text] of Object.entries(files)) {
      tabs.push({ id: name, label: name, kind: "wgsl", state: this.editorState(text, "wgsl", true) });
    }
    tabs.push({ id: "js", label: "main.js", kind: "js", state: this.editorState(js, "js", true) });
    for (const lang of LANGUAGES.filter((l) => l.id !== "js")) {
      tabs.push({ id: lang.id, label: lang.label, file: lang.file, kind: lang.id, generated: true, state: null });
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
        }
        button.addEventListener("click", () => this.showTab(tab.id));
        return button;
      }),
    );
    this.activeTab = null;
    this.showTab(tabs[0].id);
    this.renderLibrary();
  }

  editorState(text, kind, editable) {
    const { EditorState, EditorView, extensions, languages } = this.cm;
    return EditorState.create({
      doc: text,
      extensions: [
        extensions,
        languages(kind),
        EditorState.readOnly.of(!editable),
        EditorView.editable.of(editable),
        EditorView.updateListener.of((update) => {
          if (!update.docChanged || !editable) return;
          const tab = this.tabs.find((t) => t.id === this.activeTab);
          if (tab) tab.state = update.state;
          this.scheduleBuild();
        }),
      ],
    });
  }

  showTab(id) {
    const current = this.tabs.find((t) => t.id === this.activeTab);
    if (current) {
      current.scroll = this.view.scrollSnapshot();
      if (!current.generated) current.state = this.view.state;
    }
    const tab = this.tabs.find((t) => t.id === id) ?? this.tabs[0];
    this.activeTab = tab.id;
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
    this.root.dataset.tab = tab.generated ? "generated" : "source";
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

  scheduleBuild() {
    clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.renderLibrary();
      if (this.blocked) this.invalidateCode();
      else this.build();
    }, BUILD_DELAY);
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
      if (!map.has(shader)) map.set(shader, new Set());
      map.get(shader).add(key);
    };

    let module;
    let state;
    try {
      await this.ensureTarget();
      module = await loadProgram(this.text("js"), files);
      recorder = note(setupKeys);
      state = await capturing(() => module.setup(this.renderer, this.target));
    } catch (err) {
      if (token === this.buildToken && demo === this.demo) this.showError(err);
      return;
    } finally {
      recorder = null;
    }
    if (token !== this.buildToken || this.destroyed || demo !== this.demo) return;

    const shaders = shadersIn(state, this.fc.Shader);
    for (const [key, value] of this.overrides) {
      for (const shader of shaders) {
        if (shader.listKeys().includes(key)) {
          try {
            shader.set(key, value);
          } catch {
            this.overrides.delete(key);
          }
        }
      }
    }

    const previous = this.program && !this.program.failed ? { ...this.program, previous: null } : null;
    const program = { module, state, shaders, setupKeys, previous, epoch: previous?.epoch ?? performance.now() };
    try {
      recorder = note(frameKeys);
      await capturing(() => this.draw(program, performance.now()));
    } catch (err) {
      if (token === this.buildToken && demo === this.demo) this.showError(err);
      return;
    } finally {
      recorder = null;
    }

    this.program = program;
    this.showError(null);
    this.setStatus(`${this.backend} · built in ${Math.round(performance.now() - started)} ms`);
    this.controls = describeControls(
      shaders.map((shader) => ({ shader, hidden: frameKeys.get(shader) ?? new Set() })),
      Object.values(files),
    );
    renderControls(this.ui.controls, this.controls, (control, value) => this.adjust(control, value));
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
      this.overrides.set(control.key, value);
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
        const keys = [...(program.setupKeys.get(shader) ?? []), ...this.overrides.keys()];
        for (const key of keys) {
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
    program.module.frame(program.state, this.renderer, this.target, (now - program.epoch) / 1000, size, pointer);
  }

  tick(now) {
    if (this.destroyed) return;
    this.raf = requestAnimationFrame((t) => this.tick(t));
    const program = this.program;
    if (this.offscreen || !program || program.failed) return;
    this.resize();
    try {
      this.draw(program, now);
    } catch (err) {
      // A frame that throws after its build hands the canvas back to the build before it.
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

    for (const button of ui.demos?.querySelectorAll("button") ?? []) {
      this.on(button, "click", () => this.select(button.dataset.demo));
    }
    this.on(ui.add, "click", () => this.picker.open(parseLibrary(this.text("js")) ?? []));
    this.on(ui.reset, "click", () => {
      this.edits.delete(this.demo.id);
      this.demo = null;
      this.select(this.root.dataset.active);
    });
    this.on(ui.copy, "click", async () => {
      try {
        await navigator.clipboard.writeText(this.view.state.doc.toString());
        ui.copy.textContent = "Copied";
        setTimeout(() => (ui.copy.textContent = "Copy"), 1200);
      } catch {
        // clipboard blocked; selection still works
      }
    });
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
      const io = new IntersectionObserver(([entry]) => (this.offscreen = !entry.isIntersecting), { threshold: 0.05 });
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
  const [{ EditorView, basicSetup }, { EditorState }, { javascript }, { dracula }, { wgsl }, { highlight }] =
    await Promise.all([
      import("codemirror"),
      import("@codemirror/state"),
      import("@codemirror/lang-javascript"),
      import("thememirror"),
      import("./wgsl.js"),
      import("./highlight.js"),
    ]);
  const extensions = [
    basicSetup,
    dracula,
    EditorView.theme({
      "&": { height: "100%", fontSize: "13px" },
      ".cm-scroller": { fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace", lineHeight: "1.55" },
      ".cm-gutters": { border: "none" },
    }),
  ];
  const languages = (kind) => {
    if (kind === "wgsl") return wgsl();
    if (kind === "js") return javascript();
    return highlight(kind);
  };
  return { EditorView, EditorState, extensions, languages };
}
