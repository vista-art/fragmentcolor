// locks — Astro integration that tracks `<Lock id="...">` regions in MDX/MD.
//
// Scans `docs/website/src/content/docs/**/*.{md,mdx}`, hashes the inner
// content of every <Lock> block (SHA-256), and tracks per-block version
// history in `<repo>/.claude/locks/locks.json`. The store is gitignored
// and chmod 600'd on POSIX systems — convention-level "agents shouldn't
// poke at this directly," not real enforcement.
//
// Runs in two places:
//   * `astro:server:setup` — initial scan, then watches the docs tree
//     via the dev server's Vite watcher and re-scans only the changed
//     file on save. This is the loop the user wants — text edits don't
//     touch the Rust pipeline.
//   * `astro:build:start` — full scan on production builds. Parse
//     errors fail the build (unpaired/nested `<Lock>`, missing `id`).
//
// Companion CLI lives in Rust: `cargo run --release -p fce --example
// locks -- status | history | diff`. It reads the same JSON; the hash
// scheme is SHA-256 hex on both sides so values round-trip.

import type { AstroIntegration } from "astro";
import { promises as fs } from "fs";
import path from "path";
import { createHash } from "crypto";
import { fileURLToPath } from "url";

/**
 * Astro and Vite disagree on the type of the project root: Astro's
 * `config.root` is a `URL`, Vite's `server.config.root` is an absolute
 * path string. Normalise to a string.
 */
function rootToPath(root: URL | string): string {
  if (typeof root === "string") {
    return root.startsWith("file://") ? fileURLToPath(root) : root;
  }
  return fileURLToPath(root);
}

interface ParsedBlock {
  id: string;
  description?: string;
  comments?: string;
  content: string;
}

interface Snapshot {
  version: number;
  hash: string;
  content: string;
  saved_at: number;
  description?: string;
  comments?: string;
}

interface Block {
  post_id: string;
  lock_id: string;
  current_version: number;
  current_hash: string;
  current_content: string;
  description?: string;
  comments?: string;
  updated_at: number;
  history: Snapshot[];
}

interface Store {
  blocks: Block[];
}

const DOCS_REL = "src/content/docs";
const STORE_REL = ".claude/locks/locks.json";

function hashContent(s: string): string {
  return createHash("sha256").update(s, "utf8").digest("hex");
}

function parseAttr(attrs: string, name: string): string | undefined {
  const re = new RegExp(`(^|\\s)${name}="([^"]*)"`);
  const m = attrs.match(re);
  return m ? m[2] : undefined;
}

function parseLocks(text: string): { blocks: ParsedBlock[]; errors: string[] } {
  const blocks: ParsedBlock[] = [];
  const errors: string[] = [];
  let i = 0;
  const len = text.length;

  while (i < len) {
    const open = text.indexOf("<Lock", i);
    if (open === -1) break;

    // Confirm `<Lock` is followed by whitespace, `>`, or `/` so we
    // don't match `<Lockable`.
    const after = open + 5;
    if (after >= len) {
      errors.push(`file ends mid-<Lock at offset ${open}`);
      break;
    }
    const next = text[after];
    if (next !== " " && next !== "\t" && next !== "\n" && next !== "\r" && next !== ">" && next !== "/") {
      i = open + 1;
      continue;
    }

    const closeOpen = text.indexOf(">", after);
    if (closeOpen === -1) {
      errors.push(`unclosed <Lock tag near offset ${open}`);
      break;
    }
    if (closeOpen > 0 && text[closeOpen - 1] === "/") {
      errors.push(`<Lock /> is self-closing at offset ${open} — Lock must wrap content`);
      i = closeOpen + 1;
      continue;
    }

    const attrs = text.slice(after, closeOpen);
    const contentStart = closeOpen + 1;

    const closeStart = text.indexOf("</Lock>", contentStart);
    if (closeStart === -1) {
      errors.push(`<Lock> opened at offset ${open} never closed`);
      break;
    }

    const nestedOpen = text.indexOf("<Lock", contentStart);
    if (nestedOpen !== -1 && nestedOpen < closeStart) {
      errors.push(`nested Lock blocks not supported (outer at ${open}, inner at ${nestedOpen})`);
      i = closeStart + "</Lock>".length;
      continue;
    }

    const content = text.slice(contentStart, closeStart);
    const id = parseAttr(attrs, "id");
    if (!id) {
      errors.push(`<Lock> at offset ${open} missing required \`id\` attribute`);
      i = closeStart + "</Lock>".length;
      continue;
    }

    blocks.push({
      id,
      description: parseAttr(attrs, "description"),
      comments: parseAttr(attrs, "comments"),
      content,
    });

    i = closeStart + "</Lock>".length;
  }

  return { blocks, errors };
}

async function readStore(p: string): Promise<Store> {
  try {
    const s = await fs.readFile(p, "utf-8");
    const parsed = JSON.parse(s);
    if (parsed && Array.isArray(parsed.blocks)) return parsed as Store;
    return { blocks: [] };
  } catch {
    return { blocks: [] };
  }
}

async function writeStore(p: string, store: Store): Promise<void> {
  await fs.mkdir(path.dirname(p), { recursive: true });
  await fs.writeFile(p, JSON.stringify(store, null, 2));
  if (process.platform !== "win32") {
    try {
      await fs.chmod(p, 0o600);
    } catch {
      // best effort — chmod failure shouldn't break the dev server
    }
  }
}

