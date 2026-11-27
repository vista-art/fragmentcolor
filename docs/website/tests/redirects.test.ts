import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, it } from "node:test";

const SITE = new URL("..", import.meta.url).pathname;
const PAGES = join(SITE, "src/pages");
const CONTENT = join(SITE, "src/content/docs");

/** Every file under `dir`, at any depth. */
function listFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    return entry.isDirectory() ? listFiles(path) : [path];
  });
}

/** Each endpoint under `src/pages` that calls `redirectTo`: the route it serves and its destination. */
function listRedirects(): { route: string; destination: string }[] {
  return listFiles(PAGES).flatMap((file) => {
    const match = readFileSync(file, "utf8").match(/redirectTo\("([^"]+)"\)/);
    if (!match) return [];
    const route = "/" + relative(PAGES, file).replace(/\.ts$/, "");
    return [{ route, destination: match[1] }];
  });
}

/** The content page the site builds at `path`, such as `/api/errors/`, if there is one. */
function findPage(path: string): string | undefined {
  const slug = path.replace(/^\/+|\/+$/g, "");
  const stems = slug === "" ? ["index"] : [slug, `${slug}/index`];
  return stems
    .flatMap((stem) => [`${stem}.md`, `${stem}.mdx`])
    .map((name) => join(CONTENT, name))
    .find((file) => existsSync(file));
}

/** The id Starlight gives a heading: `{#id}` when the heading sets one, its slug otherwise. */
function readHeadingIds(page: string): Set<string> {
  const ids = new Set<string>();
  let fenced = false;
  for (const line of readFileSync(page, "utf8").split("\n")) {
    if (line.trimStart().startsWith("```")) fenced = !fenced;
    const heading = !fenced && line.match(/^#{1,6} (.*)$/);
    if (!heading) continue;
    const explicit = heading[1].match(/\\\{#([^\\]+)\\\}\s*$/);
    const slug = heading[1]
      .toLowerCase()
      .replace(/[^\p{L}\p{N}\s_-]/gu, "")
      .replace(/ /g, "-");
    ids.add(explicit ? explicit[1] : slug);
  }
  return ids;
}

describe("redirects", () => {
  const redirects = listRedirects();

  it("finds the redirect endpoints", () => {
    assert.ok(redirects.length > 0);
  });

  for (const { route, destination } of redirects) {
    it(`${route} leads to a page the site builds`, () => {
      const [path, anchor] = destination.split("#");
      const page = findPage(path);
      assert.ok(page, `${destination} names no page`);
      if (anchor) assert.ok(readHeadingIds(page).has(anchor), `${destination} names no heading`);
    });

    it(`${route} shadows no page`, () => {
      assert.equal(findPage(route), undefined, `a page still sits at ${route}`);
    });
  }
});
