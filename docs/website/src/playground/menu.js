// The demo picker names the running demo, and its button opens a panel of
// demo cards like the shader library's: a thumbnail per demo, category chips,
// and a search box. The panel is a dialog that keeps the page usable. The
// arrow keys walk the cards, typed letters go to the search, Enter picks a
// demo, and Escape closes the panel.

import { injectStyle } from "./panel.js";

const ALL = "";
// The panel keeps this much room under it, and never gets shorter than MIN_HEIGHT.
const GAP = 8;
const MIN_HEIGHT = 240;

let pickers = 0;

/**
 * @param {HTMLElement} root the <fc-playground> element
 * @param {{ id: string, title: string, summary: string }[]} demos the demos the panel offers
 * @param {(id: string) => void} onPick called with the demo the reader picks
 * @returns {{ show(id: string): void, unwire(): void }} `show` names a demo without calling `onPick`
 */
export function wireMenu(root, demos, onPick) {
  const button = root.querySelector("button.fcp-menu-button");
  const panel = root.querySelector(".fcp-demo-picker");
  const value = root.querySelector(".fcp-menu-value");
  const summary = root.querySelector(".fcp-summary");
  const cards = [...(panel?.querySelectorAll(".fcp-picker-card") ?? [])];

  const show = (id) => {
    const demo = demos.find((d) => d.id === id);
    if (!demo) return;
    for (const card of cards) {
      if (card.dataset.demo === id) card.setAttribute("aria-current", "true");
      else card.removeAttribute("aria-current");
    }
    if (value) value.textContent = demo.title;
    if (summary) summary.textContent = demo.summary;
  };
  if (!button || !panel) return { show, unwire() {} };

  injectStyle();
  const search = panel.querySelector(".fcp-picker-search");
  const chips = [...panel.querySelectorAll(".fcp-picker-chip")];
  const status = panel.querySelector(".fcp-picker-status");
  const grid = panel.querySelector(".fcp-picker-grid");
  panel.id = `fcp-demos-${++pickers}`;
  button.setAttribute("aria-controls", panel.id);

  const cleanups = [];
  const on = (target, type, fn, capture = false) => {
    target.addEventListener(type, fn, capture);
    cleanups.push(() => target.removeEventListener(type, fn, capture));
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

  // The panel spans from its button to the playground's right edge, and down to
  // the bottom of the playground or of the window, whichever comes first.
  const fit = () => {
    const box = root.getBoundingClientRect();
    const anchor = button.getBoundingClientRect();
    const width = box.right - anchor.left - (anchor.left - box.left);
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

  const pick = (card) => {
    const id = card?.dataset.demo;
    if (!id) return;
    const changed = card !== current();
    close(true);
    if (!changed) return;
    show(id);
    onPick(id);
  };

  // A key that types a character. Space is left out, as it presses a card.
  // Moving the focus to the search box on such a key types the character there.
  const typed = (e) => e.key.length === 1 && e.key !== " " && !e.ctrlKey && !e.metaKey && !e.altKey;

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
