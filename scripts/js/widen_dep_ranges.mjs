#!/usr/bin/env node
/*
Widen dependency ranges in package.json files to "major-only" ranges.

Policy:
- For any semver spec we can parse (e.g., 5, 5.0, ^5.1.2, ~0.34.4, 7.x, 5.*),
  rewrite to: 
    ">=<currentVersion> <nextMajor>.0.0"
  where currentVersion is the best-effort minimal version parsed from the spec.
- For 0.x.y, nextMajor is 1.0.0 (we relax minors per user preference).
- Skip non-semver specs: workspace:, file:, link:, git:, http(s):, npm:, tags (latest, beta, etc.).
- Apply to dependencies and devDependencies.

Additionally aligns "fragmentcolor" dependency to the crate version from Cargo.toml
by enforcing the range: ">=<crateVersion> <(major+1).0.0".
*/

import { promises as fs } from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';

const repoRoot = process.cwd();

const SKIP_DIRS = new Set(['node_modules', 'pkg', 'dist', 'build', '.astro']);

function shouldSkipDir(p) {
  const base = path.basename(p);
  return SKIP_DIRS.has(base);
}

async function exists(p) {
  try { await fs.stat(p); return true; } catch { return false; }
}

async function* walk(dir, maxDepth = 5, depth = 0) {
  if (depth > maxDepth) return;
  let ents;
  try { ents = await fs.readdir(dir, { withFileTypes: true }); } catch { return; }
  for (const ent of ents) {
    const full = path.join(dir, ent.name);
    if (ent.isDirectory()) {
      if (shouldSkipDir(full)) continue;
      yield* walk(full, maxDepth, depth + 1);
    } else if (ent.isFile() && ent.name === 'package.json') {
      yield full;
    }
  }
}

function extractVersion(spec) {
  // Extract first occurrence of X[.Y][.Z]
  const m = String(spec).match(/(\d+)(?:\.(\d+))?(?:\.(\d+))?/);
  if (!m) return null;
  const major = parseInt(m[1] ?? '0', 10);
  const minor = parseInt(m[2] ?? '0', 10);
  const patch = parseInt(m[3] ?? '0', 10);
  return { major, minor, patch };
}

function isSemverLike(spec) {
  const s = String(spec).trim();
  if (!s) return false;
  // Skip known non-semver forms
  const lowers = s.toLowerCase();
  if (
    lowers.startsWith('workspace:') ||
    lowers.startsWith('file:') ||
    lowers.startsWith('link:') ||
    lowers.startsWith('git+') ||
    lowers.startsWith('git:') ||
    lowers.startsWith('http:') ||
    lowers.startsWith('https:') ||
    lowers.startsWith('npm:') ||
    lowers === 'latest' || lowers === 'beta' || lowers === 'canary'
  ) {
    return false;
  }
  return /(\d+)(?:\.(\d+))?(?:\.(\d+))?/.test(s);
}

function makeMajorOnlyRange(v) {
  const nextMajor = v.major === 0 ? 1 : v.major + 1;
  return `>=${v.major}.${v.minor}.${v.patch} <${nextMajor}.0.0`;
}

async function readJson(p) {
  return JSON.parse(await fs.readFile(p, 'utf8'));
}

async function writeJson(p, obj) {
  const out = JSON.stringify(obj, null, 2) + '\n';
  await fs.writeFile(p, out, 'utf8');
}

async function readCrateVersion(root) {
  const cargoPath = path.join(root, 'Cargo.toml');
  try {
    const txt = await fs.readFile(cargoPath, 'utf8');
    const m = txt.match(/version\s*=\s*"(\d+)\.(\d+)\.(\d+)"/);
    if (!m) return null;
    return { major: parseInt(m[1], 10), minor: parseInt(m[2], 10), patch: parseInt(m[3], 10) };
  } catch {
    return null;
  }
}

function readPublishedNpmVersion(pkgName) {
  try {
    const out = execSync(`npm view ${pkgName} version`, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
    const m = out.match(/^(\d+)\.(\d+)\.(\d+)$/);
    if (!m) return null;
    return { major: parseInt(m[1], 10), minor: parseInt(m[2], 10), patch: parseInt(m[3], 10) };
  } catch {
    return null;
  }
}

function widenMap(depMap) {
  if (!depMap) return false;
  let changed = false;
  for (const [name, spec] of Object.entries(depMap)) {
    if (!isSemverLike(spec)) continue;
    const v = extractVersion(spec);
    if (!v) continue;
    const widened = makeMajorOnlyRange(v);
    if (depMap[name] !== widened) {
      depMap[name] = widened;
      changed = true;
    }
  }
  return changed;
}

async function widenPackageJson(file, crateVersionRangeOverride) {
  let obj;
  try { obj = await readJson(file); } catch { return false; }
  let changed = false;

  // Apply general widening
  if (widenMap(obj.dependencies)) changed = true;
  if (widenMap(obj.devDependencies)) changed = true;

  // Ensure fragmentcolor aligns to crate version
  if (obj.dependencies && Object.prototype.hasOwnProperty.call(obj.dependencies, 'fragmentcolor') && crateVersionRangeOverride) {
    if (obj.dependencies.fragmentcolor !== crateVersionRangeOverride) {
      obj.dependencies.fragmentcolor = crateVersionRangeOverride;
      changed = true;
    }
  }

  if (changed) await writeJson(file, obj);
  return changed;
}

export async function widenWorkspaceDeps(root = repoRoot) {
  // Determine fragmentcolor aligned range using the latest published npm version (policy: website must depend on latest published)
  const pv = readPublishedNpmVersion('fragmentcolor');
  let fcRange = null;
  if (pv) fcRange = makeMajorOnlyRange(pv);

  // Scan roots
  const targets = [];
  const docsPkg = path.join(root, 'docs', 'website', 'package.json');
  if (await exists(docsPkg)) targets.push(docsPkg);

  for (const base of [path.join(root, 'platforms', 'web'), path.join(root, 'examples')]) {
    if (!(await exists(base))) continue;
    for await (const pkg of walk(base, 5)) targets.push(pkg);
  }

  let total = 0;
  for (const file of targets) {
    const rel = path.relative(root, file);
    const did = await widenPackageJson(file, fcRange);
    if (did) {
      total++;
      console.log(`[widen] updated ${rel}`);
    }
  }
  if (total === 0) console.log('[widen] no changes');
}

// Allow running directly
if (import.meta.url === `file://${process.argv[1]}`) {
  widenWorkspaceDeps().catch((e) => { console.error(e); process.exit(1); });
}
