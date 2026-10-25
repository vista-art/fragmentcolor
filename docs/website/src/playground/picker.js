// Library picker for the playground: a searchable popover over the shader
// catalog. Pressing a card toggles its slug, and the playground decides what
// that does to the program.

import { element, injectStyle } from "./panel.js";

const ENDPOINT = "/playground/library.json";
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
        image.addEventListener("error", () => this.retry(image));
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

  // One dropped request would leave a card blank for good, so a preview that
  // fails loads once more before its card says it has none.
  retry(image) {
    if (image.dataset.retried) {
      image.parentElement?.classList.add("fcp-picker-missing");
      image.remove();
      return;
    }
    image.dataset.retried = "1";
    const source = image.getAttribute("src");
    setTimeout(() => (image.src = `${source}${source.includes("?") ? "&" : "?"}retry=${Date.now()}`), 500);
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
