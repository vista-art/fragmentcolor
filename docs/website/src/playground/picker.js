// Library picker for the playground: a searchable popover over the shader
// catalog. Pressing a card toggles its slug, and the playground decides what
// that does to the program.

const ENDPOINT = "/playground/library.json";
const STYLE_ID = "fcp-picker-style";
const ALL = "";
const SELECTED = ":selected";

let library;
let panels = 0;

// Fetches the catalog once per page. A failed request is retried on the next call.
export function loadLibrary() {
  library ??= fetch(ENDPOINT)
    .then((response) => {
      if (!response.ok) throw new Error(`${ENDPOINT} returned ${response.status}`);
      return response.json();
    })
    .catch((error) => {
      library = undefined;
      throw error;
    });
  return library;
}

function element(tag, className, attributes = {}) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  for (const [name, value] of Object.entries(attributes)) node.setAttribute(name, value);
  return node;
}

export class LibraryPicker {
  /**
   * @param {HTMLElement} host element the panel is placed in
   * @param {object} [options]
   * @param {(slug: string, selected: boolean) => void} [options.onToggle] called after a card is pressed
   * @param {() => void} [options.onClose] called when the panel closes
   * @param {HTMLElement} [options.trigger] button that opens the panel; presses on it do not count as outside
   */
  constructor(host, { onToggle, onClose, trigger } = {}) {
    this.host = host;
    this.onToggle = onToggle;
    this.onClose = onClose;
    this.trigger = trigger;
    this.selected = new Set();
    this.entries = new Map();
    this.query = "";
    this.category = ALL;
    this.cards = [];
    this.panel = null;
    this.observer = null;
    this.returnFocus = null;
    this.handleKey = (event) => this.closeOnEscape(event);
    this.handlePointer = (event) => this.closeOnOutside(event);
  }

  get isOpen() {
    return Boolean(this.panel && !this.panel.hidden);
  }

  /** Shows the panel with `selected` (an iterable of slugs) marked as pressed. */
  async open(selected = []) {
    this.build();
    this.setSelected(selected);
    if (!this.isOpen) {
      this.returnFocus = document.activeElement;
      this.panel.hidden = false;
      this.trigger?.setAttribute("aria-expanded", "true");
      document.addEventListener("keydown", this.handleKey);
      document.addEventListener("pointerdown", this.handlePointer, true);
    }
    this.search.focus();
    await this.fill();
  }

  /** Hides the panel. With `restoreFocus`, focus returns to where it was before `open`. */
  close(restoreFocus = false) {
    if (!this.isOpen) return;
    this.panel.hidden = true;
    this.trigger?.setAttribute("aria-expanded", "false");
    document.removeEventListener("keydown", this.handleKey);
    document.removeEventListener("pointerdown", this.handlePointer, true);
    if (restoreFocus && this.returnFocus?.isConnected) this.returnFocus.focus();
    this.returnFocus = null;
    this.onClose?.();
  }

  /** Opens the panel when closed and closes it when open. */
  toggle(selected = []) {
    if (this.isOpen) {
      this.close(true);
      return Promise.resolve();
    }
    return this.open(selected);
  }

  /** Marks `selected` as pressed without opening or closing the panel. */
  setSelected(selected) {
    this.selected = new Set(selected);
    this.updateCards();
  }

  /** Removes the panel from the page. */
  destroy() {
    this.close();
    this.observer?.disconnect();
    this.panel?.remove();
    this.panel = null;
    this.selectedChip = null;
    this.cards = [];
    this.entries.clear();
  }

