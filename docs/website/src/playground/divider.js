// The divider between the canvas and the code. A drag moves it, and so do
// the arrow keys, Home, and End. A double click puts the default split back.
// The split the viewer leaves returns on every playground, the landing page included.

// The canvas pane's share of the width, in percent.
// Playground.astro applies it in an inline script, before the first paint.
export const SPLIT_KEY = "fc-playground-split";
// An arrow key moves the divider by this share of the width, in percent.
const STEP = 2;

function store(value) {
  try {
    if (value === null) localStorage.removeItem(SPLIT_KEY);
    else localStorage.setItem(SPLIT_KEY, String(value));
  } catch {
    // with storage blocked, the split lasts until the page reloads
  }
}

/**
 * @param {HTMLElement} root the <fc-playground> element
 * @returns {() => void} removes the listeners
 */
export function wireDivider(root) {
  const body = root.querySelector(".fcp-body");
  const stage = root.querySelector(".fcp-stage");
  const handle = root.querySelector(".fcp-divider");
  if (!body || !stage || !handle) return () => {};
  const cleanups = [];
  const on = (target, type, fn) => {
    target.addEventListener(type, fn);
    cleanups.push(() => target.removeEventListener(type, fn));
  };
  const style = getComputedStyle(body);

  // The canvas share and its limits at the current width. The minimum widths
  // come from the stylesheet, whose clamp holds them when the window resizes.
  const measure = () => {
    const width = body.getBoundingClientRect().width || 1;
    const min = ((parseFloat(style.getPropertyValue("--fcp-stage-min")) || 0) / width) * 100;
    const max = 100 - ((parseFloat(style.getPropertyValue("--fcp-code-min")) || 0) / width) * 100;
    const share = (stage.getBoundingClientRect().width / width) * 100;
    return { min, max: Math.max(min, max), share };
  };

  const show = () => {
    const { min, max, share } = measure();
    const now = Math.round(share);
    handle.setAttribute("aria-valuemin", String(Math.round(min)));
    handle.setAttribute("aria-valuemax", String(Math.round(max)));
    handle.setAttribute("aria-valuenow", String(now));
    handle.setAttribute("aria-valuetext", `Canvas ${now}%, code ${100 - now}%`);
  };

  // Moves the divider to `share` percent, within the limits, and returns where it landed.
  const move = (share) => {
    const { min, max } = measure();
    const value = Math.round(Math.min(max, Math.max(min, share)) * 100) / 100;
    body.style.setProperty("--fcp-split", `${value}%`);
    show();
    return value;
  };

  // A drag follows the pointer that started it. Other presses wait while the
  // divider holds that pointer, and the panes ignore the pointer only during the capture.
  let drag = null;
  const owns = (e) => drag?.id === e.pointerId;
  on(handle, "pointerdown", (e) => {
    if (e.button !== 0 || (drag && handle.hasPointerCapture(drag.id))) return;
    // No text selection and no editor focus; the divider takes the focus instead.
    e.preventDefault();
    handle.focus({ preventScroll: true });
    const rect = body.getBoundingClientRect();
    // The line keeps its distance from the pointer instead of jumping under it.
    const offset = e.clientX - stage.getBoundingClientRect().right;
    drag = { id: e.pointerId, left: rect.left + offset, width: rect.width, value: null };
    handle.setPointerCapture(e.pointerId);
  });
  on(handle, "gotpointercapture", (e) => {
    if (owns(e)) root.classList.add("fcp-resizing");
  });
  on(handle, "pointermove", (e) => {
    if (!owns(e) || !handle.hasPointerCapture(e.pointerId)) return;
    drag.value = move(((e.clientX - drag.left) / drag.width) * 100);
  });
  // A release, a cancel, or a lost capture ends the drag, whichever comes first.
  const end = (e) => {
    if (!owns(e)) return;
    if (drag.value != null) store(drag.value);
    drag = null;
    root.classList.remove("fcp-resizing");
  };
  for (const type of ["pointerup", "pointercancel", "lostpointercapture"]) on(handle, type, end);
  on(handle, "dblclick", () => {
    body.style.removeProperty("--fcp-split");
    store(null);
    show();
  });
  on(handle, "keydown", (e) => {
    const { min, max, share } = measure();
    const target = { ArrowLeft: share - STEP, ArrowRight: share + STEP, Home: min, End: max }[e.key];
    if (target === undefined) return;
    e.preventDefault();
    store(move(target));
  });

  if (typeof ResizeObserver !== "undefined") {
    const observer = new ResizeObserver(show);
    observer.observe(body);
    cleanups.push(() => observer.disconnect());
  }
  show();
  return () => {
    for (const fn of cleanups) fn();
    root.classList.remove("fcp-resizing");
  };
}