async function walkDocs(dir: string): Promise<string[]> {
  const out: string[] = [];
  const stat = await fs.stat(dir).catch(() => null);
  if (!stat || !stat.isDirectory()) return out;
  const entries = await fs.readdir(dir, { withFileTypes: true });
  for (const e of entries) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) {
      out.push(...(await walkDocs(p)));
    } else if (e.isFile() && (e.name.endsWith(".mdx") || e.name.endsWith(".md"))) {
      out.push(p);
    }
  }
  return out;
}

function ingest(store: Store, postId: string, parsed: ParsedBlock, now: number): boolean {
  const hash = hashContent(parsed.content);
  const existing = store.blocks.find((b) => b.post_id === postId && b.lock_id === parsed.id);
  if (!existing) {
    store.blocks.push({
      post_id: postId,
      lock_id: parsed.id,
      current_version: 1,
      current_hash: hash,
      current_content: parsed.content,
      description: parsed.description,
      comments: parsed.comments,
      updated_at: now,
      history: [
        {
          version: 1,
          hash,
          content: parsed.content,
          saved_at: now,
          description: parsed.description,
          comments: parsed.comments,
        },
      ],
    });
    return true;
  }
  if (existing.current_hash === hash) {
    let changed = false;
    if (existing.description !== parsed.description) {
      existing.description = parsed.description;
      changed = true;
    }
    if (existing.comments !== parsed.comments) {
      existing.comments = parsed.comments;
      changed = true;
    }
    return changed;
  }
  const next = existing.current_version + 1;
  existing.current_version = next;
  existing.current_hash = hash;
  existing.current_content = parsed.content;
  existing.description = parsed.description;
  existing.comments = parsed.comments;
  existing.updated_at = now;
  existing.history.push({
    version: next,
    hash,
    content: parsed.content,
    saved_at: now,
    description: parsed.description,
    comments: parsed.comments,
  });
  return true;
}

async function scanFile(
  repoRoot: string,
  filePath: string,
  store: Store,
  now: number,
  errors: string[],
): Promise<boolean> {
  let text: string;
  try {
    text = await fs.readFile(filePath, "utf-8");
  } catch (e) {
    errors.push(`read ${filePath}: ${e instanceof Error ? e.message : String(e)}`);
    return false;
  }
  const { blocks, errors: parseErrors } = parseLocks(text);
  const rel = path.relative(repoRoot, filePath).split(path.sep).join("/");
  for (const e of parseErrors) errors.push(`${rel}: ${e}`);
  let changed = false;
  for (const b of blocks) {
    if (ingest(store, rel, b, now)) changed = true;
  }
  return changed;
}

interface RunPaths {
  projectRoot: string;
  repoRoot: string;
  docsDir: string;
  storePath: string;
}

function pathsFromProjectRoot(projectRoot: string): RunPaths {
  const repoRoot = path.resolve(projectRoot, "..", "..");
  return {
    projectRoot,
    repoRoot,
    docsDir: path.join(projectRoot, DOCS_REL),
    storePath: path.join(repoRoot, STORE_REL),
  };
}

export default function locks(): AstroIntegration {
  return {
    name: "locks",
    hooks: {
      "astro:server:setup": async ({ server, logger }) => {
        const projectRoot = rootToPath(server.config.root);
        const { repoRoot, docsDir, storePath } = pathsFromProjectRoot(projectRoot);

        const fullScan = async (): Promise<void> => {
          const now = Math.floor(Date.now() / 1000);
          const store = await readStore(storePath);
          const errors: string[] = [];
          const files = await walkDocs(docsDir);
          for (const f of files) {
            await scanFile(repoRoot, f, store, now, errors);
          }
          for (const e of errors) logger.warn(e);
          await writeStore(storePath, store);
        };

        await fullScan();
        logger.info(`scanned ${docsDir.replace(repoRoot + path.sep, "")} -> ${STORE_REL}`);

        const onChange = async (filePath: string): Promise<void> => {
          if (!filePath.endsWith(".mdx") && !filePath.endsWith(".md")) return;
          if (!filePath.startsWith(docsDir)) return;
          const now = Math.floor(Date.now() / 1000);
          const store = await readStore(storePath);
          const errors: string[] = [];
          await scanFile(repoRoot, filePath, store, now, errors);
          for (const e of errors) logger.warn(e);
          await writeStore(storePath, store);
        };

        server.watcher.on("change", onChange);
        server.watcher.on("add", onChange);
      },

      "astro:build:start": async ({ logger }) => {
        const projectRoot = process.cwd();
        const { repoRoot, docsDir, storePath } = pathsFromProjectRoot(projectRoot);

        const now = Math.floor(Date.now() / 1000);
        const store = await readStore(storePath);
        const errors: string[] = [];
        const files = await walkDocs(docsDir);
        for (const f of files) {
          await scanFile(repoRoot, f, store, now, errors);
        }
        if (errors.length > 0) {
          for (const e of errors) logger.error(e);
          throw new Error(`locks: ${errors.length} parse error(s) — see above`);
        }
        await writeStore(storePath, store);
        logger.info(`scanned ${files.length} file(s) -> ${STORE_REL}`);
      },
    },
  };
}
