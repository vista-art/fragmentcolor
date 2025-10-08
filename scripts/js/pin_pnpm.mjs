#!/usr/bin/env node
/*
Pin the pnpm packageManager version across JS subprojects in this repo.

Usage:
  node scripts/js/pin_pnpm.mjs [version]

Examples:
  node scripts/js/pin_pnpm.mjs 10.15.1
  node scripts/js/pin_pnpm.mjs 10.14.0

Notes:
- Scans these roots: docs/website, platforms/web/**, examples/**
- Skips: node_modules, pkg, dist, build, .astro
*/

import { promises as fs } from 'node:fs';
import path from 'node:path';

const repoRoot = process.cwd();
const version = process.argv[2] || '10.15.1';
const value = `pnpm@${version}`;

const SKIP_DIRS = new Set(['node_modules', 'pkg', 'dist', 'build', '.astro']);
const SCAN_ROOTS = [
  path.join(repoRoot, 'docs', 'website'),
  path.join(repoRoot, 'platforms', 'web'),
  path.join(repoRoot, 'examples'),
];

function shouldSkipDir(p) {
  const base = path.basename(p);
  if (SKIP_DIRS.has(base)) return true;
  return false;
}

async function exists(p) {
  try { await fs.stat(p); return true; } catch { return false; }
}

async function* walk(dir, maxDepth = 4, depth = 0) {
  if (depth > maxDepth) return;
  let ents;
  try {
    ents = await fs.readdir(dir, { withFileTypes: true });
  } catch {
    return;
  }
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

async function updatePackageJson(file) {
  let raw;
  try {
    raw = await fs.readFile(file, 'utf8');
  } catch {
    return false;
  }
  let obj;
  try {
    obj = JSON.parse(raw);
  } catch {
    return false;
  }
  const prev = obj.packageManager;
  if (prev === value) return false;
  obj.packageManager = value;
  const out = JSON.stringify(obj, null, 2) + '\n';
  await fs.writeFile(file, out, 'utf8');
  return true;
}

async function main() {
  const targets = new Set();

  // docs/website direct
  const docsPkg = path.join(repoRoot, 'docs', 'website', 'package.json');
  if (await exists(docsPkg)) targets.add(docsPkg);

  // platforms/web/** and examples/**
  for (const root of [path.join(repoRoot, 'platforms', 'web'), path.join(repoRoot, 'examples')]) {
    if (!(await exists(root))) continue;
    for await (const pkg of walk(root, 4)) {
      targets.add(pkg);
    }
  }

  let changed = 0;
  for (const file of targets) {
    const did = await updatePackageJson(file);
    if (did) {
      changed++;
      console.log(`updated packageManager in ${path.relative(repoRoot, file)} -> ${value}`);
    }
  }
  if (changed === 0) {
    console.log('No package.json files needed changes.');
  } else {
    console.log(`Updated ${changed} file(s).`);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
