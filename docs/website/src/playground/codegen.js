// Writes a playground demo's live state as a program in every language
// FragmentColor ships. JavaScript is the editable program the page runs; the
// Rust, Python, Swift, and Kotlin tabs are the same setup and frame functions,
// spelled the way each binding takes them.

export const LANGUAGES = [
  { id: "js", label: "JavaScript", file: "main.js" },
  { id: "rust", label: "Rust", file: "main.rs" },
  { id: "python", label: "Python", file: "main.py" },
  { id: "swift", label: "Swift", file: "Main.swift" },
  { id: "kotlin", label: "Kotlin", file: "Main.kt" },
];

const READ_ONLY = ["rust", "python", "swift", "kotlin"];

// What frame receives each tick, in the host's argument order, and the type
// each language gives that argument.
const FRAME_INPUTS = [
  { key: "time", param: "time", rust: "f32", swift: "Float", kotlin: "Float" },
  { key: "resolution", param: "size", rust: "[f32; 2]", swift: "[Float]", kotlin: "List<Float>" },
  { key: "pointer", param: "pointer", rust: "[f32; 2]", swift: "[Float]", kotlin: "List<Float>" },
];

// Line widths the default formatters use: rustfmt, black, swift-format, ktlint.
const WIDTH = { rust: 100, python: 88, swift: 100, kotlin: 100 };

// A uniform's scalar type, in the controls' words or WGSL's.
const SCALARS = { float: "float", f32: "float", f16: "float", int: "int", i32: "int", uint: "uint", u32: "uint" };

// Rust infers a whole integer array from the suffix on its first item.
const RUST_SUFFIX = { int: "i32", uint: "u32" };

// Comment syntax for the note a demo gets in a language it does not provide.
const COMMENT = { js: "//", rust: "//", python: "#", swift: "//", kotlin: "//" };

const LIBRARY_LINE =
  /^([ \t]*const[ \t]+LIBRARY[ \t]*=[ \t]*)\[([^\]\r\n]*)\]([ \t]*;?[ \t]*(?:\/\/[^\r\n]*)?\r?)$/m;

const STRING = /"((?:[^"\\\r\n]|\\.)*)"|'((?:[^'\\\r\n]|\\.)*)'/g;

/**
 * @param {{ kind, library, files, values, frame, demo }} model the demo's live state, as host.js builds it
 * @returns {{ js: string, rust: string, python: string, swift: string, kotlin: string }}
 */
export function generateCode(model) {
  const demo = model.demo ?? {};
  const spec = describe(model);
  const custom = typeof demo.code === "function" ? (demo.code(model) ?? {}) : {};
  // Only fragment demos fall back to the template; other kinds bring their own programs.
  const fallback = spec.kind === "fragment" ? WRITERS : null;
  const write = (lang) => fallback?.[lang](spec) ?? writeMissing(lang, spec);
  const code = { js: demo.js ? replaceLibrary(demo.js, spec.library) : write("js") };
  for (const lang of READ_ONLY) code[lang] = custom[lang] ?? write(lang);
  return code;
}

/**
 * Source literal for a uniform value, in the form that language's `set` takes.
 * @param {string} lang a LANGUAGES id
 * @param {number | boolean | ArrayLike<number>} value
 * @param {string} [scalar] "float", "int" or "uint" (or f32, i32, u32); floats when absent
 */
export function formatValue(lang, value, scalar = "float") {
  if (typeof value === "boolean") return lang === "python" ? (value ? "True" : "False") : String(value);
  const type = SCALARS[scalar] ?? "float";
  const list = Array.isArray(value) || ArrayBuffer.isView(value);
  const items = Array.from(list ? value : [value], (item) => formatNumber(lang, item, type));
  if (lang === "rust" && RUST_SUFFIX[type] && items.length) items[0] += RUST_SUFFIX[type];
  if (!list) return items[0];
  return lang === "kotlin" ? `listOf(${items.join(", ")})` : `[${items.join(", ")}]`;
}