  build() {
    if (this.panel) return;
    injectStyle();
    if (getComputedStyle(this.host).position === "static") this.host.style.position = "relative";

    const id = `fcp-picker-${++panels}`;
    const panel = element("div", "fcp-picker not-content", { id, role: "dialog", "aria-label": "Shader library" });
    panel.hidden = true;

    const head = element("div", "fcp-picker-head");
    this.search = element("input", "fcp-picker-search", {
      type: "search",
      placeholder: "Search the shader library",
      "aria-label": "Search the shader library",
      autocomplete: "off",
      spellcheck: "false",
    });
    this.search.addEventListener("input", () => {
      this.query = this.search.value;
      this.applyFilter();
    });
    this.search.addEventListener("keydown", (event) => {
      if (event.key !== "ArrowDown") return;
      event.preventDefault();
      this.visibleCards()[0]?.focus();
    });
    const close = element("button", "fcp-picker-close", { type: "button", "aria-label": "Close the library" });
    close.innerHTML =
      '<svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg>';
    close.addEventListener("click", () => this.close(true));
    head.append(this.search, close);

    this.chips = element("div", "fcp-picker-chips", { role: "group", "aria-label": "Filter by category" });
    this.chips.addEventListener("click", (event) => {
      const chip = event.target.closest(".fcp-picker-chip");
      if (chip) this.filterCategory(chip.dataset.category);
    });

    this.status = element("p", "fcp-picker-status", { "aria-live": "polite" });

    this.grid = element("div", "fcp-picker-grid", { role: "group", "aria-label": "Shader functions" });
    this.grid.addEventListener("click", (event) => {
      const card = event.target.closest(".fcp-picker-card");
      if (card) this.toggleCard(card);
    });
    this.grid.addEventListener("keydown", (event) => this.moveFocus(event));

    // Focus that leaves the panel would land on code the panel covers.
    panel.addEventListener("focusout", (event) => {
      const next = event.relatedTarget;
      if (next instanceof Node && (panel.contains(next) || this.trigger?.contains(next))) return;
      if (next) this.close();
    });

    panel.append(head, this.chips, this.status, this.grid);
    this.host.append(panel);
    this.panel = panel;

    if (this.trigger) {
      this.trigger.setAttribute("aria-controls", id);
      this.trigger.setAttribute("aria-expanded", "false");
    }
  }

  async fill() {
    if (this.cards.length) return;
    this.status.textContent = "Loading the shader library…";
    let entries;
    try {
      entries = await loadLibrary();
    } catch {
      this.status.textContent = "The shader library could not be loaded. Close and reopen to retry.";
      return;
    }
    if (!this.panel || this.cards.length) return;
    this.renderChips(entries);
    this.renderCards(entries);
    this.setSelected(this.selected);
    this.applyFilter();
  }

  renderChips(entries) {
    const categories = new Map();
    for (const entry of entries) {
      const known = categories.get(entry.category);
      if (known) known.count++;
      else categories.set(entry.category, { title: entry.categoryTitle || entry.category, count: 1 });
    }
    const chip = (category, label, title) => {
      const button = element("button", "fcp-picker-chip", { type: "button", "data-category": category, title });
      button.textContent = label;
      return button;
    };
    this.chips.append(chip(ALL, "All", `All ${entries.length} functions`));
    this.selectedChip = chip(SELECTED, "Selected", "Functions in this program");
    this.chips.append(this.selectedChip);
    for (const [id, { title, count }] of categories) {
      this.chips.append(chip(id, id, `${title} (${count})`));
    }
  }

  renderCards(entries) {
    if ("IntersectionObserver" in window) {
      this.observer = new IntersectionObserver((records) => this.reveal(records), {
        root: this.grid,
        rootMargin: "200px 0px",
      });
    }
    const fragment = document.createDocumentFragment();
    for (const entry of entries) {
      const card = element("button", "fcp-picker-card", {
        type: "button",
        "aria-pressed": "false",
        "data-slug": entry.slug,
        "data-category": entry.category,
      });
      card.dataset.search = `${entry.slug} ${entry.summary} ${entry.signature}`.toLowerCase();

      const thumb = element("span", "fcp-picker-thumb");
      if (entry.preview) {
        const image = element("img", "", { alt: "", width: "256", height: "256", decoding: "async" });
        image.dataset.src = entry.preview;
        image.addEventListener("load", () => image.classList.add("fcp-picker-loaded"));
        image.addEventListener("error", () => image.remove());
        thumb.append(image);
      }

      const slug = element("span", "fcp-picker-slug");
      const prefix = element("span", "fcp-picker-prefix");
      prefix.textContent = `${entry.category}/`;
      slug.append(prefix);
      // Long names wrap after an underscore rather than mid-word.
      for (const part of entry.name.split(/(?<=_)/)) slug.append(part, element("wbr"));

      card.append(thumb, slug);
      fragment.append(card);
      this.cards.push(card);
      this.entries.set(entry.slug, entry);
    }
    this.grid.append(fragment);
    for (const image of this.grid.querySelectorAll("img")) {
      if (this.observer) this.observer.observe(image);
      else this.load(image);
    }
  }

