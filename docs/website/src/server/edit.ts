// Edits pages on the deployed site, where the filesystem is read-only.
// Reads and writes go through the GitHub contents API. Every save commits to
// one branch, and one pull request at a time carries that branch into main.

import { Buffer } from "node:buffer";
import { createHash, timingSafeEqual } from "node:crypto";

export interface EditEnv {
  /** Secret the editor sends as a Bearer token. Editing is off without it. */
  editToken?: string;
  /** Token for the GitHub REST API. */
  githubToken?: string;
}

const OWNER = "vista-art";
const API = `https://api.github.com/repos/${OWNER}/fragmentcolor`;
const BASE = "main";
const BRANCH = "content/edits";

// Segments start with a letter, digit, `_` or `-`, which rules out `.` and `..`.
const PAGE_PATH =
  /^docs\/website\/src\/content\/docs\/((?:[\w-][\w.-]*\/)*[\w-][\w.-]*)\.mdx?$/;

// The Rust build writes these pages from docs/api and overwrites any edit.
const GENERATED = "api/";

interface Page {
  path: string;
  /** Commit scope: the page's top folder, or `website` for a top-level page. */
  scope: string;
  /** The page's path below its scope, without the extension. */
  name: string;
}

interface Pull {
  html_url: string;
  state: "open" | "closed";
  head: { sha: string };
}

interface File {
  content: string;
  sha: string;
}

/** Where pending edits stand on GitHub. */
type Edits =
  | { kind: "missing" }
  | { kind: "spent" }
  | { kind: "live"; pull: Pull | null };

interface Reply {
  status: number;
  data: unknown;
}

type GitHub = (method: string, path: string, body?: unknown) => Promise<Reply>;

class GitHubError extends Error {}

/** Answers `GET` with a page's source and `POST` with a commit to it. */
export async function handleEdit(request: Request, env: EditEnv): Promise<Response> {
  if (!env.editToken) return text("Not Found", 404);
  if (!authorized(request, env.editToken)) return text("Unauthorized", 401);
  if (!env.githubToken) return text("GITHUB_TOKEN is not set", 503);

  const github = connect(env.githubToken);
  try {
    if (request.method === "GET") return await read(request, github);
    if (request.method === "POST") return await write(request, github);
    return text("Method Not Allowed", 405);
  } catch (error) {
    if (error instanceof GitHubError) return text(error.message, 502);
    throw error;
  }
}

function authorized(request: Request, token: string): boolean {
  const match = /^Bearer (.+)$/.exec(request.headers.get("authorization") ?? "");
  return match !== null && timingSafeEqual(digest(match[1]), digest(token));
}

function digest(value: string): Buffer {
  return createHash("sha256").update(value).digest();
}

function parsePage(path: string): Page | null {
  const slug = PAGE_PATH.exec(path)?.[1];
  if (!slug || slug.startsWith(GENERATED)) return null;
  const [top, ...rest] = slug.split("/");
  return rest.length ? { path, scope: top, name: rest.join("/") } : { path, scope: "website", name: top };
}

async function read(request: Request, github: GitHub): Promise<Response> {
  const path = new URL(request.url).searchParams.get("path");
  if (!path) return text("path required", 400);
  const page = parsePage(path);
  if (!page) return text("path not allowed", 403);

  // A page merged into main after the branch was cut exists only on main.
  const edits = await inspect(github);
  const refs = edits.kind === "live" ? [BRANCH, BASE] : [BASE];
  for (const ref of refs) {
    const file = await fetchFile(github, page, ref);
    if (!file) continue;
    return new Response(Buffer.from(file.content, "base64").toString("utf8"), {
      headers: {
        "content-type": "text/plain; charset=utf-8",
        "cache-control": "no-store",
        "x-edit-ref": ref,
        "x-edit-sha": file.sha,
      },
    });
  }
  return text("page not found", 404);
}

