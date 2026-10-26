import assert from "node:assert/strict";
import { Buffer } from "node:buffer";
import { afterEach, beforeEach, describe, it, mock } from "node:test";
import { handleEdit, type EditEnv } from "../src/server/edit.ts";

const ENV: EditEnv = { editToken: "test-edit-token", githubToken: "test-github-token" };
const AUTH = { authorization: "Bearer test-edit-token" };
const POST = "docs/website/src/content/docs/blog/first.mdx";
const API = "https://api.github.com/repos/vista-art/fragmentcolor";

interface Branch {
  head: string;
  /** Workflow changes in the branch's history. */
  workflows: number;
  files: Map<string, { text: string; sha: string }>;
}

interface Pull {
  html_url: string;
  state: "open" | "closed";
  head: { sha: string };
}

/**
 * Just enough of the GitHub REST API to follow one edit through.
 *
 * Like GitHub with a token that lacks the Workflows permission, it refuses
 * to merge or move a branch onto commits that change a workflow file.
 */
class FakeGitHub {
  branches = new Map<string, Branch>();
  pulls: Pull[] = [];
  calls: string[] = [];
  count = 0;

  constructor(files: Record<string, string>) {
    const head = this.nextSha();
    const entries = Object.entries(files).map(([path, text]) => [path, { text, sha: this.nextSha() }] as const);
    this.branches.set("main", { head, workflows: 0, files: new Map(entries) });
  }

  /** Lands a commit on main that changes a file under `.github/workflows`. */
  changeWorkflow(): void {
    const main = this.branches.get("main")!;
    main.head = this.nextSha();
    main.workflows += 1;
  }

  /** Copies main into a new branch, as a ref created at main's head would. */
  cut(): Branch {
    const main = this.branches.get("main")!;
    return { head: main.head, workflows: main.workflows, files: new Map(main.files) };
  }

  nextSha(): string {
    this.count += 1;
    return `sha${this.count}`;
  }

  async fetch(input: string | URL | Request, init?: RequestInit): Promise<Response> {
    const url = new URL(String(input));
    const method = init?.method ?? "GET";
    const headers = new Headers(init?.headers);
    assert.equal(headers.get("authorization"), "Bearer test-github-token");
    const path = url.pathname.slice(new URL(API).pathname.length);
    this.calls.push(`${method} ${path}`);
    const body = init?.body ? JSON.parse(String(init.body)) : null;
    return this.route(method, path, url.searchParams, body);
  }

  route(method: string, path: string, query: URLSearchParams, body: any): Response {
    const main = this.branches.get("main")!;
    const edits = this.branches.get("content/edits");

    if (method === "GET" && path.startsWith("/git/ref/heads/")) {
      const branch = this.branches.get(path.slice("/git/ref/heads/".length));
      return branch ? reply(200, { object: { sha: branch.head } }) : reply(404, { message: "Not Found" });
    }
    if (method === "POST" && path === "/git/refs") {
      const name = body.ref.slice("refs/heads/".length);
      if (this.branches.has(name)) return reply(422, { message: "Reference already exists" });
      assert.equal(body.sha, main.head);
      this.branches.set(name, this.cut());
      return reply(201, {});
    }
    if (method === "DELETE" && path === "/git/refs/heads/content/edits") {
      this.branches.delete("content/edits");
      return new Response(null, { status: 204 });
    }
    if ((method === "PATCH" && path === "/git/refs/heads/content/edits") || path === "/merges") {
      assert.ok(edits);
      if (edits.workflows < main.workflows) {
        return reply(422, { message: "refusing to allow a Personal Access Token to create or update workflow" });
      }
      if (method === "PATCH") {
        this.branches.set("content/edits", this.cut());
        return reply(200, {});
      }
      for (const [file, entry] of main.files) if (!edits.files.has(file)) edits.files.set(file, entry);
      edits.head = this.nextSha();
      return reply(201, { sha: edits.head });
    }
    if (path.startsWith("/contents/")) {
      const file = path.slice("/contents/".length);
      if (method === "GET") {
        const entry = this.branches.get(query.get("ref") ?? "main")?.files.get(file);
        if (!entry) return reply(404, { message: "Not Found" });
        return reply(200, { content: Buffer.from(entry.text).toString("base64"), sha: entry.sha });
      }
      if (method === "PUT") {
        const branch = this.branches.get(body.branch)!;
        if (branch.files.get(file)?.sha !== body.sha) return reply(409, { message: "does not match" });
        const entry = { text: Buffer.from(body.content, "base64").toString("utf8"), sha: this.nextSha() };
        branch.files.set(file, entry);
        branch.head = this.nextSha();
        return reply(200, { content: { sha: entry.sha }, commit: { sha: branch.head, message: body.message } });
      }
    }
    if (method === "GET" && path === "/pulls") {
      assert.equal(query.get("head"), "vista-art:content/edits");
      assert.equal(query.get("base"), "main");
      return reply(200, this.pulls.slice(-1));
    }
    if (method === "POST" && path === "/pulls") {
      const pull: Pull = {
        html_url: `https://github.com/vista-art/fragmentcolor/pull/${100 + this.pulls.length}`,
        state: "open",
        head: { sha: edits!.head },
      };
      this.pulls.push(pull);
      return reply(201, pull);
    }
    throw new Error(`unexpected call ${method} ${path}`);
  }