  reveal(records) {
    for (const record of records) {
      if (!record.isIntersecting) continue;
      this.observer.unobserve(record.target);
      this.load(record.target);
    }
  }

  load(image) {
    if (image.dataset.src) image.src = image.dataset.src;
    delete image.dataset.src;
  }

  filterCategory(category) {
    this.category = category;
    this.applyFilter();
  }

  applyFilter() {
    for (const chip of this.chips.children) {
      chip.setAttribute("aria-pressed", String(chip.dataset.category === this.category));
    }
    if (!this.cards.length) return;
    const terms = this.query.toLowerCase().split(/\s+/).filter(Boolean);
    let shown = 0;
    for (const card of this.cards) {
      const inCategory =
        this.category === ALL ||
        (this.category === SELECTED ? this.selected.has(card.dataset.slug) : card.dataset.category === this.category);
      const visible = inCategory && terms.every((term) => card.dataset.search.includes(term));
      card.hidden = !visible;
      if (visible) shown++;
    }
    this.grid.scrollTop = 0;
    this.status.textContent =
      shown === 0
        ? "No functions match."
        : shown === this.cards.length
          ? `${shown} functions`
          : `${shown} of ${this.cards.length} functions`;
  }

  // Selecting a slug that declares a name a selected slug also declares swaps them,
  // since one WGSL module cannot define the same name twice.
  toggleCard(card) {
    const slug = card.dataset.slug;
    const selected = !this.selected.has(slug);
    const replaced = selected ? this.clashes(slug) : [];
    for (const other of replaced) {
      this.selected.delete(other);
      this.onToggle?.(other, false);
    }
    if (selected) this.selected.add(slug);
    else this.selected.delete(slug);
    this.updateCards();
    if (replaced.length) this.status.textContent = `Replaced ${replaced.join(" and ")} with ${slug}.`;
    this.onToggle?.(slug, selected);
  }

  /** Selected slugs that declare a name `slug` also declares. */
  clashes(slug) {
    const names = this.entries.get(slug)?.defines ?? [];
    return [...this.selected].filter(
      (other) => other !== slug && (this.entries.get(other)?.defines ?? []).some((name) => names.includes(name)),
    );
  }

  updateCards() {
    for (const card of this.cards) {
      const slug = card.dataset.slug;
      const entry = this.entries.get(slug);
      const pressed = this.selected.has(slug);
      const clashes = pressed ? [] : this.clashes(slug);
      card.setAttribute("aria-pressed", String(pressed));
      card.toggleAttribute("data-clash", clashes.length > 0);
      card.title = [describeClash(entry, clashes, this.entries), entry.signature, entry.summary]
        .filter(Boolean)
        .join("\n\n");
    }
    this.updateSelectedChip();
  }

  updateSelectedChip() {
    if (this.selectedChip) {
      this.selectedChip.textContent = this.selected.size ? `Selected ${this.selected.size}` : "Selected";
    }
  }

  visibleCards() {
    return this.cards.filter((card) => !card.hidden);
  }

  // Arrow keys walk the grid; Up from the first row returns to the search box.
  moveFocus(event) {
    const steps = { ArrowLeft: -1, ArrowRight: 1, ArrowUp: -this.columns(), ArrowDown: this.columns() };
    const step = steps[event.key];
    if (!step) return;
    const cards = this.visibleCards();
    const index = cards.indexOf(document.activeElement);
    if (index < 0) return;
    event.preventDefault();
    const next = index + step;
    if (next < 0) this.search.focus();
    else cards[Math.min(next, cards.length - 1)].focus();
  }

  columns() {
    const template = getComputedStyle(this.grid).gridTemplateColumns;
    return Math.max(1, template.split(" ").filter(Boolean).length);
  }

  closeOnEscape(event) {
    if (event.key !== "Escape") return;
    event.preventDefault();
    this.close(true);
  }

  closeOnOutside(event) {
    const path = event.composedPath();
    if (path.includes(this.panel) || (this.trigger && path.includes(this.trigger))) return;
    this.close();
  }
}

function describeClash(entry, clashes, entries) {
  if (!clashes.length) return "";
  const shared = entry.defines.filter((name) => clashes.some((slug) => entries.get(slug).defines.includes(name)));
  const verb = clashes.length > 1 ? "define" : "defines";
  return `Replaces ${clashes.join(" and ")}, which also ${verb} ${shared.join(", ")}.`;
}

