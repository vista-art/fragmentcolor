import type { APIRoute } from "astro";
import * as path from "node:path";
import * as fs from "node:fs/promises";

export const prerender = false;

const WEBSITE_ROOT = process.cwd();
const REPO_ROOT = path.resolve(WEBSITE_ROOT, "../..");

const ALLOWED_PREFIXES = [
  path.join(WEBSITE_ROOT, "src", "content") + path.sep,
  path.join(REPO_ROOT, "docs", "api") + path.sep,
];

function resolveEditablePath(input: string): string | null {
  if (!input) return null;
  const cleaned = input.replace(/^\.?\//, "");
  const candidates = [
    path.resolve(REPO_ROOT, cleaned),
    path.resolve(WEBSITE_ROOT, cleaned),
  ];
  for (const abs of candidates) {
    if (ALLOWED_PREFIXES.some((p) => abs.startsWith(p))) return abs;
  }
  return null;
}

function devOnly(): Response | null {
  if (!import.meta.env.DEV) return new Response("Not Found", { status: 404 });
  return null;
}

export const GET: APIRoute = async ({ url }) => {
  const gate = devOnly();
  if (gate) return gate;

  const rel = url.searchParams.get("path");
  if (!rel) return new Response("path required", { status: 400 });

  const abs = resolveEditablePath(rel);
  if (!abs) return new Response("path not allowed", { status: 403 });

  try {
    const content = await fs.readFile(abs, "utf8");
    return new Response(content, {
      headers: { "content-type": "text/plain; charset=utf-8" },
    });
  } catch (e) {
    return new Response(`read failed: ${e}`, { status: 500 });
  }
};

export const POST: APIRoute = async ({ request }) => {
  const gate = devOnly();
  if (gate) return gate;

  const body = (await request.json()) as { path?: string; content?: string };
  if (!body.path || typeof body.content !== "string") {
    return new Response("path and content required", { status: 400 });
  }

  const abs = resolveEditablePath(body.path);
  if (!abs) return new Response("path not allowed", { status: 403 });

  try {
    await fs.writeFile(abs, body.content, "utf8");
    return new Response(JSON.stringify({ ok: true, path: abs }), {
      headers: { "content-type": "application/json" },
    });
  } catch (e) {
    return new Response(`write failed: ${e}`, { status: 500 });
  }
};