  /** Closes the open pull request on the branch's current head. */
  closePull(): void {
    const pull = this.pulls.at(-1)!;
    pull.state = "closed";
    pull.head.sha = this.branches.get("content/edits")!.head;
  }
}

function reply(status: number, data: unknown): Response {
  return Response.json(data, { status });
}

function load(path: string, headers: Record<string, string> = AUTH): Request {
  return new Request(`https://fragmentcolor.org/api/edit?path=${encodeURIComponent(path)}`, { headers });
}

function save(path: string, content: string, sha: string, headers: Record<string, string> = AUTH): Request {
  return new Request("https://fragmentcolor.org/api/edit", {
    method: "POST",
    headers: { ...headers, "content-type": "application/json" },
    body: JSON.stringify({ path, content, sha }),
  });
}

let github: FakeGitHub;

beforeEach(() => {
  github = new FakeGitHub({ [POST]: "# First\n" });
  mock.method(globalThis, "fetch", (input: string, init?: RequestInit) => github.fetch(input, init));
});

afterEach(() => mock.restoreAll());

describe("auth", () => {
  it("answers 404 when EDIT_TOKEN is unset", async () => {
    const response = await handleEdit(load(POST), { githubToken: "test-github-token" });
    assert.equal(response.status, 404);
    assert.deepEqual(github.calls, []);
  });

  it("answers 401 without a token or with a wrong one", async () => {
    const wrong: Record<string, string>[] = [
      {},
      { authorization: "Bearer wrong" },
      { authorization: "test-edit-token" },
    ];
    for (const headers of wrong) {
      assert.equal((await handleEdit(load(POST, headers), ENV)).status, 401);
      assert.equal((await handleEdit(save(POST, "x", "sha2", headers), ENV)).status, 401);
    }
    assert.deepEqual(github.calls, []);
  });

  it("answers 503 when GITHUB_TOKEN is unset", async () => {
    const response = await handleEdit(load(POST), { editToken: "test-edit-token" });
    assert.equal(response.status, 503);
  });
});

