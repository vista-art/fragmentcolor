// The demo menu names the running demo and opens the list of demos.
// It follows the WAI-ARIA select-only combobox, so focus stays on the button
// while the arrow keys, Home, End, Page Up, Page Down, and typed letters move
// through the list. Enter, Space, and Tab pick a demo, and Escape closes the list.

// Letters typed within this many milliseconds of each other build one search.
const SEARCH_DELAY = 500;
// Page Up and Page Down move this many demos.
const PAGE = 10;

let menus = 0;

/**
 * @param {HTMLElement} root the <fc-playground> element
 * @param {{ id: string, title: string, summary: string }[]} demos the demos the list offers, in its order
 * @param {(id: string) => void} onPick called with the demo the reader picks
 * @returns {{ show(id: string): void, unwire(): void }} `show` names a demo without calling `onPick`
 */
export function wireMenu(root, demos, onPick) {
  const button = root.querySelector(".fcp-menu-button");
  const list = root.querySelector(".fcp-menu-list");
  const value = root.querySelector(".fcp-menu-value");
  const summary = root.querySelector(".fcp-summary");
  if (!button || !list || !value) return { show() {}, unwire() {} };
  const options = [...list.querySelectorAll('[role="option"]')];
  const uid = `fcp-menu-${++menus}`;
  list.id = `${uid}-list`;
  options.forEach((option, index) => (option.id = `${uid}-${index}`));
  button.setAttribute("aria-controls", list.id);

  const cleanups = [];
  const on = (target, type, fn, capture = false) => {
    target.addEventListener(type, fn, capture);
    cleanups.push(() => target.removeEventListener(type, fn, capture));
  };

  // The option with visual focus while the list is open, and the letters typed so far.
  let active = -1;
  let search = "";
  let timer = 0;

  const current = () => options.findIndex((option) => option.getAttribute("aria-selected") === "true");
  const optionAt = (e) => options.indexOf(e.target.closest('[role="option"]'));

  const show = (id) => {
    const demo = demos.find((d) => d.id === id);
    if (!demo) return;
    for (const option of options) option.setAttribute("aria-selected", String(option.dataset.demo === id));
    value.textContent = demo.title;
    if (summary) summary.textContent = demo.summary;
  };

  // Moves visual focus to `index` and scrolls the list just enough to show it.
  const focus = (index, scroll = true) => {
    active = Math.max(0, Math.min(options.length - 1, index));
    const option = options[active];
    for (const other of options) other.classList.toggle("active", other === option);
    button.setAttribute("aria-activedescendant", option.id);
    if (!scroll) return;
    const pad = parseFloat(getComputedStyle(list).paddingTop) || 0;
    const top = option.offsetTop - pad;
    const bottom = option.offsetTop + option.offsetHeight + pad;
    if (top < list.scrollTop) list.scrollTop = top;
    else if (bottom > list.scrollTop + list.clientHeight) list.scrollTop = bottom - list.clientHeight;
  };

  // The list opens scrolled to the top, or as little past it as `index` needs.
  const open = (index = current()) => {
    list.hidden = false;
    list.scrollTop = 0;
    button.setAttribute("aria-expanded", "true");
    focus(index);
  };

  const close = () => {
    list.hidden = true;
    button.setAttribute("aria-expanded", "false");
    button.removeAttribute("aria-activedescendant");
    for (const option of options) option.classList.remove("active");
    active = -1;
    search = "";
  };

  // Closes the list, and picks the option at `index` when it names another demo.
  const pick = (index) => {
    const id = options[index]?.dataset.demo;
    close();
    if (!id || index === current()) return;
    show(id);
    onPick(id);
  };

  // Focuses the next title that starts with the letters typed so far.
  // Typing one letter again and again steps through the titles that start with it.
  const find = (key) => {
    clearTimeout(timer);
    timer = setTimeout(() => (search = ""), SEARCH_DELAY);
    search += key.toLowerCase();
    const repeated = [...search].every((letter) => letter === search[0]);
    const start = repeated ? active + 1 : active;
    const ordered = [...options.slice(start), ...options.slice(0, start)];
    const starts = (prefix) => ordered.find((o) => o.textContent.trim().toLowerCase().startsWith(prefix));
    const match = starts(search) ?? (repeated ? starts(search[0]) : undefined);
    if (match) focus(options.indexOf(match));
  };

  on(button, "keydown", (e) => {
    if (e.ctrlKey || e.metaKey) return;
    const last = options.length - 1;
    // Space is a letter while a search is under way, as titles have spaces too.
    const typed = e.key.length === 1 && (e.key !== " " || search !== "");
    if (list.hidden) {
      const from = { ArrowDown: current(), ArrowUp: current(), Enter: current(), " ": current(), Home: 0, End: last }[e.key];
      if (from === undefined && !typed) return;
      e.preventDefault();
      open(from);
      if (typed) find(e.key);
      return;
    }
    const to = {
      ArrowDown: active + 1,
      ArrowUp: active - 1,
      PageDown: active + PAGE,
      PageUp: active - PAGE,
      Home: 0,
      End: last,
    }[e.key];
    if (e.key === "Tab") {
      // Tab picks the focused demo and lets the focus move on.
      pick(active);
    } else if (e.key === "Escape") {
      e.preventDefault();
      close();
    } else if (e.key === "Enter" || (e.key === " " && !typed) || (e.altKey && e.key === "ArrowUp")) {
      e.preventDefault();
      pick(active);
    } else if (to !== undefined) {
      e.preventDefault();
      focus(to);
    } else if (typed) {
      e.preventDefault();
      find(e.key);
    }
  });
  on(button, "click", () => (list.hidden ? open() : close()));
  on(button, "blur", close);
  // Presses on the list leave the focus on the button.
  on(list, "mousedown", (e) => e.preventDefault());
  on(list, "pointermove", (e) => {
    const index = optionAt(e);
    if (index >= 0 && index !== active) focus(index, false);
  });
  on(list, "click", (e) => {
    const index = optionAt(e);
    if (index >= 0) pick(index);
  });
  // A tap outside closes the list where a tap does not move the focus, as on iOS.
  on(
    document,
    "pointerdown",
    (e) => {
      if (!list.hidden && !button.contains(e.target) && !list.contains(e.target)) close();
    },
    true,
  );

  return {
    show,
    unwire() {
      for (const fn of cleanups) fn();
      clearTimeout(timer);
      close();
    },
  };
}
