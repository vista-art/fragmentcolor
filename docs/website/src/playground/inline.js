// Inline controls for the playground editors. A slider follows the number
// under the cursor and a swatch sits before each color literal, so a value
// can be dragged or picked in place, the way live-coding tools do it.

import { Annotation, StateEffect } from "@codemirror/state";
import { Decoration, EditorView, ViewPlugin, WidgetType } from "@codemirror/view";
import { syntaxTree } from "@codemirror/language";

/** Marks a transaction made by an inline slider or swatch. */
export const scrubbed = Annotation.define();

/** Recomputes the controls, for when rangeFor or isColor would answer differently. */
export const refreshInline = StateEffect.define();

// Lines longer than this (minified code, data) get no controls.
const MAX_LINE = 2000;

// A "-" right after one of these is a sign, and a "/" starts a regex.
const SIGN_AFTER = new Set("(,[{=:?;!&|^~<>+-*/%");
const SIGN_WORDS = new Set([
  "return", "case", "else", "if", "while", "switch", "do", "default", "yield", "await",
  "typeof", "void", "delete", "new", "throw", "in", "of", "instanceof", "extends",
]);

const JS_NUMBER = /(?:\d[\d_]*(?:\.[\d_]*)?|\.\d[\d_]*)(?:[eE][+-]?\d[\d_]*)?/y;
const WGSL_NUMBER = /(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?[fhiu]?/y;
const PREFIXED = /0(?:[xX][\da-fA-F_.]*(?:[pP][+-]?\d+)?|[bBoO])\w*/y;
const WORD = /[A-Za-z_$][\w$]*/y;
const LITERAL = /^(-?)(\d[\d_]*(?:\.[\d_]*)?|\.\d[\d_]*)([eE][+-]?\d[\d_]*)?([fhiu]?)$/;
const VECTOR = /^vec([34])([fh]?)$/;
const NAMED_COLOR = /color|tint/i;

const isDigit = (c) => c >= "0" && c <= "9";
const isWordChar = (c) => c !== undefined && /[\w$]/.test(c);
const isPunct = (token, text) => token?.type === "punct" && token.text === text;

function closeComment(text, from, nested) {
  let depth = 1;
  for (let i = from; i < text.length; i++) {
    if (text[i] === "*" && text[i + 1] === "/") {
      if (--depth === 0) return i + 2;
      i++;
    } else if (nested && text[i] === "/" && text[i + 1] === "*") {
      depth++;
      i++;
    }
  }
  return text.length;
}

function closeQuote(text, from, quote) {
  for (let i = from; i < text.length; i++) {
    if (text[i] === "\\") i++;
    else if (text[i] === quote) return i + 1;
  }
  return text.length;
}

function closeRegex(text, from) {
  let inClass = false;
  for (let i = from; i < text.length; i++) {
    const c = text[i];
    if (c === "\\") i++;
    else if (c === "[") inClass = true;
    else if (c === "]") inClass = false;
    else if (c === "/" && !inClass) {
      let end = i + 1;
      while (/[a-z]/i.test(text[end] ?? "")) end++;
      return end;
    }
  }
  return -1;
}

// True when a "-" or "/" after code[index] starts an operand.
function opensOperand(code, index) {
  const token = code[index];
  if (!token) return true;
  if (token.type === "word") return SIGN_WORDS.has(token.text);
  return token.type === "punct" && SIGN_AFTER.has(token.text) && !endsPostfix(code, index);
}

// The second sign of a++ or b-- closes its operand.
function endsPostfix(code, index) {
  const [operand, first, second] = [code[index - 2], code[index - 1], code[index]];
  if (second.text !== "+" && second.text !== "-") return false;
  if (!isPunct(first, second.text) || first.to !== second.from) return false;
  return isPunct(operand, ")") || isPunct(operand, "]") || (operand?.type === "word" && !SIGN_WORDS.has(operand.text));
}

function parseLiteral(text) {
  const match = LITERAL.exec(text);
  if (!match) return null;
  const [, sign, mantissa, exponent = "", suffix] = match;
  const value = Number(`${sign}${mantissa}${exponent}`.replace(/_/g, ""));
  if (!Number.isFinite(value)) return null;
  const integer = suffix === "i" || suffix === "u" || (!suffix && !mantissa.includes(".") && !exponent);
  return { mantissa, suffix, integer, value };
}

/**
 * Splits one line into tokens: number, word, punct, string, comment, regex,
 * and other (hex and malformed numbers). Whitespace is dropped. Comments and
 * strings that stay open run to the end of the line. A number is `unsigned`
 * where a minus in front of it would read as a binary operator.
 * @param {string} text
 * @param {"wgsl" | "js"} language
 */
export function tokenize(text, language) {
  const js = language === "js";
  const tokens = [];
  const code = []; // the tokens that are not comments
  const outer = []; // brace depth around each open ${ of a template literal
  let depth = 0;

  const add = (token) => {
    tokens.push(token);
    if (token.type !== "comment") code.push(token);
  };
  const plain = (type, from, to) => add({ type, from, to, text: text.slice(from, to) });

  // Template text from `start` up to the closing backtick or the next ${.
  const template = (from, start) => {
    for (let i = from; i < text.length; i++) {
      if (text[i] === "\\") i++;
      else if (text[i] === "`") {
        plain("string", start, i + 1);
        return i + 1;
      } else if (text[i] === "$" && text[i + 1] === "{") {
        plain("string", start, i + 1);
        plain("punct", i + 1, i + 2);
        outer.push(depth);
        depth = 0;
        return i + 2;
      }
    }
    plain("string", start, text.length);
    return text.length;
  };

  const number = (start) => {
    const prefixed = text[start] === "0" && /[xXbBoO]/.test(text[start + 1] ?? "");
    const pattern = prefixed ? PREFIXED : js ? JS_NUMBER : WGSL_NUMBER;
    pattern.lastIndex = start;
    let end = start + pattern.exec(text)[0].length;
    if (prefixed || isWordChar(text[end])) {
      // hex, BigInt, or digits glued to letters
      while (isWordChar(text[end])) end++;
      plain("other", start, end);
      return end;
    }
    let from = start;
    let unsigned = false;
    const previous = code.at(-1);
    const glued = previous?.type === "punct" && previous.to === start ? previous : null;
    if (glued?.text === "-" && opensOperand(code, code.length - 2)) {
      tokens.pop();
      code.pop();
      from = glued.from;
    } else {
      // a sign here would read as "a--1", "a+-1" or "if x -1"
      unsigned = glued?.text === "-" || glued?.text === "+" || !opensOperand(code, code.length - 1);
    }
    const literal = parseLiteral(text.slice(from, end));
    if (!literal) {
      plain("other", from, end);
      return end;
    }
    add({
      type: "number",
      from,
      to: end,
      text: text.slice(from, end),
      value: literal.value,
      integer: literal.integer,
      suffix: literal.suffix,
      unsigned: unsigned || literal.suffix === "u",
    });
    return end;
  };

  let i = 0;
  while (i < text.length) {
    const c = text[i];
    const next = text[i + 1];
    if (c === " " || c === "\t" || c === "\r" || c === "\n") {
      i++;
    } else if (c === "/" && next === "/") {
      plain("comment", i, text.length);
      break;
    } else if (c === "/" && next === "*") {
      const end = closeComment(text, i + 2, !js);
      plain("comment", i, end);
      i = end;
    } else if (js && (c === '"' || c === "'")) {
      const end = closeQuote(text, i + 1, c);
      plain("string", i, end);
      i = end;
    } else if (js && c === "`") {
      i = template(i + 1, i);
    } else if (js && c === "}" && outer.length && depth === 0) {
      depth = outer.pop();
      i = template(i + 1, i);
    } else if (js && c === "/" && opensOperand(code, code.length - 1) && closeRegex(text, i + 1) > 0) {
      const end = closeRegex(text, i + 1);
      plain("regex", i, end);
      i = end;
    } else if (isDigit(c) || (c === "." && isDigit(next) && !isWordChar(text[i - 1]))) {
      i = number(i);
    } else if (/[A-Za-z_$]/.test(c)) {
      WORD.lastIndex = i;
      const end = i + WORD.exec(text)[0].length;
      plain("word", i, end);
      i = end;
    } else {
      if (js && outer.length) {
        if (c === "{") depth++;
        else if (c === "}") depth--;
      }
      plain("punct", i, i + 1);
      i++;
    }
  }
  return tokens;
}

/**
 * Every scrubbable number on a line: decimal literals outside comments and
 * strings, with a leading minus when it is a sign. Hex literals are left out.
 * @param {string} text
 * @param {"wgsl" | "js"} language
 * @returns {{from: number, to: number, text: string, value: number, integer: boolean, suffix: string, unsigned: boolean}[]}
 */
export function findNumbers(text, language) {
  return tokenize(text, language).filter((token) => token.type === "number");
}

/**
 * The number that contains or touches `column` on a line, or null.
 * @param {string} text
 * @param {number} column
 * @param {"wgsl" | "js"} language
 */
export function findNumber(text, column, language) {
  for (const token of tokenize(text, language)) {
    if (token.from > column) break;
    if (token.type === "number" && column <= token.to) return token;
  }
  return null;
}

/**
 * Rewrites a numeric literal with a new value in the same style. Integers
 * stay integers. Floats keep a decimal point and up to `decimals` places.
 * The sign follows the value and the suffix stays.
 * @param {string} original
 * @param {number} value
 * @param {number} [decimals]
 */
export function formatNumber(original, value, decimals = 3) {
  const literal = parseLiteral(original);
  if (!literal || !Number.isFinite(value)) return original;
  if (literal.integer) {
    let whole = Math.round(value);
    if (literal.suffix === "u") whole = Math.max(0, whole);
    if (Math.abs(whole) >= 1e21) return original;
    let digits = String(Math.abs(whole));
    if (literal.mantissa.includes("_")) digits = digits.replace(/\B(?=(\d{3})+$)/g, "_");
    return `${whole < 0 ? "-" : ""}${digits}${literal.suffix}`;
  }
  const rounded = Number(value.toFixed(decimals));
  if (Math.abs(rounded) >= 1e21) return original;
  let digits = Math.abs(rounded).toFixed(decimals).replace(/0+$/, "").replace(/\.$/, ".0");
  if (literal.mantissa.startsWith(".")) digits = digits.replace(/^0\./, ".");
  return `${rounded < 0 ? "-" : ""}${digits}${literal.suffix}`;
}

/**
 * Rewrites one color component, a value in [0, 1] with up to 3 decimals.
 * WGSL components and JS components written with a point keep a point.
 * @param {string} original
 * @param {number} value
 * @param {"wgsl" | "js"} language
 */
export function formatComponent(original, value, language) {
  const literal = parseLiteral(original);
  const clamped = Math.min(1, Math.max(0, Number(value.toFixed(3))));
  let digits = clamped.toFixed(3).replace(/0+$/, "").replace(/\.$/, "");
  const float = language === "wgsl" || /[.eE]/.test(original);
  if (float && !digits.includes(".")) digits += ".0";
  if (literal?.mantissa.startsWith(".")) digits = digits.replace(/^0\./, ".");
  return `${digits}${literal?.suffix ?? ""}`;
}

/**
 * The slider range for a literal with no range from the host. Floats span
 * one unit, or their own size, either way. Integers reach 10 down and up.
 * @param {{value: number, integer: boolean, unsigned?: boolean}} literal
 * @returns {{min: number, max: number, step: number}}
 */
export function suggestRange({ value, integer, unsigned = false }) {
  let min;
  let max;
  let step;
  if (integer) {
    min = Math.min(0, value - 10);
    max = value + Math.max(10, Math.abs(value));
    step = 1;
  } else {
    const span = Math.max(Math.abs(value), 1);
    min = value - span;
    max = value + span;
    step = 10 ** Math.round(Math.log10(span / 100));
  }
  if (unsigned) min = Math.max(0, min);
  const tidy = (n) => Number(n.toPrecision(12));
  return { min: tidy(min), max: tidy(max), step: tidy(step) };
}

/**
 * A hex color for the first three components, read as sRGB in [0, 1].
 * @param {number[]} rgb
 */
export function toHex(rgb) {
  const byte = (c) => Math.round(Math.min(1, Math.max(0, c)) * 255);
  return `#${rgb
    .slice(0, 3)
    .map((c) => byte(c).toString(16).padStart(2, "0"))
    .join("")}`;
}

/**
 * The [r, g, b] components in [0, 1] of a "#rrggbb" color, or null.
 * @param {string} hex
 */
export function fromHex(hex) {
  const match = /^#?([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(hex);
  return match ? match.slice(1).map((h) => parseInt(h, 16) / 255) : null;
}

// Numbers separated by commas up to `close`, with an optional trailing comma.
function readList(tokens, start, close) {
  const items = [];
  for (let i = start; tokens[i]?.type === "number"; ) {
    items.push(tokens[i]);
    const separator = tokens[i + 1];
    if (isPunct(separator, close)) return { items, end: separator };
    if (!isPunct(separator, ",")) return null;
    i += 2;
    if (isPunct(tokens[i], close)) return { items, end: tokens[i] };
  }
  return null;
}

const inUnit = (token) => token.value >= 0 && token.value <= 1 && token.suffix !== "i" && token.suffix !== "u";

// vec3<f32>(r, g, b), vec3f(...), vec3(...) and the vec4 forms.
function vectorAt(tokens, index) {
  const head = tokens[index];
  const match = head.type === "word" && VECTOR.exec(head.text);
  if (!match) return null;
  let typed = Boolean(match[2]);
  let open = index + 1;
  if (!typed && isPunct(tokens[open], "<")) {
    const type = tokens[open + 1];
    if (type?.type !== "word" || !/^f(?:32|16)$/.test(type.text) || !isPunct(tokens[open + 2], ">")) return null;
    typed = true;
    open += 3;
  }
  if (!isPunct(tokens[open], "(")) return null;
  const list = readList(tokens, open + 1, ")");
  if (!list || list.items.length !== Number(match[1]) || !list.items.every(inUnit)) return null;
  // vec3(1, 0, 0) is an integer vector; floats would change its type
  if (!typed && list.items.every((item) => item.integer)) return null;
  return { kind: "vector", from: head.from, to: list.end.to, components: list.items };
}

const unquote = (token) => (token?.type === "string" && token.text.length > 1 ? token.text.slice(1, -1) : null);

// [r, g, b] or [r, g, b, a], with the .set("key", ...) key or the method it is passed to.
function arrayAt(tokens, index) {
  const open = tokens[index];
  if (!isPunct(open, "[")) return null;
  const before = tokens[index - 1];
  const indexes = before && (before.type === "number" || before.type === "string" || isPunct(before, ")") || isPunct(before, "]"));
  if (indexes || (before?.type === "word" && !SIGN_WORDS.has(before.text))) return null;
  const list = readList(tokens, index + 1, "]");
  if (!list || list.items.length < 3 || list.items.length > 4 || !list.items.every(inUnit)) return null;
  const color = { kind: "array", from: open.from, to: list.end.to, components: list.items, key: null, call: null };
  const back = (count) => tokens[index - count];
  if (isPunct(back(1), ",") && isPunct(back(3), "(") && back(4)?.text === "set") color.key = unquote(back(2));
  if (isPunct(back(1), "(") && back(2)?.type === "word") color.call = back(2).text;
  return color;
}

/**
 * Color literals on a line. WGSL: vec3 and vec4 constructors whose arguments
 * are all numbers in [0, 1]. JS: arrays of three or four such numbers, with
 * the `.set()` key or the method name they are passed to.
 * @param {string} text
 * @param {"wgsl" | "js"} language
 */
export function findColors(text, language) {
  const tokens = tokenize(text, language);
  const colors = [];
  for (let i = 0; i < tokens.length; i++) {
    const color = language === "js" ? arrayAt(tokens, i) : vectorAt(tokens, i);
    if (color) colors.push({ ...color, hex: toHex(color.components.map((c) => c.value)) });
  }
  return colors;
}

// A JS array is a color when it sets a key or calls a method named for one.
function namedColor(color) {
  return NAMED_COLOR.test(color.key ?? "") || NAMED_COLOR.test(color.call ?? "");
}

// The language's own tree has the final say on comments and strings that span lines.
function insideCode(state, pos) {
  for (let node = syntaxTree(state).resolveInner(pos, 1); node; node = node.parent) {
    if (node.name === "Interpolation") return true;
    if (/comment|string|regexp/i.test(node.name)) return false;
  }
  return true;
}

// Host callbacks run inside a view update, where a throw would disable the plugin.
function ask(callback, ...args) {
  try {
    return callback(...args);
  } catch (err) {
    console.warn("[inline] host callback failed", err);
    return undefined;
  }
}

function validRange(range) {
  if (!range) return null;
  const { min, max, step } = range;
  const ok = [min, max, step].every(Number.isFinite) && max > min && step > 0;
  return ok ? { min, max, step } : null;
}

const placesOf = (step) => (step < 1 ? Math.ceil(-Math.log10(step) - 1e-9) : 0);

class Slider extends WidgetType {
  constructor(session, value, controls) {
    super();
    this.session = session;
    this.value = value;
    this.controls = controls;
  }

  // A session lives as long as its literal keeps its start.
  eq(other) {
    return other.session === this.session;
  }

  // A widget right after changed text gets redrawn even when eq() holds, so
  // the session hands back the same element and CodeMirror leaves it in place.
  toDOM(view) {
    this.session.dom ??= this.build(view);
    return this.session.dom;
  }

  build(view) {
    const { controls, session } = this;
    const { min, max, step } = session;
    const dom = document.createElement("span");
    dom.className = "cm-inline-slider";
    const input = document.createElement("input");
    input.type = "range";
    input.min = String(min);
    input.max = String(max);
    input.step = String(step);
    input.value = String(this.value);
    input.tabIndex = -1;
    input.title = "Drag to change this number";
    input.setAttribute("aria-label", "Number");
    let moved = false; // a scrub happened since the last settle
    const settle = () => {
      if (!moved) return;
      moved = false;
      controls.settle(view);
    };
    input.addEventListener("input", () => {
      moved = true;
      controls.scrub(view, dom, Number(input.value), session);
    });
    input.addEventListener("change", settle);
    // a press hands the keyboard back to the editor on release, moved or not
    input.addEventListener("pointerdown", () => {
      const page = input.ownerDocument;
      const release = () => {
        page.removeEventListener("pointerup", release, true);
        page.removeEventListener("pointercancel", release, true);
        settle();
        controls.refocus(view, input);
      };
      page.addEventListener("pointerup", release, true);
      page.addEventListener("pointercancel", release, true);
    });
    input.addEventListener("keydown", (event) => {
      if (event.key !== "Escape" && event.key !== "Enter") return;
      event.preventDefault();
      view.focus();
    });
    dom.append(input);
    return dom;
  }

  ignoreEvent() {
    return true;
  }
}

class Swatch extends WidgetType {
  constructor(from, hex, controls) {
    super();
    this.from = from;
    this.hex = hex;
    this.controls = controls;
  }

  eq(other) {
    return other.from === this.from;
  }

  toDOM(view) {
    const { controls } = this;
    const dom = document.createElement("span");
    dom.className = "cm-inline-swatch";
    const input = document.createElement("input");
    input.type = "color";
    input.value = this.hex;
    input.tabIndex = -1;
    input.title = "Pick a color";
    input.setAttribute("aria-label", "Color");
    input.addEventListener("input", () => controls.pick(view, dom, input.value));
    input.addEventListener("change", () => {
      controls.settle(view);
      controls.refocus(view, input);
    });
    dom.append(input);
    return dom;
  }

  ignoreEvent() {
    return true;
  }
}

class Controls {
  constructor(view, config) {
    this.config = config;
    this.session = null;
    this.destroyed = false;
    this.swatches = this.findSwatches(view);
    this.slider = this.findSlider(view, null);
    this.decorations = this.compose();
  }

  update(update) {
    const refresh = update.transactions.some((tr) => tr.effects.some((effect) => effect.is(refreshInline)));
    const parsed = syntaxTree(update.startState) !== syntaxTree(update.state);
    const changed = update.docChanged || refresh || parsed;
    if (changed || update.viewportChanged) this.swatches = this.findSwatches(update.view);
    if (changed || update.selectionSet) this.slider = this.findSlider(update.view, update);
    this.decorations = this.compose();
    if (update.docChanged) {
      update.view.requestMeasure({ key: this, read: () => null, write: () => this.sync(update.view) });
    }
  }

  compose() {
    return Decoration.set(this.slider ? [...this.swatches, this.slider] : this.swatches, true);
  }

  enabled(view) {
    return !view.state.readOnly && view.state.facet(EditorView.editable);
  }

  findSwatches(view) {
    if (!this.enabled(view)) return [];
    const { state } = view;
    const { language, isColor } = this.config;
    const found = [];
    let last = 0;
    for (const { from, to } of view.visibleRanges) {
      for (let pos = from; pos <= to; ) {
        const line = state.doc.lineAt(pos);
        pos = line.to + 1;
        if (line.number <= last || line.length > MAX_LINE) continue;
        last = line.number;
        for (const color of findColors(line.text, language)) {
          const start = line.from + color.from;
          const end = line.from + color.to;
          if (!insideCode(state, start)) continue;
          if (language === "js" && !(isColor ? ask(isColor, state, start, end) : namedColor(color))) continue;
          const widget = new Swatch(start, color.hex, this);
          found.push(Decoration.widget({ widget, side: -1 }).range(start));
        }
      }
    }
    return found;
  }

  findSlider(view, update) {
    const { state } = view;
    const cursor = state.selection.main;
    if (!this.enabled(view) || !cursor.empty || view.composing) return this.drop();
    const line = state.doc.lineAt(cursor.head);
    if (line.length > MAX_LINE) return this.drop();
    const literal = findNumber(line.text, cursor.head - line.from, this.config.language);
    if (!literal || !insideCode(state, line.from + literal.from)) return this.drop();
    const from = line.from + literal.from;
    const to = line.from + literal.to;
    if (this.keeps(update, from)) Object.assign(this.session, { from, to });
    else this.session = this.open(state, literal, from, to);
    const widget = new Slider(this.session, literal.value, this);
    return Decoration.widget({ widget, side: 1 }).range(to);
  }

  // Scrubs and edits away from the literal keep the session, and with it the
  // range and the DOM. An edit that touches the literal starts over.
  keeps(update, from) {
    if (!this.session || !update) return false;
    let { from: start, to: end } = this.session;
    for (const tr of update.transactions) {
      if (!tr.docChanged) continue;
      if (!tr.annotation(scrubbed) && tr.changes.touchesRange(start, end)) return false;
      start = tr.changes.mapPos(start);
      end = tr.changes.mapPos(end, 1);
    }
    return start === from;
  }

  open(state, literal, from, to) {
    const custom = this.config.rangeFor ? validRange(ask(this.config.rangeFor, state, from, to)) : null;
    let { min, max, step } = custom ?? suggestRange(literal);
    if (literal.unsigned) min = Math.max(0, min);
    if (max <= min) max = min + step;
    // A host range with a fractional step lets a JS integer such as 7 take decimals.
    const fractional = Boolean(custom) && !Number.isInteger(step) && this.config.language === "js";
    return { from, to, min, max, step, fractional, decimals: Math.max(3, placesOf(step)), dom: null };
  }

  drop() {
    this.session = null;
    return null;
  }

  destroy() {
    this.destroyed = true;
  }

  // A selection-only transaction closes the last history event, so typing
  // right after a drag does not merge into its last step. It waits for the
  // event to finish, since a slider removed mid-update can fire "change".
  settle(view) {
    queueMicrotask(() => {
      if (!this.destroyed && !view.composing) view.dispatch({ selection: view.state.selection });
    });
  }

  // Gives the keyboard back to the editor, unless the page moved focus elsewhere.
  refocus(view, input) {
    queueMicrotask(() => {
      const active = input.ownerDocument.activeElement;
      const idle = !active || active === input || active === input.ownerDocument.body;
      if (!this.destroyed && idle) view.focus();
    });
  }

  // Keeps each control showing its literal after edits it did not make.
  sync(view) {
    const { language } = this.config;
    for (const dom of view.contentDOM.querySelectorAll(".cm-inline-slider, .cm-inline-swatch")) {
      const input = dom.firstChild;
      if (!input || input === view.root.activeElement) continue;
      let pos;
      try {
        pos = view.posAtDOM(dom);
      } catch {
        continue;
      }
      const line = view.state.doc.lineAt(pos);
      if (dom.classList.contains("cm-inline-swatch")) {
        const color = findColors(line.text, language).find((c) => line.from + c.from === pos);
        if (color && input.value !== color.hex) input.value = color.hex;
      } else {
        const literal = findNumber(line.text, pos - line.from, language);
        if (literal && Number(input.value) !== literal.value) input.value = String(literal.value);
      }
    }
  }

  // The literal is found again from the widget's place, since offsets go stale mid-drag.
  scrub = (view, dom, value, session) => {
    let pos;
    try {
      pos = view.posAtDOM(dom);
    } catch {
      return;
    }
    const line = view.state.doc.lineAt(pos);
    const literal = findNumber(line.text, pos - line.from, this.config.language);
    if (!literal || line.from + literal.to !== pos) return;
    const target = literal.unsigned ? Math.max(0, value) : value;
    const loose = session.fractional && literal.integer && !literal.suffix;
    const text = loose ? String(Number(target.toFixed(session.decimals))) : formatNumber(literal.text, target, session.decimals);
    if (text === literal.text) return;
    const from = line.from + literal.from;
    const head = view.state.selection.main.head;
    const offset = head >= pos ? text.length : Math.min(Math.max(head - from, 0), text.length);
    view.dispatch({
      changes: { from, to: pos, insert: text },
      selection: { anchor: from + offset },
      annotations: scrubbed.of(true),
      userEvent: "input.scrub",
    });
  };

  pick = (view, dom, hex) => {
    let pos;
    try {
      pos = view.posAtDOM(dom);
    } catch {
      return;
    }
    const { language } = this.config;
    const line = view.state.doc.lineAt(pos);
    const color = findColors(line.text, language).find((c) => line.from + c.from === pos);
    const rgb = fromHex(hex);
    if (!color || !rgb) return;
    const changes = [];
    color.components.slice(0, 3).forEach((component, i) => {
      // a channel the picker left alone keeps its exact text
      if (Math.round(component.value * 255) === Math.round(rgb[i] * 255)) return;
      const insert = formatComponent(component.text, rgb[i], language);
      changes.push({ from: line.from + component.from, to: line.from + component.to, insert });
    });
    if (!changes.length) return;
    view.dispatch({ changes, annotations: scrubbed.of(true), userEvent: "input.scrub" });
  };
}

const theme = EditorView.baseTheme({
  ".cm-inline-slider": {
    display: "inline-flex",
    alignItems: "center",
    height: "14px",
    margin: "0 3px 0 5px",
    padding: "0 5px",
    verticalAlign: "middle",
    borderRadius: "7px",
    backgroundColor: "rgba(165, 180, 252, 0.08)",
    boxShadow: "inset 0 0 0 1px rgba(165, 180, 252, 0.22)",
  },
  ".cm-inline-slider input": {
    appearance: "none",
    display: "block",
    width: "90px",
    height: "14px",
    margin: "0",
    padding: "0",
    background: "transparent",
    accentColor: "#a5b4fc",
    cursor: "ew-resize",
    outline: "none",
  },
  ".cm-inline-slider input::-webkit-slider-runnable-track": {
    height: "2px",
    borderRadius: "1px",
    background: "rgba(248, 248, 242, 0.28)",
  },
  ".cm-inline-slider input::-webkit-slider-thumb": {
    appearance: "none",
    width: "10px",
    height: "10px",
    marginTop: "-4px",
    border: "none",
    borderRadius: "50%",
    background: "#a5b4fc",
  },
  ".cm-inline-slider input::-moz-range-track": {
    height: "2px",
    borderRadius: "1px",
    background: "rgba(248, 248, 242, 0.28)",
  },
  ".cm-inline-slider input::-moz-range-thumb": {
    width: "10px",
    height: "10px",
    border: "none",
    borderRadius: "50%",
    background: "#a5b4fc",
  },
  ".cm-inline-slider input:focus-visible::-webkit-slider-thumb": {
    boxShadow: "0 0 0 3px rgba(165, 180, 252, 0.35)",
  },
  ".cm-inline-swatch": {
    display: "inline-block",
    margin: "0 5px 0 1px",
    lineHeight: "0",
    verticalAlign: "middle",
  },
  ".cm-inline-swatch input": {
    appearance: "none",
    display: "block",
    boxSizing: "border-box",
    width: "12px",
    height: "12px",
    margin: "0",
    padding: "0",
    border: "1px solid rgba(248, 248, 242, 0.55)",
    borderRadius: "3px",
    background: "none",
    cursor: "pointer",
  },
  ".cm-inline-swatch input::-webkit-color-swatch-wrapper": { padding: "0" },
  ".cm-inline-swatch input::-webkit-color-swatch": { border: "none", borderRadius: "2px" },
  ".cm-inline-swatch input::-moz-color-swatch": { border: "none", borderRadius: "2px" },
  "&light .cm-inline-slider input::-webkit-slider-runnable-track": { background: "rgba(0, 0, 0, 0.25)" },
  "&light .cm-inline-slider input::-moz-range-track": { background: "rgba(0, 0, 0, 0.25)" },
  "&light .cm-inline-swatch input": { borderColor: "rgba(0, 0, 0, 0.35)" },
});

/**
 * Inline sliders and color swatches for one editor.
 * @param {object} [options]
 * @param {"wgsl" | "js"} [options.language] number and comment syntax, "js" by default
 * @param {(state, from: number, to: number) => {min: number, max: number, step: number} | null | undefined} [options.rangeFor]
 *   slider range for the literal at from..to; anything else falls back to the heuristic
 * @param {(state, from: number, to: number) => boolean} [options.isColor]
 *   whether the JS array at from..to is a color; without it, arrays passed to
 *   `.set("...color...", [...])` or to a method named for a color or tint qualify
 * @returns {import("@codemirror/state").Extension}
 */
export function inlineControls(options = {}) {
  const config = {
    language: options.language === "wgsl" ? "wgsl" : "js",
    rangeFor: options.rangeFor,
    isColor: options.isColor,
  };
  const plugin = ViewPlugin.define((view) => new Controls(view, config), {
    decorations: (controls) => controls.decorations,
  });
  return [theme, plugin];
}