describe("paths", () => {
  const refused = [
    "docs/website/src/content/docs/api/core/renderer.mdx",
    "docs/website/src/content/docs/api/index.md",
    "docs/website/src/content/docs/tutorials/../api/renderer.mdx",
    "docs/website/src/pages/api/edit.ts",
    "docs/api/core/renderer/renderer.md",
    "docs/website/src/content/docs/blog/first.ts",
    "docs/website/src/content/docs/blog/../../../../../.github/workflows/ci.md",
    "docs/website/src/content/docs/blog/./first.mdx",
    "docs/website/src/content/docs/blog//first.mdx",
    "docs/website/src/content/docs/blog/.hidden.md",
    "docs/website/src/content/docs/blog\\..\\first.mdx",
    "/docs/website/src/content/docs/blog/first.mdx",
    "src/content/docs/blog/first.mdx",
    `${POST}?ref=main`,
    `${POST}\n`,
  ];

  it("refuses generated pages and anything but the site's Markdown pages", async () => {
    for (const path of refused) {
      assert.equal((await handleEdit(load(path), ENV)).status, 403, path);
      assert.equal((await handleEdit(save(path, "x", "sha2"), ENV)).status, 403, path);
    }
    assert.deepEqual(github.calls, []);
  });

  it("asks for a path and a body", async () => {
    assert.equal((await handleEdit(load(""), ENV)).status, 400);
    const request = new Request("https://fragmentcolor.org/api/edit", { method: "POST", headers: AUTH, body: "{" });
    assert.equal((await handleEdit(request, ENV)).status, 400);
  });
});

