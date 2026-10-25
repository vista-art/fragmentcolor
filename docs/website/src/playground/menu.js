// The demo menu: a row of chips, one per demo, and a button that opens the picker.
//
// The row scrolls sideways and is one tab stop, walked with the arrow keys. The
// picker is a panel of demo cards like the shader library's, with category chips
// and a search box, in a dialog that keeps the page usable. Its arrow keys walk
// the cards, typed letters go to the search, Enter picks, and Escape closes it.

import { injectStyle } from "./panel.js";

const ALL = "";
// The panel keeps this much room under it, and never gets shorter than MIN_HEIGHT.
const GAP = 8;
const MIN_HEIGHT = 240;
// The width of the fade at each end of the demo row, which a chip in focus stays clear of.
const FADE = 48;

let pickers = 0;

/**
 * @param {HTMLElement} root the <fc-playground> element
 * @param {{ id: string, title: string, summary: string }[]} demos the demos the row and the panel offer
 * @param {(id: string) => void} onPick called with the demo the reader picks
 * @returns {{ show(id: string): void, unwire(): void }} `show` names a demo without calling `onPick`
 */
export function wireMenu(root, demos, onPick) {
  const button = root.querySelector("button.fcp-menu-button");
  const panel = root.querySelector(".fcp-demo-picker");
  const value = root.querySelector(".fcp-menu-value");
  const summary = root.querySelector(".fcp-summary");
  const strip = root.querySelector(".fcp-strip");
  const items = [...(strip?.querySelectorAll(".fcp-strip-item") ?? [])];
  const cards = [...(panel?.querySelectorAll(".fcp-picker-card") ?? [])];

  // Scrolls the row, not the page, to center a chip the row or its fades hide.
  const reveal = (item) => {
    const left = item.offsetLeft - strip.offsetLeft;
    const start = strip.scrollLeft + (strip.scrollLeft > 0 ? FADE : 0);
    const end = strip.scrollLeft + strip.clientWidth - FADE;
    if (left < start || left + item.offsetWidth > end) {
      strip.scrollLeft = left - (strip.clientWidth - item.offsetWidth) / 2;
    }
  };

  const show = (id) => {
    const demo = demos.find((d) => d.id === id);
    if (!demo) return;
    for (const card of cards) {
      if (card.dataset.demo === id) card.setAttribute("aria-current", "true");
      else card.removeAttribute("aria-current");
    }
    for (const item of items) {
      const pressed = item.dataset.demo === id;
      item.setAttribute("aria-pressed", String(pressed));
      item.tabIndex = pressed ? 0 : -1;
      if (pressed) reveal(item);
    }
    if (value) value.textContent = demo.title;
    if (summary) summary.textContent = demo.summary;
  };
  if (!button || !panel || !strip) return { show, unwire() {} };

  injectStyle();
  const search = panel.querySelector(".fcp-picker-search");
  const chips = [...panel.querySelectorAll(".fcp-picker-chip")];
  const status = panel.querySelector(".fcp-picker-status");
  const grid = panel.querySelector(".fcp-picker-grid");
  panel.id = `fcp-demos-${++pickers}`;
  button.setAttribute("aria-controls", panel.id);

  const cleanups = [];
  const on = (target, type, fn, options = false) => {
    target.addEventListener(type, fn, options);
    cleanups.push(() => target.removeEventListener(type, fn, options));
  };

  let category = ALL;
  const visible = () => cards.filter((card) => !card.hidden);
  const current = () => cards.find((card) => card.getAttribute("aria-current") === "true");

  const filter = () => {
    for (const chip of chips) chip.setAttribute("aria-pressed", String(chip.dataset.category === category));
    const terms = search.value.toLowerCase().split(/\s+/).filter(Boolean);
    for (const card of cards) {
      const inCategory = category === ALL || card.dataset.category === category;
      card.hidden = !inCategory || !terms.every((term) => card.dataset.search.includes(term));
    }
    const shown = visible().length;
    status.textContent =
      shown === 0 ? "No demos match." : shown === cards.length ? `${shown} demos` : `${shown} of ${cards.length} demos`;
    grid.scrollTop = 0;
  };

  // The panel hangs from the end of the demo row, where its button sits, and can
  // reach back as far as the playground's left edge. It runs down to the bottom
  // of the playground or of the window, whichever comes first.
  const fit = () => {
    const box = root.getBoundingClientRect();
    const anchor = strip.parentElement.getBoundingClientRect();
    const width = anchor.right - box.left - (box.right - anchor.right);
    const height = Math.min(box.bottom, window.innerHeight) - anchor.bottom - GAP;
    panel.style.setProperty("--fcp-picker-room-x", `${Math.max(width, 0)}px`);
    panel.style.setProperty("--fcp-picker-room-y", `${Math.max(height, MIN_HEIGHT)}px`);
  };

  // Focuses a card and scrolls the grid, not the page, to show it.
  const focus = (card, center = false) => {
    if (!card) return;
    card.focus({ preventScroll: true });
    const top = card.offsetTop - grid.offsetTop;
    const room = grid.clientHeight - card.offsetHeight;
    if (center) grid.scrollTop = top - room / 2;
    else if (top < grid.scrollTop) grid.scrollTop = top;
    else if (top > grid.scrollTop + room) grid.scrollTop = top - room;
  };

  const isOpen = () => !panel.hidden;

  // Each opening shows every demo, with the running one in focus.
  const open = () => {
    if (isOpen()) return;
    search.value = "";
    category = ALL;
    fit();
    panel.hidden = false;
    button.setAttribute("aria-expanded", "true");
    filter();
    focus(current() ?? visible()[0], true);
  };

  const close = (restoreFocus = false) => {
    if (!isOpen()) return;
    panel.hidden = true;
    button.setAttribute("aria-expanded", "false");
    if (restoreFocus) button.focus();
  };

  // Runs a demo the reader picked, unless it is the one running.
  const choose = (id) => {
    if (!id || id === current()?.dataset.demo) return;
    show(id);
    onPick(id);
  };

  const pick = (card) => {
    if (!card) return;
    close(true);
    choose(card.dataset.demo);
  };

  // A key that types a character. Space is left out, as it presses a card.
  // Moving the focus to the search box on such a key types the character there.
  const typed = (e) => e.key.length === 1 && e.key !== " " && !e.ctrlKey && !e.metaKey && !e.altKey;

  // Fades the ends of the row that hide more demos.
  const shade = () => {
    const start = strip.scrollLeft > 1;
    const end = strip.scrollLeft + strip.clientWidth < strip.scrollWidth - 1;
    strip.dataset.more = start && end ? "both" : start ? "start" : end ? "end" : "";
  };
  on(strip, "scroll", shade);
  // The row also changes width with the toolbar, as when the status text grows.
  const resize = new ResizeObserver(shade);
  resize.observe(strip);
  cleanups.push(() => resize.disconnect());

  on(strip, "click", (e) => choose(e.target.closest(".fcp-strip-item")?.dataset.demo));
  // The arrow keys, Home, and End move the focus along the row, and Enter or Space runs a demo.
  on(strip, "keydown", (e) => {
    const index = items.indexOf(document.activeElement);
    const to = { ArrowLeft: index - 1, ArrowRight: index + 1, Home: 0, End: items.length - 1 }[e.key];
    if (index < 0 || to === undefined) return;
    e.preventDefault();
    const item = items[Math.min(Math.max(to, 0), items.length - 1)];
    for (const other of items) other.tabIndex = other === item ? 0 : -1;
    item.focus({ preventScroll: true });
    reveal(item);
  });
  // A wheel turned over the row scrolls it sideways until it reaches an end.
  on(
    strip,
    "wheel",
    (e) => {
      if (e.ctrlKey || e.metaKey || Math.abs(e.deltaX) >= Math.abs(e.deltaY)) return;
      const max = strip.scrollWidth - strip.clientWidth;
      if ((e.deltaY < 0 && strip.scrollLeft <= 0) || (e.deltaY > 0 && strip.scrollLeft >= max - 1)) return;
      e.preventDefault();
      strip.scrollLeft += e.deltaMode === 1 ? e.deltaY * 16 : e.deltaY;
    },
    { passive: false },
  );

  on(button, "click", () => (isOpen() ? close(true) : open()));
  on(button, "keydown", (e) => {
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      open();
    } else if (!isOpen() && typed(e)) {
      open();
      search.focus();
    }
  });

  on(panel, "keydown", (e) => {
    if (e.key !== "Escape") return;
    e.preventDefault();
    e.stopPropagation();
    close(true);
  });
  on(panel.querySelector(".fcp-picker-close"), "click", () => close(true));
  for (const chip of chips) {
    on(chip, "click", () => {
      category = chip.dataset.category;
      filter();
    });
  }

  on(search, "input", filter);
  on(search, "keydown", (e) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      focus(visible()[0]);
    } else if (e.key === "Enter") {
      e.preventDefault();
      pick(visible()[0]);
    }
  });

  on(grid, "click", (e) => pick(e.target.closest(".fcp-picker-card")));
  // Arrow keys walk the grid, Home and End jump to its ends, and Up from the top row returns to the search.
  on(grid, "keydown", (e) => {
    const shown = visible();
    const index = shown.indexOf(document.activeElement);
    if (index < 0) return;
    const columns = Math.max(1, getComputedStyle(grid).gridTemplateColumns.split(" ").filter(Boolean).length);
    const to = {
      ArrowLeft: index - 1,
      ArrowRight: index + 1,
      ArrowUp: index - columns,
      ArrowDown: index + columns,
      Home: 0,
      End: shown.length - 1,
    }[e.key];
    if (to === undefined) {
      if (typed(e)) search.focus();
      return;
    }
    e.preventDefault();
    if (to < 0) search.focus();
    else focus(shown[Math.min(to, shown.length - 1)]);
  });

  // Focus that leaves the panel would land on what it covers.
  on(panel, "focusout", (e) => {
    const next = e.relatedTarget;
    if (next instanceof Node && (panel.contains(next) || button.contains(next))) return;
    if (next) close();
  });
  // A press outside closes the panel, also where a press does not move the focus, as on iOS.
  on(
    document,
    "pointerdown",
    (e) => {
      const path = e.composedPath();
      if (isOpen() && !path.includes(panel) && !path.includes(button)) close();
    },
    true,
  );
  on(window, "resize", () => isOpen() && fit());

  // A thumbnail fades in once it loads. One that fails leaves its card saying it has no preview.
  for (const image of panel.querySelectorAll(".fcp-picker-thumb img")) {
    const loaded = () => image.classList.add("fcp-picker-loaded");
    if (image.complete && image.naturalWidth) loaded();
    on(image, "load", loaded);
    on(image, "error", () => {
      image.parentElement?.classList.add("fcp-picker-missing");
      image.remove();
    });
  }

  return {
    show,
    unwire() {
      close();
      for (const fn of cleanups) fn();
    },
  };
}