/**
 * The slugs on the `const LIBRARY = [...];` line, or null when the program
 * has no such line or the array holds anything but string literals.
 * @param {string} jsText
 * @returns {string[] | null}
 */
export function parseLibrary(jsText) {
  const match = LIBRARY_LINE.exec(jsText ?? "");
  if (!match) return null;
  const slugs = [];
  const rest = match[2].replace(STRING, (_, double, single) => {
    slugs.push((double ?? single).replace(/\\(.)/g, "$1"));
    return ",";
  });
  return /^[\s,]*$/.test(rest) ? slugs : null;
}

/**
 * Rewrites the array on the `const LIBRARY = [...];` line, keeping its
 * indent and any trailing comment. Text without a parsable line comes back as is.
 * @param {string} jsText
 * @param {string[]} slugs
 */
export function replaceLibrary(jsText, slugs) {
  if (parseLibrary(jsText) === null) return jsText;
  return jsText.replace(LIBRARY_LINE, (_, head, _items, tail) => `${head}${jsList(slugs)}${tail}`);
}

// Model -> the parts every template needs. A value entry may carry its
// scalar type third, as in ["u.count", 8, "uint"]; without one it is a float.
function describe(model) {
  const demo = model.demo ?? {};
  const values = model.values ?? demo.values ?? [];
  const files = model.files ?? demo.files ?? [];
  const frame = model.frame ?? demo.frame ?? {};
  const names = Array.isArray(files) ? files : Object.keys(files);
  return {
    kind: model.kind ?? demo.kind ?? "fragment",
    title: demo.title ?? demo.id ?? null,
    library: model.library ?? demo.library ?? [],
    files: nameSources(names),
    values: Array.isArray(values) ? values : Object.entries(values),
    frame: FRAME_INPUTS.filter((input) => frame[input.key]).map((input) => ({
      ...input,
      uniform: frame[input.key],
    })),
  };
}

// One file is SOURCE / source; several are named after their files, like COMMON_SOURCE.
function nameSources(files) {
  const taken = new Set();
  return files.map((file) => {
    const dot = file.lastIndexOf(".");
    const stem = dot > 0 ? file.slice(0, dot) : file;
    const extension = dot > 0 ? file.slice(dot + 1) : null;
    const words = files.length === 1 ? [] : stem.toLowerCase().split(/[^a-z0-9]+/).filter(Boolean);
    if (/^\d/.test(words[0] ?? "")) words.unshift("file");
    let suffix = "";
    for (let n = 2; taken.has(words.join("_") + suffix); n++) suffix = String(n);
    taken.add(words.join("_") + suffix);
    const camel = words.map((word, i) => (i ? word[0].toUpperCase() + word.slice(1) : word)).join("");
    return {
      file,
      stem,
      extension,
      upper: [...words, "source"].join("_").toUpperCase() + suffix,
      lower: (camel ? `${camel}Source` : "source") + suffix,
      url: (camel ? `${camel}URL` : "url") + suffix,
    };
  });
}

function formatNumber(lang, value, type) {
  const number = Number(value);
  if (type !== "float") return String(Number.isFinite(number) ? Math.round(number) || 0 : 0);
  const rounded = Number.isFinite(number) ? Number(number.toPrecision(6)) : 0;
  const text = String(rounded === 0 ? 0 : rounded);
  if (lang === "js") return text;
  const float = /[.e]/.test(text) ? text : `${text}.0`;
  return lang === "kotlin" ? `${float}f` : float;
}

function quote(text) {
  return JSON.stringify(String(text));
}

function jsList(items) {
  return `[${items.map(quote).join(", ")}]`;
}

// `head(a, b, c)tail` on one line when it fits, otherwise one item per line.
function wrap(indent, head, items, tail, width, trailing) {
  const line = `${indent}${head}${items.join(", ")}${tail}`;
  if (line.length <= width) return [line];
  const last = items.length - 1;
  return [
    `${indent}${head}`,
    ...items.map((item, i) => `${indent}    ${item}${i < last || trailing ? "," : ""}`),
    `${indent}${tail}`,
  ];
}

