// Shader library index for the playground picker. Every entry is a function
// from the shader catalog pages whose WGSL file the site serves, so a slug
// listed here always resolves through Shader.fetch. Signatures and declared
// names come from that file, so they match the code the slug loads.

import type { APIRoute } from "astro";
import { publicDir, srcDir } from "astro:config/server";
import { existsSync } from "node:fs";
import { readFile, readdir } from "node:fs/promises";

export const prerender = true;

interface Entry {
  slug: string;
  category: string;
  categoryTitle: string;
  name: string;
  signature: string;
  summary: string;
  preview: string | null;
  defines: string[];
}

interface Category {
  id: string;
  title: string;
}

const NAME = /^[A-Za-z_][A-Za-z0-9_]*$/;
const INDEX_LINK = /^\s*[-*]\s+\[([^\]]+)\]\(\/shaders\/([^/)\s]+)\/?\)/gm;
const SIGNATURE = /```wgsl[^\n]*\n([\s\S]*?)```/;
const PREVIEW = /!\[[^\]]*\]\(([^)\s]+)\)/;
const COMMENT = /\/\*[\s\S]*?\*\/|\/\/[^\n]*/g;
const DECLARATION =
  /^(?:@\w+(?:\([^)]*\))?\s+)*(?:fn|struct|const|alias|override|var(?:<[^>]*>)?)\s+([A-Za-z_][A-Za-z0-9_]*)/gm;

function folder(base: string | URL, path: string): URL {
  const root = new URL(base);
  if (!root.pathname.endsWith("/")) root.pathname += "/";
  return new URL(path, root);
}

function stripFrontmatter(text: string): string {
  return text.replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n/, "");
}

function readTitle(text: string): string | null {
  const match = /^---\r?\n[\s\S]*?^title:\s*["']?(.*?)["']?\s*$/m.exec(text);
  return match ? match[1] : null;
}

// Markdown to plain text, leaving the inside of code spans untouched.
function toPlainText(markdown: string): string {
  return markdown
    .split(/(`[^`]*`)/)
    .map((part) => {
      if (part.startsWith("`") && part.endsWith("`") && part.length > 1) {
        return part.slice(1, -1);
      }
      return part
        .replace(/!?\[([^\]]*)\]\([^)]*\)/g, "$1")
        .replace(/(\*\*|__)(.+?)\1/g, "$2")
        .replace(/(^|[^\w*])[*_]([^*_\s][^*_]*?)[*_](?=[^\w*]|$)/g, "$1$2");
    })
    .join("")
    .replace(/\s+/g, " ")
    .trim();
}

// Top-level names a WGSL file declares, and the signature of `name` as written there.
function readSource(source: string, name: string): { defines: string[]; signature: string | null } {
  const code = source.replace(COMMENT, "");
  const defines = [...new Set(Array.from(code.matchAll(DECLARATION), (match) => match[1]))];
  const header = new RegExp(`^fn\\s+${name}\\s*\\([^{]*`, "m").exec(code);
  const signature = header ? header[0].replace(/[ \t]+$/gm, "").trim() : null;
  return { defines, signature };
}

async function readCategories(catalog: URL): Promise<Category[]> {
  const files = (await readdir(catalog)).filter((file) => file.endsWith(".mdx") && file !== "index.mdx");
  const pages = new Set(files.map((file) => file.slice(0, -".mdx".length)));
  const index = await readFile(new URL("index.mdx", catalog), "utf8").catch(() => "");
  const ordered: Category[] = [];
  for (const [, title, id] of index.matchAll(INDEX_LINK)) {
    if (pages.has(id) && !ordered.some((category) => category.id === id)) {
      ordered.push({ id, title: toPlainText(title) });
    }
  }
  const rest = [...pages].filter((id) => !ordered.some((category) => category.id === id)).sort();
  for (const id of rest) ordered.push({ id, title: "" });
  return ordered;
}

async function parseSection(section: string, category: Category, shaders: URL): Promise<Entry | null> {
  const newline = section.indexOf("\n");
  const name = (newline < 0 ? section : section.slice(0, newline)).trim();
  if (!NAME.test(name)) return null;
  const source = await readFile(new URL(`${category.id}/${name}.wgsl`, shaders), "utf8").catch(() => null);
  if (source === null) return null;
  const code = readSource(source, name);

  const body = newline < 0 ? "" : section.slice(newline + 1);
  const fence = body.indexOf("```");
  const prose = (fence < 0 ? body : body.slice(0, fence)).replace(PREVIEW, "");
  const signature = code.signature ?? SIGNATURE.exec(body)?.[1].trim() ?? "";
  const image = PREVIEW.exec(body)?.[1] ?? `/shaders/${category.id}/${name}.png`;
  const preview = existsSync(folder(publicDir, image.replace(/^\//, ""))) ? image : null;

  return {
    slug: `${category.id}/${name}`,
    category: category.id,
    categoryTitle: category.title,
    name,
    signature,
    summary: toPlainText(prose),
    preview,
    defines: code.defines,
  };
}

async function readLibrary(): Promise<Entry[]> {
  const catalog = folder(srcDir, "content/docs/shaders/");
  const shaders = folder(publicDir, "shaders/");
  const entries: Entry[] = [];
  for (const category of await readCategories(catalog)) {
    const page = await readFile(new URL(`${category.id}.mdx`, catalog), "utf8");
    if (!category.title) category.title = readTitle(page) ?? category.id;
    const sections = stripFrontmatter(page).split(/^## /m).slice(1);
    for (const section of sections) {
      const entry = await parseSection(section, category, shaders);
      if (entry) entries.push(entry);
    }
  }
  return entries;
}

export const GET: APIRoute = async () => {
  const entries = await readLibrary();
  return new Response(JSON.stringify(entries), {
    headers: { "content-type": "application/json; charset=utf-8" },
  });
};
