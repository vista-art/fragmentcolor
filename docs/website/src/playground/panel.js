// The look of the playground's picker panels: a search box, category chips,
// and a grid of cards with a thumbnail each. The first picker a page opens
// adds the styles, and every other picker shares them.

const STYLE_ID = "fcp-picker-style";

export function element(tag, className, attributes = {}) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  for (const [name, value] of Object.entries(attributes)) node.setAttribute(name, value);
  return node;
}

/** Adds the panel styles to the page, once. */
export function injectStyle() {
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
.fcp-picker-card[aria-pressed="true"],
.fcp-picker-card[aria-current="true"] {
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
.fcp-picker-thumb.fcp-picker-missing {
  display: grid;
  place-items: center;
}
.fcp-picker-thumb.fcp-picker-missing::before {
  content: "No preview";
  color: var(--fcp-picker-muted);
  font-size: 0.7rem;
}
.fcp-picker-card[data-clash] {
  border-style: dashed;
}
.fcp-picker-card[aria-pressed="true"] .fcp-picker-thumb::after,
.fcp-picker-card[aria-current="true"] .fcp-picker-thumb::after,
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
.fcp-picker-title {
  display: block;
  font-size: 0.8125rem;
  line-height: 1.3;
  color: var(--fcp-picker-strong);
}
.fcp-picker-title .fcp-picker-prefix {
  font-size: 0.6875rem;
}
/* The demo picker drops down from its button and fits the room below it.
   Its cards show wide thumbnails over the demo titles. */
.fcp-picker.fcp-demo-picker {
  top: calc(100% + 4px);
  left: 0;
  right: auto;
  margin: 0;
  width: min(var(--fcp-picker-width, 46rem), var(--fcp-picker-room-x, 100vw));
  max-height: min(36rem, var(--fcp-picker-room-y, 78vh));
}
.fcp-demo-picker .fcp-picker-grid {
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
}
.fcp-demo-picker .fcp-picker-thumb {
  aspect-ratio: 16 / 10;
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
  .fcp-demo-picker .fcp-picker-grid {
    grid-template-columns: repeat(auto-fill, minmax(124px, 1fr));
  }
}
@media (prefers-reduced-motion: reduce) {
  .fcp-picker-card,
  .fcp-picker-thumb img {
    transition: none;
  }
}
`;