describe("edits", () => {
  it("reads main and opens the branch and pull request on the first save", async () => {
    const loaded = await handleEdit(load(POST), ENV);
    assert.equal(loaded.status, 200);
    assert.equal(await loaded.text(), "# First\n");
    assert.equal(loaded.headers.get("x-edit-ref"), "main");
    const sha = loaded.headers.get("x-edit-sha")!;

    github.calls = [];
    const saved = await handleEdit(save(POST, "# First edit\n", sha), ENV);
    assert.equal(saved.status, 200);
    const result = (await saved.json()) as { pr: string; sha: string };
    assert.equal(result.pr, "https://github.com/vista-art/fragmentcolor/pull/100");
    assert.deepEqual(github.calls, [
      "GET /git/ref/heads/content/edits",
      "GET /git/ref/heads/main",
      "POST /git/refs",
      `PUT /contents/${POST}`,
      "POST /pulls",
    ]);
    const branch = github.branches.get("content/edits")!;
    assert.deepEqual(branch.files.get(POST), { text: "# First edit\n", sha: result.sha });
    assert.equal(github.branches.get("main")!.files.get(POST)!.text, "# First\n");
  });

  it("edits every hand-written page, scoping the commit by its folder", async () => {
    const pages = {
      [POST]: "docs(blog): edit first",
      "docs/website/src/content/docs/tutorials/infinite-zoom.mdx": "docs(tutorials): edit infinite-zoom",
      "docs/website/src/content/docs/shaders/camera.mdx": "docs(shaders): edit camera",
      "docs/website/src/content/docs/welcome/index.mdx": "docs(welcome): edit index",
      "docs/website/src/content/docs/index.mdx": "docs(website): edit index",
    };
    const main = github.branches.get("main")!;
    for (const path of Object.keys(pages)) main.files.set(path, { text: "# Page\n", sha: github.nextSha() });

    const messages: string[] = [];
    const route = github.route.bind(github);
    github.route = (method, path, query, body) => {
      if (method === "PUT") messages.push(body.message);
      return route(method, path, query, body);
    };
    for (const path of Object.keys(pages)) {
      const sha = (await handleEdit(load(path), ENV)).headers.get("x-edit-sha")!;
      assert.equal((await handleEdit(save(path, "# Edit\n", sha), ENV)).status, 200, path);
    }
    assert.deepEqual(messages, Object.values(pages));
    assert.equal(github.pulls.length, 1);
  });

  it("builds the second save on the branch and reuses the pull request", async () => {
    const first = await handleEdit(save(POST, "# First edit\n", "sha2"), ENV);
    const { pr, sha } = (await first.json()) as { pr: string; sha: string };

    const loaded = await handleEdit(load(POST), ENV);
    assert.equal(await loaded.text(), "# First edit\n");
    assert.equal(loaded.headers.get("x-edit-ref"), "content/edits");
    assert.equal(loaded.headers.get("x-edit-sha"), sha);

    github.calls = [];
    const second = await handleEdit(save(POST, "# Second edit\n", sha), ENV);
    assert.equal(second.status, 200);
    assert.equal(((await second.json()) as { pr: string }).pr, pr);
    assert.deepEqual(github.calls, [
      "GET /git/ref/heads/content/edits",
      "GET /pulls",
      `GET /contents/${POST}`,
      `PUT /contents/${POST}`,
    ]);
    assert.equal(github.pulls.length, 1);
    assert.equal(github.branches.get("content/edits")!.files.get(POST)!.text, "# Second edit\n");
  });

  it("refuses a save over a post that changed since it loaded", async () => {
    await handleEdit(save(POST, "# First edit\n", "sha2"), ENV);
    const stale = await handleEdit(save(POST, "# Other tab\n", "sha2"), ENV);
    assert.equal(stale.status, 409);
    assert.match(await stale.text(), /changed on GitHub/);
    assert.equal(github.branches.get("content/edits")!.files.get(POST)!.text, "# First edit\n");
  });

  it("starts from main again once the pull request closes", async () => {
    await handleEdit(save(POST, "# Rejected edit\n", "sha2"), ENV);
    github.closePull();

    const loaded = await handleEdit(load(POST), ENV);
    assert.equal(await loaded.text(), "# First\n");
    assert.equal(loaded.headers.get("x-edit-ref"), "main");

    github.calls = [];
    const saved = await handleEdit(save(POST, "# Fresh edit\n", loaded.headers.get("x-edit-sha")!), ENV);
    assert.equal(saved.status, 200);
    assert.equal(((await saved.json()) as { pr: string }).pr, "https://github.com/vista-art/fragmentcolor/pull/101");
    assert.deepEqual(github.calls, [
      "GET /git/ref/heads/content/edits",
      "GET /pulls",
      "GET /git/ref/heads/main",
      "DELETE /git/refs/heads/content/edits",
      "POST /git/refs",
      `PUT /contents/${POST}`,
      "POST /pulls",
    ]);
    assert.equal(github.branches.get("content/edits")!.files.get(POST)!.text, "# Fresh edit\n");
  });

  it("keeps saving after main changes a workflow file", async () => {
    const first = await handleEdit(save(POST, "# First edit\n", "sha2"), ENV);
    github.changeWorkflow();

    const second = await handleEdit(save(POST, "# Second edit\n", ((await first.json()) as { sha: string }).sha), ENV);
    assert.equal(second.status, 200);

    github.closePull();
    github.changeWorkflow();
    const sha = (await handleEdit(load(POST), ENV)).headers.get("x-edit-sha")!;
    const third = await handleEdit(save(POST, "# Third edit\n", sha), ENV);
    assert.equal(third.status, 200);
    assert.equal(github.branches.get("content/edits")!.files.get(POST)!.text, "# Third edit\n");
  });

  it("holds a post that reached main after the branch was cut until the pull request closes", async () => {
    await handleEdit(save(POST, "# First edit\n", "sha2"), ENV);
    const late = "docs/website/src/content/docs/blog/late.md";
    const main = github.branches.get("main")!;
    main.files.set(late, { text: "# Late\n", sha: github.nextSha() });
    main.head = github.nextSha();

    const loaded = await handleEdit(load(late), ENV);
    assert.equal(await loaded.text(), "# Late\n");
    assert.equal(loaded.headers.get("x-edit-ref"), "main");
    const sha = loaded.headers.get("x-edit-sha")!;

    const held = await handleEdit(save(late, "# Late edit\n", sha), ENV);
    assert.equal(held.status, 409);
    assert.match(await held.text(), /newer than the open edits pull request/);
    assert.equal(github.branches.get("content/edits")!.files.has(late), false);

    github.closePull();
    const saved = await handleEdit(save(late, "# Late edit\n", sha), ENV);
    assert.equal(saved.status, 200);
    assert.equal(github.branches.get("content/edits")!.files.get(late)!.text, "# Late edit\n");
  });

  it("reports GitHub failures as 502", async () => {
    github.route = () => reply(500, { message: "boom" });
    const response = await handleEdit(load(POST), ENV);
    assert.equal(response.status, 502);
    assert.match(await response.text(), /GitHub answered 500 to GET ref: boom/);
  });
});