function injectStyle() {
  if (document.getElementById(STYLE_ID)) return;
  const style = element("style", "", { id: STYLE_ID });
  style.textContent = STYLE;
  document.head.append(style);
}

const CHECK =
  "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Cpath d='M3.5 8.5l3 3 6-7' fill='none' stroke='white' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E\")";

const SWAP =
  "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Cpath d='M2.5 5.5h10l-2.5-2.5M13.5 10.5h-10l2.5 2.5' fill='none' stroke='white' stroke-width='1.6' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E\")";

const STYLE = `
.fcp-picker {
  --fcp-picker-bg: var(--sl-color-bg-nav, hsl(224, 14%, 16%));
  --fcp-picker-field: var(--sl-color-black, hsl(224, 10%, 10%));
  --fcp-picker-line: var(--sl-color-hairline-light, hsl(224, 10%, 23%));
  --fcp-picker-text: var(--sl-color-text, hsl(224, 6%, 77%));
  --fcp-picker-strong: var(--sl-color-white, hsl(0, 0%, 100%));
  --fcp-picker-muted: var(--sl-color-gray-3, hsl(224, 6%, 56%));
  --fcp-picker-accent: var(--sl-color-accent, hsl(224, 100%, 60%));
  --fcp-picker-accent-low: var(--sl-color-accent-low, hsl(224, 54%, 20%));
  --fcp-picker-accent-high: var(--sl-color-accent-high, hsl(224, 100%, 85%));
  --fcp-picker-inset: var(--fcp-picker-top, 0.5rem);
  position: absolute;
  z-index: 40;
  top: var(--fcp-picker-inset);
  left: 0;
  right: 0;
  margin-inline: auto;
  box-sizing: border-box;
  width: min(100%, var(--fcp-picker-width, 46rem));
  /* The host clips its overflow, so the panel also fits inside the host. */
  max-height: min(36rem, 78vh, calc(100% - 2 * var(--fcp-picker-inset)));
  max-height: min(36rem, 78dvh, calc(100% - 2 * var(--fcp-picker-inset)));
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
  padding: 16px;
  border: 1px solid var(--fcp-picker-line);
  border-radius: 0.75rem;
  background: var(--fcp-picker-bg);
  color: var(--fcp-picker-text);
  box-shadow: var(--sl-shadow-lg, 0 16px 40px hsla(0, 0%, 0%, 0.45));
  font-size: var(--sl-text-sm, 0.875rem);
  line-height: 1.4;
  text-align: left;
  container-type: inline-size;
}
.fcp-picker[hidden],
.fcp-picker-card[hidden] {
  display: none;
}
.fcp-picker *,
.fcp-picker *::before,
.fcp-picker *::after {
  box-sizing: border-box;
}
.fcp-picker-head,
.fcp-picker-chips,
.fcp-picker-status {
  flex: none;
}
.fcp-picker-head {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}
.fcp-picker-search {
  flex: 1;
  min-width: 0;
  height: 2.25rem;
  margin: 0;
  padding: 0 0.75rem;
  border: 1px solid var(--fcp-picker-line);
  border-radius: 0.5rem;
  background: var(--fcp-picker-field);
  color: var(--fcp-picker-strong);
  font: inherit;
  font-size: 16px;
}
.fcp-picker-search::placeholder {
  color: var(--fcp-picker-muted);
}
.fcp-picker-search:focus {
  outline: none;
  border-color: var(--fcp-picker-accent);
  box-shadow: 0 0 0 1px var(--fcp-picker-accent);
}
.fcp-picker-close {
  flex: none;
  display: grid;
  place-items: center;
  width: 2.25rem;
  height: 2.25rem;
  margin: 0;
  padding: 0;
  border: 1px solid transparent;
  border-radius: 0.5rem;
  background: transparent;
  color: var(--fcp-picker-muted);
  cursor: pointer;
}
.fcp-picker-close:hover {
  color: var(--fcp-picker-strong);
  border-color: var(--fcp-picker-line);
}
.fcp-picker-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}
.fcp-picker-chip {
  margin: 0;
  padding: 0.125rem 0.625rem;
  border: 1px solid var(--fcp-picker-line);
  border-radius: 999px;
  background: transparent;
  color: var(--fcp-picker-text);
  font: inherit;
  font-size: 0.75rem;
  line-height: 1.5rem;
  cursor: pointer;
  white-space: nowrap;
}
.fcp-picker-chip:hover {
  color: var(--fcp-picker-strong);
  border-color: var(--fcp-picker-muted);
}
.fcp-picker-chip[aria-pressed="true"] {
  border-color: var(--fcp-picker-accent);
  background: var(--fcp-picker-accent-low);
  color: var(--fcp-picker-accent-high);
}
.fcp-picker-status {
  margin: 0;
  color: var(--fcp-picker-muted);
  font-size: 0.75rem;
}
.fcp-picker-grid {
  flex: 1 1 auto;
  min-height: 7rem;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(112px, 1fr));
  align-content: start;
  gap: 0.5rem;
  margin: 0 -4px;
  padding: 4px;
  overflow-x: hidden;
  overflow-y: auto;
  overscroll-behavior: contain;
}
.fcp-picker-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  min-width: 0;
  margin: 0;
  padding: 0.375rem;
  border: 1px solid var(--fcp-picker-line);
  border-radius: 0.5rem;
  background: var(--fcp-picker-field);
  color: var(--fcp-picker-text);
  font: inherit;
  text-align: left;
  cursor: pointer;
  transition: border-color 120ms ease, background-color 120ms ease;
}
.fcp-picker-card:hover {
  border-color: var(--fcp-picker-muted);
}
.fcp-picker-card[aria-pressed="true"] {
  border-color: var(--fcp-picker-accent);
  background: var(--fcp-picker-accent-low);
  box-shadow: inset 0 0 0 1px var(--fcp-picker-accent);
}
.fcp-picker-thumb {
  position: relative;
  display: block;
  aspect-ratio: 1;
  overflow: hidden;
  border-radius: 0.3rem;
  background: var(--fcp-picker-bg);
}
.fcp-picker-thumb img {
  display: block;
  width: 100%;
  height: 100%;
  max-width: none;
  margin: 0;
  object-fit: cover;
  opacity: 0;
  transition: opacity 160ms ease;
}
.fcp-picker-thumb img.fcp-picker-loaded {
  opacity: 1;
}
.fcp-picker-card[data-clash] {
  border-style: dashed;
}
.fcp-picker-card[aria-pressed="true"] .fcp-picker-thumb::after,
.fcp-picker-card[data-clash] .fcp-picker-thumb::after {
  content: "";
  position: absolute;
  top: 0.375rem;
  right: 0.375rem;
  width: 1.25rem;
  height: 1.25rem;
  border-radius: 50%;
  background: var(--fcp-picker-accent) ${CHECK} center / 0.875rem no-repeat;
  box-shadow: 0 1px 3px hsla(0, 0%, 0%, 0.4);
}
.fcp-picker-card[data-clash] .fcp-picker-thumb::after {
  background: var(--fcp-picker-muted) ${SWAP} center / 0.875rem no-repeat;
}
.fcp-picker-slug {
  display: block;
  font-family: var(--sl-font-mono, ui-monospace, SFMono-Regular, Menlo, Consolas, monospace);
  font-size: 0.6875rem;
  line-height: 1.3;
  color: var(--fcp-picker-strong);
  overflow-wrap: anywhere;
}
.fcp-picker-prefix {
  display: block;
  color: var(--fcp-picker-muted);
}
.fcp-picker-chip:focus-visible,
.fcp-picker-card:focus-visible,
.fcp-picker-close:focus-visible {
  outline: 2px solid var(--fcp-picker-accent);
  outline-offset: 2px;
}
@media (max-width: 30rem) {
  .fcp-picker {
    --fcp-picker-inset: var(--fcp-picker-top, 0px);
    border-radius: 0.5rem;
  }
}
/* Narrow panels keep the chips on one scrolling row to leave room for the grid. */
@container (max-width: 30rem) {
  .fcp-picker-chips {
    flex-wrap: nowrap;
    margin: 0 -16px;
    padding: 2px 16px;
    overflow-x: auto;
    overscroll-behavior-x: contain;
    scrollbar-width: thin;
  }
  .fcp-picker-chip {
    flex: none;
  }
  .fcp-picker-grid {
    grid-template-columns: repeat(auto-fill, minmax(92px, 1fr));
  }
}
@media (prefers-reduced-motion: reduce) {
  .fcp-picker-card,
  .fcp-picker-thumb img {
    transition: none;
  }
}
`;