function writeJs(spec) {
  const lines = ['import { Shader } from "fragmentcolor";'];
  for (const source of spec.files) lines.push(`import ${source.upper} from "./${source.file}?raw";`);
  const parts = ["...LIBRARY", ...spec.files.map((source) => source.upper)].join(", ");
  lines.push(
    "",
    `const LIBRARY = ${jsList(spec.library)};`,
    "",
    "export async function setup(renderer, target) {",
    `  const shader = await Shader.fetch([${parts}]);`,
  );
  for (const [key, value, type] of spec.values) {
    lines.push(`  shader.set(${quote(key)}, ${formatValue("js", value, type)});`);
  }
  lines.push(
    "  return { shader };",
    "}",
    "",
    "export function frame({ shader }, renderer, target, time, size, pointer) {",
  );
  for (const input of spec.frame) lines.push(`  shader.set(${quote(input.uniform)}, ${input.param});`);
  lines.push("  renderer.render(shader, target);", "}");
  return `${lines.join("\n")}\n`;
}

function writeRust(spec) {
  const lines = ["use std::error::Error;", "", "use fragmentcolor::{Renderer, Shader, Target};", ""];
  if (spec.files.length) {
    for (const source of spec.files) lines.push(`const ${source.upper}: &str = include_str!(${quote(source.file)});`);
    lines.push("");
  }
  const parts = [...spec.library.map(quote), ...spec.files.map((source) => source.upper)];
  const call = `    let shader = Shader::new([${parts.join(", ")}])?;`;
  lines.push(
    "// Add a `fn main` that opens a window,",
    "// then calls setup once and frame on every redraw.",
    "fn setup() -> Result<Shader, Box<dyn Error>> {",
  );
  if (call.length <= WIDTH.rust) lines.push(call);
  else lines.push("    let shader = Shader::new([", ...parts.map((part) => `        ${part},`), "    ])?;");
  for (const [key, value, type] of spec.values) {
    lines.push(`    shader.set(${quote(key)}, ${formatValue("rust", value, type)})?;`);
  }
  lines.push("    Ok(shader)", "}", "");
  const params = [
    "shader: &Shader",
    "renderer: &Renderer",
    "target: &impl Target",
    ...spec.frame.map((input) => `${input.param}: ${input.rust}`),
  ];
  lines.push(...wrap("", "fn frame(", params, ") -> Result<(), Box<dyn Error>> {", WIDTH.rust, true));
  for (const input of spec.frame) lines.push(`    shader.set(${quote(input.uniform)}, ${input.param})?;`);
  lines.push("    renderer.render(shader, target)?;", "    Ok(())", "}");
  return `${lines.join("\n")}\n`;
}

function writePython(spec) {
  const lines = [];
  if (spec.files.length) lines.push("from pathlib import Path", "");
  lines.push("from fragmentcolor import Shader", "");
  lines.push(...wrap("", "LIBRARY = [", spec.library.map(quote), "]", WIDTH.python, true));
  for (const source of spec.files) {
    lines.push(`${source.upper} = Path(__file__).with_name(${quote(source.file)}).read_text()`);
  }
  const parts = ["*LIBRARY", ...spec.files.map((source) => source.upper)].join(", ");
  lines.push(
    "",
    "",
    "# Call setup() once, then frame() on every redraw of your canvas.",
    "def setup():",
    `    shader = Shader([${parts}])`,
  );
  for (const [key, value, type] of spec.values) {
    lines.push(`    shader.set(${quote(key)}, ${formatValue("python", value, type)})`);
  }
  lines.push("    return shader", "", "");
  const params = ["shader", "renderer", "target", ...spec.frame.map((input) => input.param)];
  lines.push(`def frame(${params.join(", ")}):`);
  for (const input of spec.frame) lines.push(`    shader.set(${quote(input.uniform)}, ${input.param})`);
  lines.push("    renderer.render(shader, target)");
  return `${lines.join("\n")}\n`;
}