async function write(request: Request, github: GitHub): Promise<Response> {
  const body = (await request.json().catch(() => null)) as Record<string, unknown> | null;
  const { path, content, sha } = body ?? {};
  if (typeof path !== "string" || typeof content !== "string" || typeof sha !== "string") {
    return text("path, content and sha required", 400);
  }
  const page = parsePage(path);
  if (!page) return text("path not allowed", 403);

  // The branch never takes in main's history, which would need the
  // Workflows permission once main changes a workflow file. So a page
  // that reached main after the branch was cut waits for its pull request.
  const edits = await inspect(github);
  if (edits.kind !== "live") {
    await cutBranch(github, edits.kind === "spent");
  } else if (!(await fetchFile(github, page, BRANCH))) {
    return text(
      "This page is newer than the open edits pull request. Merge or close that pull request, then save again.",
      409,
    );
  }

  // GitHub refuses the write when the page changed since the editor loaded it.
  const put = await github("PUT", `/contents/${page.path}`, {
    message: `docs(${page.scope}): edit ${page.name}`,
    content: Buffer.from(content, "utf8").toString("base64"),
    sha,
    branch: BRANCH,
  });
  if (put.status === 409) {
    return text("The page changed on GitHub since the editor opened it.", 409);
  }
  const saved = expect(put, "PUT contents", 200, 201) as { content: { sha: string } };

  const pull = (edits.kind === "live" && edits.pull) || (await openPull(github));
  return Response.json(
    { pr: pull.html_url, sha: saved.content.sha },
    { headers: { "cache-control": "no-store" } },
  );
}

/** A branch counts as spent once its last pull request closed on its current head. */
async function inspect(github: GitHub): Promise<Edits> {
  const ref = await github("GET", `/git/ref/heads/${BRANCH}`);
  if (ref.status === 404) return { kind: "missing" };
  const head = (expect(ref, "GET ref", 200) as { object: { sha: string } }).object.sha;

  const query = new URLSearchParams({
    state: "all",
    base: BASE,
    head: `${OWNER}:${BRANCH}`,
    per_page: "1",
  });
  const pulls = expect(await github("GET", `/pulls?${query}`), "GET pulls", 200) as Pull[];
  const [latest] = pulls;
  if (latest?.state === "open") return { kind: "live", pull: latest };
  if (latest?.head.sha === head) return { kind: "spent" };
  return { kind: "live", pull: null };
}

/** Starts the branch at main's head. A spent branch is deleted, never moved. */
async function cutBranch(github: GitHub, spent: boolean): Promise<void> {
  const base = await github("GET", `/git/ref/heads/${BASE}`);
  const sha = (expect(base, "GET ref", 200) as { object: { sha: string } }).object.sha;
  if (spent) expect(await github("DELETE", `/git/refs/heads/${BRANCH}`), "DELETE ref", 204);
  const created = await github("POST", "/git/refs", { ref: `refs/heads/${BRANCH}`, sha });
  expect(created, "POST ref", 201);
}

async function openPull(github: GitHub): Promise<Pull> {
  const reply = await github("POST", "/pulls", {
    title: "docs(website): edits from the site editor",
    head: BRANCH,
    base: BASE,
    body: `Edits saved with the website editor. Each save adds a commit to \`${BRANCH}\`.`,
  });
  return expect(reply, "POST pulls", 201) as Pull;
}

async function fetchFile(github: GitHub, page: Page, ref: string): Promise<File | null> {
  const reply = await github("GET", `/contents/${page.path}?ref=${encodeURIComponent(ref)}`);
  if (reply.status === 404) return null;
  return expect(reply, "GET contents", 200) as File;
}

function connect(token: string): GitHub {
  return async (method, path, body) => {
    const response = await fetch(`${API}${path}`, {
      method,
      headers: {
        accept: "application/vnd.github+json",
        authorization: `Bearer ${token}`,
        "user-agent": "fragmentcolor-website",
        "x-github-api-version": "2022-11-28",
        ...(body === undefined ? {} : { "content-type": "application/json" }),
      },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    return { status: response.status, data: parseJson(await response.text()) };
  };
}

function parseJson(raw: string): unknown {
  try {
    return raw ? JSON.parse(raw) : null;
  } catch {
    return { message: raw.slice(0, 200) };
  }
}

function expect(reply: Reply, call: string, ...statuses: number[]): unknown {
  if (statuses.includes(reply.status)) return reply.data;
  const message = (reply.data as { message?: string } | null)?.message ?? "no message";
  throw new GitHubError(`GitHub answered ${reply.status} to ${call}: ${message}`);
}

function text(body: string, status: number): Response {
  return new Response(body, {
    status,
    headers: { "content-type": "text/plain; charset=utf-8", "cache-control": "no-store" },
  });
}
