// Editor colors for each site theme. Dark is dracula. Light is white with the
// docs' light code colors, and a site theme switch reaches every open editor.

import { Compartment } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { HighlightStyle, defaultHighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { dracula } from "thememirror";

// The docs' light code colors, keyed by the color that CodeMirror's
// default style gives the same tags. That style's specs carry the tag
// objects, because @lezer/highlight is not a direct dependency of the site.
const LIGHT_SYNTAX = {
  "#708": "#8844ae", // keywords
  "#219": "#aa0982", // atoms and booleans
  "#164": "#aa0982", // numbers and other literals
  "#a11": "#984e4d", // strings
  "#e40": "#984e4d", // template strings, escapes, and regular expressions
  "#00f": "#3b61b0", // variable definitions
  "#30a": "#403f53", // local variables
  "#085": "#096e72", // type names
  "#167": "#096e72", // class names
  "#256": "#3b61b0", // macros
  "#00c": "#3b61b0", // property definitions
  "#940": "#5d6376", // comments
  "#f00": "#a31515", // invalid tokens
};

const lightSyntax = HighlightStyle.define(
  defaultHighlightStyle.specs.map((spec) =>
    spec.color in LIGHT_SYNTAX ? { ...spec, color: LIGHT_SYNTAX[spec.color] } : spec,
  ),
);

// The current line paints at z-index -10, under the selection layer
// at -1 or lower, so a selection shows on that line too. Each set of
// rules comes before its palette and wins where both set a property.
// The dark line color is dracula's own #44475a, taken at 60% opacity.
const THEMES = {
  dark: [
    EditorView.theme({
      ".cm-activeLine::before": { backgroundColor: "rgba(68, 71, 90, 0.6)" },
      ".cm-activeLineGutter": { backgroundColor: "rgba(68, 71, 90, 0.6)" },
      ".cm-selectionLayer .cm-selectionBackground": { background: "rgba(165, 180, 252, 0.24)" },
      "&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground": {
        background: "rgba(165, 180, 252, 0.32)",
      },
    }),
    dracula,
  ],
  light: [
    EditorView.theme({
      "&": { backgroundColor: "#ffffff", color: "#403f53" },
      ".cm-content": { caretColor: "#403f53" },
      ".cm-cursor, .cm-dropCursor": { borderLeftColor: "#403f53" },
      ".cm-gutters": { backgroundColor: "#ffffff", color: "#666c78" },
      ".cm-activeLine::before": { backgroundColor: "rgba(64, 63, 83, 0.06)" },
      ".cm-activeLineGutter": { backgroundColor: "rgba(64, 63, 83, 0.06)" },
      ".cm-selectionLayer .cm-selectionBackground": { background: "rgba(61, 80, 245, 0.1)" },
      "&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground": {
        background: "rgba(61, 80, 245, 0.12)",
      },
    }),
    syntaxHighlighting(lightSyntax),
  ],
};

// Starlight sets data-theme to "light" or "dark", resolving Auto first.
function readSiteTheme() {
  return document.documentElement.dataset.theme === "light" ? "light" : "dark";
}

/**
 * Holds the theme of one playground's editor states. A new state takes `of()`,
 * and `reconfigure()` is the effect that moves a state to the current site theme.
 */
export function editorTheme() {
  const compartment = new Compartment();
  return {
    of: () => compartment.of(THEMES[readSiteTheme()]),
    reconfigure: () => compartment.reconfigure(THEMES[readSiteTheme()]),
  };
}

/**
 * Calls `onChange` when the site turns light or dark.
 * The returned function stops watching the site theme.
 */
export function watchSiteTheme(onChange) {
  let current = readSiteTheme();
  const observer = new MutationObserver(() => {
    if (readSiteTheme() === current) return;
    current = readSiteTheme();
    onChange(current);
  });
  observer.observe(document.documentElement, { attributeFilter: ["data-theme"] });
  return () => observer.disconnect();
}
