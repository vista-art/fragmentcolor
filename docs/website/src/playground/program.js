// Turns the text of main.js into a module the browser can import.
//
// The two import forms a demo uses are real Vite syntax, so the same file runs
// in a Vite project. Here the fragmentcolor import resolves to the module the
// page already initialized, and each `./name.wgsl?raw` import becomes the
// current text of that editor tab. Replacements stay on one line, so error line
// numbers match the editor.

const FC_IMPORT =
  /import\s+(?:(\*\s+as\s+[\w$]+)|([\w$]+)?\s*,?\s*(\{[^}]*\})?)\s+from\s*["']fragmentcolor["'];?/g;
const RAW_IMPORT = /import\s+([\w$]+)\s+from\s*["']\.\/([\w.-]+)\?raw["'];?/g;

function rewriteFragmentcolor(match, namespace, fallback, names) {
  if (namespace) return `const ${namespace.replace(/^\*\s+as\s+/, "")} = globalThis.__fragmentcolor;`;
  const parts = [];
  if (fallback) parts.push(`const ${fallback} = globalThis.__fragmentcolor.default;`);
  if (names) {
    const binding = names.replace(/([\w$]+)\s+as\s+([\w$]+)/g, "$1: $2");
    parts.push(`const ${binding} = globalThis.__fragmentcolor;`);
  }
  return parts.join(" ");
}

/**
 * @param {string} text main.js source
 * @param {Record<string, string>} files editor tabs by file name
 * @returns {Promise<{ setup: Function, frame: Function }>}
 */
export async function loadProgram(text, files) {
  const source = text
    .replace(FC_IMPORT, rewriteFragmentcolor)
    .replace(RAW_IMPORT, (_, name, file) => {
      if (!(file in files)) throw new Error(`main.js imports ./${file}, but no tab has that name.`);
      return `const ${name} = ${JSON.stringify(files[file])};`;
    });

  const url = URL.createObjectURL(new Blob([source], { type: "text/javascript" }));
  let module;
  try {
    module = await import(/* @vite-ignore */ url);
  } finally {
    URL.revokeObjectURL(url);
  }
  if (typeof module.setup !== "function") throw new Error("main.js must export a setup function.");
  if (typeof module.frame !== "function") throw new Error("main.js must export a frame function.");
  return module;
}
