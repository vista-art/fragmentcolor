// Uniform controls generated from what a shader reports about itself:
// `listKeys()` names every uniform and struct field, `get()` gives the value
// and its shape, and the WGSL declaration adds the type and an optional
// `// min..max` range comment.

const COMPONENTS = ["x", "y", "z", "w"];
const SCALARS = { f32: "float", f16: "float", i32: "int", u32: "uint" };

// Reads `name: type, // comment` struct fields and `var<uniform> name: type; // comment` roots.
function findDeclaration(sources, key) {
  const leaf = key.split(".").pop();
  const field = new RegExp(
    `(?:var<uniform>\\s+)?\\b${leaf}\\s*:\\s*([\\w<>, ]+?)\\s*[,;]?[ \\t]*(?://[ \\t]*(.*))?$`,
    "m",
  );
  for (const [index, source] of sources.entries()) {
    const match = source.match(field);
    if (match) {
      return { type: match[1].replace(/\s+/g, ""), comment: match[2] ?? "", order: index * 1e6 + match.index };
    }
  }
  return null;
}

// "vec3<f32>" -> { size: 3, scalar: "float" }; null for types a slider cannot drive.
function parseType(type) {
  if (!type) return null;
  if (SCALARS[type]) return { size: 1, scalar: SCALARS[type] };
  const vector = type.match(/^vec([234])(?:<(\w+)>|([fiu]))$/);
  if (!vector) return null;
  const inner = vector[2] ?? { f: "f32", i: "i32", u: "u32" }[vector[3]];
  return SCALARS[inner] ? { size: Number(vector[1]), scalar: SCALARS[inner] } : null;
}

function readNumbers(value) {
  if (typeof value === "number") return [value];
  if (Array.isArray(value) || ArrayBuffer.isView(value)) {
    const list = Array.from(value);
    if (list.length >= 1 && list.length <= 4 && list.every((n) => typeof n === "number")) return list;
  }
  return null;
}

function defaultRange(numbers, scalar) {
  const peak = Math.max(...numbers.map(Math.abs));
  if (scalar !== "float") return [scalar === "uint" ? 0 : -Math.max(10, peak * 2), Math.max(10, peak * 2)];
  if (numbers.every((n) => n >= 0 && n <= 1)) return [0, 1];
  const span = peak === 0 ? 1 : peak * 2;
  return [numbers.some((n) => n < 0) ? -span : 0, span];
}

/**
 * Describes one control per leaf key that a slider or color picker can drive.
 * @param {Array<{ shader: any, hidden: Set<string> }>} entries shaders and the keys their frame sets
 * @param {string[]} sources WGSL sources searched for declarations and range comments
 */
export function describeControls(entries, sources) {
  const controls = [];
  for (const { shader, hidden } of entries) {
    const keys = shader.listKeys();
    for (const key of keys) {
      if (hidden.has(key)) continue;
      if (keys.some((other) => other.startsWith(`${key}.`))) continue;
      let value;
      try {
        value = shader.get(key);
      } catch {
        continue;
      }
      const numbers = readNumbers(value);
      if (!numbers) continue;

      const declaration = findDeclaration(sources, key);
      const parsed = parseType(declaration?.type);
      if (declaration && !parsed) continue;
      const type = parsed ?? { size: numbers.length, scalar: "float" };
      if (type.size !== numbers.length) continue;

      const range = declaration?.comment.match(/(-?\d*\.?\d+)\s*\.\.\s*(-?\d*\.?\d+)/);
      const [min, max] = range ? [Number(range[1]), Number(range[2])] : defaultRange(numbers, type.scalar);
      const leaf = key.split(".").pop();
      const color = /colou?r|tint/i.test(leaf) && type.size >= 3 && type.scalar === "float";
      const step = type.scalar === "float" ? (max - min) / 1000 : 1;
      const order = declaration?.order ?? Infinity;
      controls.push({ shader, key, color, scalar: type.scalar, size: type.size, min, max, step, value: numbers, order });
    }
  }
  return controls.sort((a, b) => a.order - b.order);
}

function toHex(rgb) {
  return `#${rgb
    .slice(0, 3)
    .map((c) => Math.round(Math.min(Math.max(c, 0), 1) * 255).toString(16).padStart(2, "0"))
    .join("")}`;
}

function fromHex(hex) {
  return [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
}

function format(n, step) {
  return step >= 1 ? String(Math.round(n)) : n.toFixed(Math.abs(n) >= 100 ? 1 : 3).replace(/\.?0+$/, "") || "0";
}

/**
 * Renders the controls into `container` and calls `onInput(control, value)` as they move.
 */
export function renderControls(container, controls, onInput) {
  container.replaceChildren();
  container.hidden = controls.length === 0;
  for (const control of controls) {
    const row = document.createElement("div");
    row.className = "fcp-control";
    const label = document.createElement("span");
    label.className = "fcp-control-key";
    label.textContent = control.key;
    row.append(label);

    const value = [...control.value];
    if (control.color) {
      const input = document.createElement("input");
      input.type = "color";
      input.value = toHex(value);
      input.setAttribute("aria-label", control.key);
      input.addEventListener("input", () => {
        const [r, g, b] = fromHex(input.value);
        value.splice(0, 3, r, g, b);
        onInput(control, [...value]);
      });
      row.append(input);
      container.append(row);
      continue;
    }

    const sliders = document.createElement("div");
    sliders.className = "fcp-control-sliders";
    value.forEach((component, i) => {
      const field = document.createElement("label");
      field.className = "fcp-control-slider";
      const name = document.createElement("span");
      if (control.size > 1) name.textContent = COMPONENTS[i];
      field.append(name);
      const input = document.createElement("input");
      input.type = "range";
      input.min = String(Math.min(control.min, component));
      input.max = String(Math.max(control.max, component));
      input.step = String(control.step);
      input.value = String(component);
      input.setAttribute("aria-label", control.size > 1 ? `${control.key}.${COMPONENTS[i]}` : control.key);
      const readout = document.createElement("output");
      readout.textContent = format(component, control.step);
      input.addEventListener("input", () => {
        value[i] = Number(input.value);
        readout.textContent = format(value[i], control.step);
        onInput(control, control.size === 1 ? value[0] : [...value]);
      });
      field.append(input, readout);
      sliders.append(field);
    });
    row.append(sliders);
    container.append(row);
  }
}