function writeSwift(spec) {
  const lines = [];
  if (spec.files.length) lines.push("import Foundation");
  lines.push("import FragmentColor", "");
  if (spec.library.length) lines.push(...wrap("", "let library = [", spec.library.map(quote), "]", WIDTH.swift, true));
  else lines.push("let library: [String] = []");
  lines.push(
    "",
    "// Call setup once, then frame from your MTKView or CADisplayLink callback.",
    "func setup() throws -> Shader {",
  );
  for (const source of spec.files) {
    const extension = source.extension === null ? "nil" : quote(source.extension);
    lines.push(
      `    let ${source.url} = Bundle.main.url(forResource: ${quote(source.stem)}, withExtension: ${extension})!`,
      `    let ${source.lower} = try String(contentsOf: ${source.url}, encoding: .utf8)`,
    );
  }
  const sources = spec.files.map((source) => source.lower);
  const parts = sources.length ? `library + [${sources.join(", ")}]` : "library";
  lines.push(`    let shader = try Shader(${parts})`);
  for (const [key, value, type] of spec.values) {
    lines.push(`    try shader.set(${quote(key)}, ${formatValue("swift", value, type)})`);
  }
  lines.push("    return shader", "}", "");
  const params = [
    "shader: Shader",
    "renderer: Renderer",
    "target: WindowTarget",
    ...spec.frame.map((input) => `${input.param}: ${input.swift}`),
  ];
  lines.push(...wrap("", "func frame(", params, ") throws {", WIDTH.swift, false));
  for (const input of spec.frame) lines.push(`    try shader.set(${quote(input.uniform)}, ${input.param})`);
  lines.push("    try renderer.render(shader, target)", "}");
  return `${lines.join("\n")}\n`;
}

function writeKotlin(spec) {
  const kotlinQuote = (text) => quote(text).replace(/\$/g, "\\$");
  const lines = [];
  if (spec.files.length) lines.push("import android.content.res.AssetManager");
  lines.push("import org.fragmentcolor.*", "");
  if (spec.library.length) {
    lines.push(...wrap("", "private val LIBRARY = listOf(", spec.library.map(kotlinQuote), ")", WIDTH.kotlin, true));
  } else {
    lines.push("private val LIBRARY = emptyList<String>()");
  }
  lines.push(
    "",
    "// Call setup once, then frame from your Choreographer or render thread callback.",
    spec.files.length ? "fun setup(assets: AssetManager): Shader {" : "fun setup(): Shader {",
  );
  for (const source of spec.files) {
    lines.push(`    val ${source.lower} = assets.open(${kotlinQuote(source.file)}).bufferedReader().use { it.readText() }`);
  }
  const sources = spec.files.map((source) => source.lower);
  let parts = "LIBRARY";
  if (sources.length === 1) parts = `LIBRARY + ${sources[0]}`;
  if (sources.length > 1) parts = `LIBRARY + listOf(${sources.join(", ")})`;
  lines.push(`    val shader = Shader(${parts})`);
  for (const [key, value, type] of spec.values) {
    lines.push(`    shader.set(${kotlinQuote(key)}, ${formatValue("kotlin", value, type)})`);
  }
  lines.push("    return shader", "}", "");
  const params = [
    "shader: Shader",
    "renderer: Renderer",
    "target: WindowTarget",
    ...spec.frame.map((input) => `${input.param}: ${input.kotlin}`),
  ];
  lines.push(...wrap("", "fun frame(", params, ") {", WIDTH.kotlin, true));
  for (const input of spec.frame) lines.push(`    shader.set(${kotlinQuote(input.uniform)}, ${input.param})`);
  lines.push("    renderer.render(shader, target)", "}");
  return `${lines.join("\n")}\n`;
}

// A one-line note for a language a non-fragment demo leaves out.
function writeMissing(lang, spec) {
  const label = LANGUAGES.find((language) => language.id === lang)?.label ?? lang;
  const name = spec.title ? `The ${spec.title} demo` : "This demo";
  return `${COMMENT[lang]} ${name} (${spec.kind}) has no ${label} program.\n`;
}

const WRITERS = { js: writeJs, rust: writeRust, python: writePython, swift: writeSwift, kotlin: writeKotlin };
