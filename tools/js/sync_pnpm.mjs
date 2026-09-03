#!/usr/bin/env node
/*
Sync pnpm dependencies across JS subprojects in this repo.

Behavior:
- By default: --frozen in CI (process.env.CI truthy), --no-frozen locally.
- Accepts flags:
  --frozen       Use frozen lockfile (fails if lockfiles are stale)
  --no-frozen    Update lockfiles as needed
  --list         Only list discovered projects, do not install

Scan roots:
- docs/website (direct)
- platforms/web/**
- examples/**

Skips directories: node_modules, pkg, dist, build, .astro
Only pnpm projects are included: (pnpm-lock.yaml exists) OR (package.json has packageManager: pnpm@...)
Also widens dependency ranges to "major-only" (>=current <nextMajor) before installs.
*/

import { promises as fs } from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';

const repoRoot = process.cwd();

// Lazily import the widen script to keep this file standalone if needed
async function widenAll() {
  try {
    const mod = await import(new URL('./widen_dep_ranges.mjs', import.meta.url));
    if (typeof mod.widenWorkspaceDeps === 'function') {
      await mod.widenWorkspaceDeps(repoRoot);
    }
  } catch (e) {
    console.warn('[sync_pnpm] widen_dep_ranges.mjs not found or failed, continuing:', e?.message || e);
  }
}

function parseArgs(argv) {
  const out = { frozen: undefined, list: false };
  for (const a of argv.slice(2)) {
    if (a === '--frozen') out.frozen = true;
    else if (a === '--no-frozen') out.frozen = false;
    else if (a === '--list') out.list = true;
  }
  if (out.frozen === undefined) {
    out.frozen = Boolean(process.env.CI);
  }
  return out;
}

const SKIP_DIRS = new Set(['node_modules', 'pkg', 'dist', 'build', '.astro']);
function shouldSkipDir(p) {
  const base = path.basename(p);
  if (SKIP_DIRS.has(base)) return true;
  return false;
}

async function exists(p) {
  try { await fs.stat(p); return true; } catch { return false; }
}

async function readJson(p) {
  const s = await fs.readFile(p, 'utf8');
  return JSON.parse(s);
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

async function isPnpmProject(dir, pkgPath) {
  if (await exists(path.join(dir, 'pnpm-lock.yaml'))) return true;
  try {
    const pkg = await readJson(pkgPath);
    return typeof pkg.packageManager === 'string' && pkg.packageManager.startsWith('pnpm@');
  } catch {
    return false;
  }
}

async function discoverProjects() {
  const targets = new Set();
  // docs/website
  const docsPkg = path.join(repoRoot, 'docs', 'website', 'package.json');
  if (await exists(docsPkg)) {
    if (await isPnpmProject(path.dirname(docsPkg), docsPkg)) {
      targets.add(path.dirname(docsPkg));
    }
  }
  // platforms/web/** and examples/**
  for (const root of [path.join(repoRoot, 'platforms', 'web'), path.join(repoRoot, 'examples')]) {
    if (!(await exists(root))) continue;
    for await (const pkg of walk(root, 5)) {
      const dir = path.dirname(pkg);
      if (shouldSkipDir(dir)) continue;
      if (await isPnpmProject(dir, pkg)) targets.add(dir);
    }
  }
  return Array.from(targets);
}

function run(cmd, args, opts = {}) {
  return new Promise((resolve, reject) => {
    const p = spawn(cmd, args, { stdio: 'inherit', ...opts });
    p.on('close', (code) => {
      if (code === 0) resolve(); else reject(new Error(`${cmd} ${args.join(' ')} exited with ${code}`));
    });
    p.on('error', reject);
  });
}

async function main() {
  const args = parseArgs(process.argv);
  const projects = await discoverProjects();
  if (args.list) {
    for (const d of projects) console.log(d);
    return;
  }
  if (projects.length === 0) {
    console.log('[sync_pnpm] no pnpm projects discovered');
    return;
  }

  // Widen dependency ranges to major-only before installations
  await widenAll();

  const flag = args.frozen ? '--frozen-lockfile' : '--no-frozen-lockfile';
  console.log(`[sync_pnpm] syncing ${projects.length} project(s) using ${flag}`);
  for (const d of projects) {
    console.log(`  • ${d}`);
    await run('pnpm', ['--dir', d, 'install', flag]);
  }
}

main().catch((err) => {
  console.error(err?.message || err);
  process.exit(1);
});
