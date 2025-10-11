#!/usr/bin/env node
/*
Fail only on major updates for pnpm projects by inspecting pnpm outdated --json.

Usage:
  node scripts/js/ci_fail_on_major_updates.mjs --dir <path> [--dir <path> ...]

Behavior:
- For each provided directory, run: pnpm --dir <path> outdated --json
- Parse JSON objects; for each dependency, read { current, wanted, latest }
- If semver major(latest) > major(wanted), flag as major update available.
- Exit 1 if any flagged packages are found; otherwise exit 0.

This ignores minor/patch updates as requested.
*/

import { spawn } from 'node:child_process';

function parseArgs(argv) {
  const dirs = [];
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--dir') {
      if (!argv[i + 1]) throw new Error('Missing value for --dir');
      dirs.push(argv[++i]);
    }
  }
  if (dirs.length === 0) throw new Error('Provide at least one --dir <path>');
  return { dirs };
}

function run(cmd, args, opts = {}) {
  return new Promise((resolve, reject) => {
    const p = spawn(cmd, args, { stdio: ['ignore', 'pipe', 'pipe'], ...opts });
    let out = '';
    let err = '';
    p.stdout.on('data', (d) => (out += d.toString()));
    p.stderr.on('data', (d) => (err += d.toString()));
    p.on('close', (code) => {
      if (code === 0) resolve({ code, out, err });
      else resolve({ code, out, err }); // pnpm outdated may exit non-zero when outdated present
    });
    p.on('error', reject);
  });
}

function major(x) {
  const m = String(x ?? '').match(/(\d+)/);
  return m ? parseInt(m[1], 10) : null;
}

async function checkDir(dir) {
  const res = await run('pnpm', ['--dir', dir, 'outdated', '--json']);
  const list = [];
  if (!res.out.trim()) return list; // nothing

  // pnpm --json output format differs across versions/workspaces:
  // - Sometimes an array of objects
  // - Sometimes an object map (name -> info)
  // - In some setups, an object with a packages: [] field
  let parsed;
  try {
    parsed = JSON.parse(res.out);
  } catch {
    return list;
  }

  let items = [];
  if (Array.isArray(parsed)) {
    items = parsed;
  } else if (parsed && typeof parsed === 'object') {
    if (Array.isArray(parsed.packages)) {
      items = parsed.packages;
    } else {
      items = Object.values(parsed);
    }
  } else {
    return list;
  }

  for (const item of items) {
    const wantedMaj = major(item.wanted);
    const latestMaj = major(item.latest);
    if (wantedMaj == null || latestMaj == null) continue;
    if (latestMaj > wantedMaj) {
      const name = item.package ?? item.name ?? item.depName ?? item.id ?? 'unknown';
      list.push({ name, wanted: item.wanted, latest: item.latest });
    }
  }
  return list;
}

async function main() {
  const { dirs } = parseArgs(process.argv);
  let majors = [];
  for (const d of dirs) {
    const flagged = await checkDir(d);
    majors = majors.concat(flagged.map((x) => ({ ...x, dir: d })));
  }
  if (majors.length === 0) {
    console.log('No major updates available.');
    return;
  }
  console.error('Major updates available (failing CI):');
  for (const m of majors) {
    console.error(`  [${m.dir}] ${m.name}: wanted ${m.wanted} -> latest ${m.latest}`);
  }
  process.exit(1);
}

main().catch((e) => { console.error(e); process.exit(1); });
